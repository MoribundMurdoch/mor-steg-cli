use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

pub fn embed_payload(
    cover_file: &Path,
    encrypted_payload: &Path,
    stego_output: &Path,
) -> io::Result<ExitStatus> {
    Command::new("steghide")
        .arg("embed")
        .arg("-cf")
        .arg(cover_file)
        .arg("-ef")
        .arg(encrypted_payload)
        .arg("-sf")
        .arg(stego_output)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

pub fn extract_payload(stego_file: &Path, encrypted_payload_output: &Path) -> io::Result<ExitStatus> {
    Command::new("steghide")
        .arg("extract")
        .arg("-sf")
        .arg(stego_file)
        .arg("-xf")
        .arg(encrypted_payload_output)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

pub fn inspect_file(file: &Path) -> io::Result<ExitStatus> {
    Command::new("steghide")
        .arg("info")
        .arg(file)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}
