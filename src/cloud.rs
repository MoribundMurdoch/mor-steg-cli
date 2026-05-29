use std::fs::{self, File};
use std::io::{self, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub struct CloudPackageReport {
    pub output: PathBuf,
    pub entries: Vec<String>,
}

struct CentralEntry {
    name: String,
    crc32: u32,
    size: u32,
    local_header_offset: u32,
}

pub fn package_one_file(input_file: &Path, output_zip: &Path) -> io::Result<CloudPackageReport> {
    if !input_file.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "input is not a regular file",
        ));
    }

    ensure_parent_exists(output_zip)?;

    let mut writer = File::create(output_zip)?;
    let mut central_entries = Vec::new();
    let mut names = Vec::new();

    let original_name = input_file
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("steg-file");

    let file_entry_name = format!("files/{}", safe_zip_name(original_name));
    append_file_entry(&mut writer, &mut central_entries, input_file, &file_entry_name)?;
    names.push(file_entry_name.clone());

    let readme = cloud_safe_readme();
    append_bytes_entry(
        &mut writer,
        &mut central_entries,
        "README_MorSteg_Cloud_Safe.txt",
        readme.as_bytes(),
    )?;
    names.push("README_MorSteg_Cloud_Safe.txt".to_string());

    let manifest = format!(
        "MorSteg cloud-safe package\npackage_type=single_file\nprotected_file={file_entry_name}\n"
    );
    append_bytes_entry(
        &mut writer,
        &mut central_entries,
        "manifest_MorSteg.txt",
        manifest.as_bytes(),
    )?;
    names.push("manifest_MorSteg.txt".to_string());

    finish_zip(&mut writer, &central_entries)?;

    Ok(CloudPackageReport {
        output: output_zip.to_path_buf(),
        entries: names,
    })
}

pub fn package_directory(input_dir: &Path, output_zip: &Path) -> io::Result<CloudPackageReport> {
    if !input_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "input is not a directory",
        ));
    }

    ensure_parent_exists(output_zip)?;

    let mut files = Vec::new();
    collect_files(input_dir, input_dir, &mut files)?;
    files.sort_by(|left, right| left.0.cmp(&right.0));

    let mut writer = File::create(output_zip)?;
    let mut central_entries = Vec::new();
    let mut names = Vec::new();

    for (relative_name, path) in files {
        let entry_name = format!("files/{}", safe_zip_path(&relative_name));
        append_file_entry(&mut writer, &mut central_entries, &path, &entry_name)?;
        names.push(entry_name);
    }

    let readme = cloud_safe_readme();
    append_bytes_entry(
        &mut writer,
        &mut central_entries,
        "README_MorSteg_Cloud_Safe.txt",
        readme.as_bytes(),
    )?;
    names.push("README_MorSteg_Cloud_Safe.txt".to_string());

    let manifest = format!(
        "MorSteg cloud-safe package\npackage_type=directory\nfile_count={}\n",
        names.len().saturating_sub(1)
    );
    append_bytes_entry(
        &mut writer,
        &mut central_entries,
        "manifest_MorSteg.txt",
        manifest.as_bytes(),
    )?;
    names.push("manifest_MorSteg.txt".to_string());

    finish_zip(&mut writer, &central_entries)?;

    Ok(CloudPackageReport {
        output: output_zip.to_path_buf(),
        entries: names,
    })
}

fn cloud_safe_readme() -> String {
    r#"MorSteg Cloud-Safe Package

This archive is meant to protect steghide carrier files from cloud/social media recompression.

Plain English:
- Upload this .MorSteg.zip file to the cloud.
- Do not upload the raw steghide image/audio file to sites that recompress media.
- Extract the file from this archive before opening it with MorSteg.

Why:
Steghide needs the carrier file bytes to survive unchanged.
If a site resizes, recompresses, or converts the image/audio file, the hidden data may be destroyed.

This ZIP uses normal "store" entries. It is not encryption.
The secrecy comes from MorSteg's age encryption step.
"#
    .to_string()
}

fn collect_files(base: &Path, current: &Path, output: &mut Vec<(String, PathBuf)>) -> io::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_files(base, &path, output)?;
        } else if path.is_file() {
            let relative = path.strip_prefix(base).unwrap_or(&path);
            let relative_name = relative.to_string_lossy().replace('\\', "/");
            output.push((relative_name, path));
        }
    }

    Ok(())
}

fn append_file_entry(
    writer: &mut File,
    central_entries: &mut Vec<CentralEntry>,
    path: &Path,
    entry_name: &str,
) -> io::Result<()> {
    let size_u64 = fs::metadata(path)?.len();

    if size_u64 > u32::MAX as u64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "file is too large for this simple ZIP writer",
        ));
    }

    let crc = crc32_file(path)?;
    let size = size_u64 as u32;
    let offset = current_offset_u32(writer)?;

    write_local_header(writer, entry_name, crc, size)?;

    let mut reader = BufReader::new(File::open(path)?);
    io::copy(&mut reader, writer)?;

    central_entries.push(CentralEntry {
        name: entry_name.to_string(),
        crc32: crc,
        size,
        local_header_offset: offset,
    });

    Ok(())
}

fn append_bytes_entry(
    writer: &mut File,
    central_entries: &mut Vec<CentralEntry>,
    entry_name: &str,
    bytes: &[u8],
) -> io::Result<()> {
    if bytes.len() > u32::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "generated entry is too large",
        ));
    }

    let crc = crc32_bytes(bytes);
    let size = bytes.len() as u32;
    let offset = current_offset_u32(writer)?;

    write_local_header(writer, entry_name, crc, size)?;
    writer.write_all(bytes)?;

    central_entries.push(CentralEntry {
        name: entry_name.to_string(),
        crc32: crc,
        size,
        local_header_offset: offset,
    });

    Ok(())
}

fn finish_zip(writer: &mut File, central_entries: &[CentralEntry]) -> io::Result<()> {
    let central_start = current_offset_u32(writer)?;

    for entry in central_entries {
        write_central_header(writer, entry)?;
    }

    let central_end = current_offset_u32(writer)?;
    let central_size = central_end
        .checked_sub(central_start)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "bad ZIP central directory size"))?;

    if central_entries.len() > u16::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "too many files for this simple ZIP writer",
        ));
    }

    write_end_of_central_directory(
        writer,
        central_entries.len() as u16,
        central_size,
        central_start,
    )?;

    Ok(())
}

fn write_local_header(writer: &mut File, name: &str, crc32: u32, size: u32) -> io::Result<()> {
    let name_bytes = name.as_bytes();

    if name_bytes.len() > u16::MAX as usize {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "ZIP name too long"));
    }

    write_u32(writer, 0x0403_4b50)?;
    write_u16(writer, 20)?;
    write_u16(writer, 0x0800)?;
    write_u16(writer, 0)?;
    write_u16(writer, 0)?;
    write_u16(writer, 0)?;
    write_u32(writer, crc32)?;
    write_u32(writer, size)?;
    write_u32(writer, size)?;
    write_u16(writer, name_bytes.len() as u16)?;
    write_u16(writer, 0)?;
    writer.write_all(name_bytes)?;
    Ok(())
}

fn write_central_header(writer: &mut File, entry: &CentralEntry) -> io::Result<()> {
    let name_bytes = entry.name.as_bytes();

    if name_bytes.len() > u16::MAX as usize {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "ZIP name too long"));
    }

    write_u32(writer, 0x0201_4b50)?;
    write_u16(writer, 20)?;
    write_u16(writer, 20)?;
    write_u16(writer, 0x0800)?;
    write_u16(writer, 0)?;
    write_u16(writer, 0)?;
    write_u16(writer, 0)?;
    write_u32(writer, entry.crc32)?;
    write_u32(writer, entry.size)?;
    write_u32(writer, entry.size)?;
    write_u16(writer, name_bytes.len() as u16)?;
    write_u16(writer, 0)?;
    write_u16(writer, 0)?;
    write_u16(writer, 0)?;
    write_u16(writer, 0)?;
    write_u32(writer, 0)?;
    write_u32(writer, entry.local_header_offset)?;
    writer.write_all(name_bytes)?;
    Ok(())
}

fn write_end_of_central_directory(
    writer: &mut File,
    entry_count: u16,
    central_size: u32,
    central_offset: u32,
) -> io::Result<()> {
    write_u32(writer, 0x0605_4b50)?;
    write_u16(writer, 0)?;
    write_u16(writer, 0)?;
    write_u16(writer, entry_count)?;
    write_u16(writer, entry_count)?;
    write_u32(writer, central_size)?;
    write_u32(writer, central_offset)?;
    write_u16(writer, 0)?;
    Ok(())
}

fn current_offset_u32(writer: &mut File) -> io::Result<u32> {
    let offset = writer.seek(SeekFrom::Current(0))?;

    if offset > u32::MAX as u64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "ZIP file is too large for this simple ZIP writer",
        ));
    }

    Ok(offset as u32)
}

fn crc32_file(path: &Path) -> io::Result<u32> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut crc = Crc32::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = reader.read(&mut buffer)?;

        if read == 0 {
            break;
        }

        crc.update(&buffer[..read]);
    }

    Ok(crc.finalize())
}

fn crc32_bytes(bytes: &[u8]) -> u32 {
    let mut crc = Crc32::new();
    crc.update(bytes);
    crc.finalize()
}

struct Crc32 {
    value: u32,
}

impl Crc32 {
    fn new() -> Self {
        Self { value: 0xffff_ffff }
    }

    fn update(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.value ^= *byte as u32;

            for _ in 0..8 {
                if self.value & 1 != 0 {
                    self.value = (self.value >> 1) ^ 0xedb8_8320;
                } else {
                    self.value >>= 1;
                }
            }
        }
    }

    fn finalize(self) -> u32 {
        !self.value
    }
}

fn safe_zip_name(name: &str) -> String {
    name.replace('\\', "_").replace('/', "_")
}

fn safe_zip_path(path: &str) -> String {
    path.split('/')
        .filter(|part| !part.is_empty() && *part != "." && *part != "..")
        .map(safe_zip_name)
        .collect::<Vec<_>>()
        .join("/")
}

fn ensure_parent_exists(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    Ok(())
}

fn write_u16(writer: &mut File, value: u16) -> io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

fn write_u32(writer: &mut File, value: u32) -> io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

