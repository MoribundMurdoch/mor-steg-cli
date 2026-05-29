use std::path::{Path, PathBuf};

use crate::cloud;
use crate::paths::{
    active_cloud_dir, explain_cloud_folder, prompt_existing_file, prompt_existing_folder, AppConfig,
};
use crate::ui::{clear_screen, confirm, pause, print_banner};

pub fn cloud_safe_one_file_flow(config: &AppConfig) {
    clear_screen();
    print_banner();

    println!("Make one steghide file cloud-safe.\n");
    println!("Use this after you have created a disguised steghide output file.");
    println!();
    println!("Why?");
    println!("  Some cloud/social sites recompress images.");
    println!("  Recompression can destroy hidden steghide data.");
    println!("  MorSteg will wrap the exact file bytes in a .MorSteg.zip package.");
    println!();
    println!("Upload the .MorSteg.zip file, not the raw image/audio file.");
    println!();
    explain_cloud_folder(config);
    println!();

    let input_file = prompt_existing_file("Steghide file to protect");

    let output_zip = suggested_cloud_zip_path(&input_file, config);

    println!();
    println!("MorSteg will create:");
    println!("  {}", output_zip.display());
    println!();

    if !confirm("Create this cloud-safe package?") {
        println!("Cancelled.");
        pause();
        return;
    }

    match cloud::package_one_file(&input_file, &output_zip) {
        Ok(report) => {
            println!();
            println!("Cloud-safe package created.");
            println!("  {}", report.output.display());
            println!();
            println!("Inside the package:");
            for entry in report.entries {
                println!("  - {entry}");
            }
            println!();
            println!("Lazy rule:");
            println!("  Upload the .MorSteg.zip package to the cloud.");
            println!("  Do not upload the raw steghide image/audio file to sites that recompress media.");
        }
        Err(err) => {
            println!();
            println!("Could not create cloud-safe package.");
            println!("Error: {err}");
        }
    }

    pause();
}

pub fn cloud_safe_folder_flow(config: &AppConfig) {
    clear_screen();
    print_banner();

    println!("Make a whole folder of steghide files cloud-safe.\n");
    println!("Use this when you have a folder full of disguised steghide files.");
    println!("MorSteg will put the entire folder into one .MorSteg.zip package.");
    println!();
    println!("Upload the .MorSteg.zip package to the cloud.");
    println!("Do not upload the raw image/audio files to services that recompress media.");
    println!();
    explain_cloud_folder(config);
    println!();

    let input_dir = prompt_existing_folder("Folder to protect");

    let output_zip = suggested_cloud_zip_path(&input_dir, config);

    println!();
    println!("MorSteg will create:");
    println!("  {}", output_zip.display());
    println!();

    if !confirm("Create this cloud-safe package?") {
        println!("Cancelled.");
        pause();
        return;
    }

    match cloud::package_directory(&input_dir, &output_zip) {
        Ok(report) => {
            println!();
            println!("Cloud-safe folder package created.");
            println!("  {}", report.output.display());
            println!();
            println!("Packaged {} item(s).", report.entries.len());
            println!();
            println!("Lazy rule:");
            println!("  Upload the .MorSteg.zip package to the cloud.");
            println!("  Keep the raw steghide files out of image/video/social uploaders.");
        }
        Err(err) => {
            println!();
            println!("Could not create cloud-safe folder package.");
            println!("Error: {err}");
        }
    }

    pause();
}

fn suggested_cloud_zip_path(input: &Path, config: &AppConfig) -> PathBuf {
    let base_dir = active_cloud_dir(config);

    let name = input
        .file_stem()
        .and_then(|value| value.to_str())
        .or_else(|| input.file_name().and_then(|value| value.to_str()))
        .unwrap_or("MorSteg-package");

    base_dir.join(format!("{name}.MorSteg.zip"))
}
