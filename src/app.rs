use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::cloud;
use crate::crypto::{self, DecryptionMode, EncryptionMode};
use crate::deps;
use crate::paths::{
    self, active_cloud_dir, active_key_dir, explain_cloud_folder, explain_key_folder,
    explain_output_folder, prompt_existing_file, prompt_existing_folder, prompt_folder_to_create,
    prompt_output_file, to_absolute_path, AppConfig,
};
use crate::steg;
use crate::ui::{BUILD_MARKER, clear_screen, confirm, pause, print_banner, prompt, prompt_nonempty};

pub fn run() {
    let mut config = AppConfig::default();

    loop {
        clear_screen();
        print_banner();

        println!("Build: {BUILD_MARKER}");
        println!();

        match &config.output_dir {
            Some(dir) => println!("New steghide/output files save in: {}", dir.display()),
            None => println!("New steghide/output files save in: the folder you started MorSteg from"),
        }

        println!("Age key files save in: {}", active_key_dir(&config).display());
        println!("Cloud-safe packages save in: {}", active_cloud_dir(&config).display());

        println!();
        println!("What do you want to do?\n");

        println!("Easy password mode:");
        println!("[1] Hide a file using a password");
        println!("[2] Open a hidden file using a password");
        println!();

        println!("Age key mode:");
        println!("[3] Hide a file for someone else using their public age key");
        println!("[4] Open a hidden file using my private age key file");
        println!("[5] Age key helper");
        println!();

        println!("Save locations:");
        println!("[6] Choose where steghide/output files should be saved");
        println!("[7] Choose where age key files should be saved");
        println!("[8] Choose where cloud-safe packages should be saved");
        println!();

        println!("Cloud safety:");
        println!("[9] Make one steghide file cloud-safe");
        println!("[10] Make a whole folder of steghide files cloud-safe");
        println!();

        println!("Tools:");
        println!("[11] Check a file for hidden steghide info");
        println!("[12] Check if MorSteg has the tools it needs");
        println!("[13] Quit\n");

        println!("Not sure? Use [1] to hide a file and [2] to open it later.\n");

        match prompt("Choose").trim() {
            "1" => embed_flow(&config, EncryptionMode::AgePassphrase),
            "2" => extract_flow(&config, DecryptionMode::AgePassphrase),
            "3" => embed_with_recipient_flow(&config),
            "4" => extract_with_identity_flow(&config),
            "5" => age_key_helper_flow(&config),
            "6" => choose_output_folder_flow(&mut config),
            "7" => choose_key_folder_flow(&mut config),
            "8" => choose_cloud_folder_flow(&mut config),
            "9" => cloud_safe_one_file_flow(&config),
            "10" => cloud_safe_folder_flow(&config),
            "11" => inspect_flow(),
            "12" => check_deps_flow(),
            "13" | "q" | "quit" | "exit" => {
                println!("Goodbye.");
                break;
            }
            _ => {
                println!("\nUnknown option.");
                pause();
            }
        }
    }
}

fn embed_with_recipient_flow(config: &AppConfig) {
    clear_screen();
    print_banner();

    println!("Hide a file for someone else using their public age key.\n");
    println!("Use this when someone gave you an age recipient key.");
    println!("That public key is safe to share. It lets you lock a file for them.");
    println!("Only their matching private age key file can unlock it.");
    println!();
    println!("Public age keys usually look like one of these:");
    println!("  age1...");
    println!("  age1pq1...");
    println!();

    let recipient = prompt_nonempty("Paste the public age key");

    let mode = EncryptionMode::AgeRecipient { recipient };
    embed_flow(config, mode);
}

fn embed_flow(config: &AppConfig, encryption_mode: EncryptionMode) {
    clear_screen();
    print_banner();

    println!("Build: {BUILD_MARKER}\n");
    println!("Hide a file inside a normal-looking file.");
    println!("MorSteg first encrypts your secret with age.");
    println!("Then it asks steghide to hide that encrypted blob inside the cover file.\n");

    crypto::print_mode_note(&encryption_mode);
    println!();

    if !deps::core_dependencies_exist() {
        deps::print_deps_missing();
        pause();
        return;
    }

    println!("Step 1 of 3: Choose the normal-looking cover file.");
    println!("This is the file people will see.");
    println!("It is usually a .jpg, .jpeg, .bmp, .wav, or .au file.");
    println!();
    println!("Tip: You can drag the file into this terminal and press Enter.");
    println!();

    let cover_file = prompt_existing_file("Cover file");

    println!("\nStep 2 of 3: Choose the secret file you want to hide.");
    println!("This can be a text file, zip file, PDF, image, or almost anything else.");
    println!();
    println!("Tip: You can drag the file into this terminal and press Enter.");
    println!();

    let secret_file = prompt_existing_file("Secret file");

    println!("\nStep 3 of 3: Choose the new disguised output file name.");
    explain_output_folder(config);
    println!("Example: vacation-photo.jpg");
    println!();

    let stego_file = prompt_output_file("Output file", config);

    println!("\nReady to hide your secret:");
    println!("  Secret file: {}", to_absolute_path(&secret_file).display());
    println!("  Cover file:  {}", to_absolute_path(&cover_file).display());
    println!("  Output file: {}", to_absolute_path(&stego_file).display());

    if !confirm("\nContinue?") {
        println!("Cancelled.");
        pause();
        return;
    }

    let (_temp_workspace, temp_payload) = match paths::create_private_temp_payload_path() {
        Ok(paths) => paths,
        Err(err) => {
            println!("\nCould not create a private temporary workspace.");
            println!("Error: {err}");
            pause();
            return;
        }
    };

    println!("\n--- [ STEP A: ENCRYPT THE SECRET ] ---");

    let age_status = crypto::encrypt_payload(&encryption_mode, &secret_file, &temp_payload);

    if !status_success(age_status, "Encryption") {
        pause();
        return;
    }

    println!("\n--- [ STEP B: HIDE THE ENCRYPTED SECRET ] ---");
    println!("The secret is now encrypted in a private temporary workspace.");
    println!("Now steghide will hide the encrypted blob in the cover file.");
    println!("Steghide may ask for a second password.");
    println!("You can type one, or press Enter to leave the steghide password blank.");

    let steg_status = steg::embed_payload(&cover_file, &temp_payload, &stego_file);

    show_status("Hiding", steg_status);
    println!("Temporary encrypted workspace cleaned up automatically.");
    pause();
}

fn extract_with_identity_flow(config: &AppConfig) {
    clear_screen();
    print_banner();

    println!("Open a hidden file using my private age key file.\n");
    println!("Use this when the hidden file was locked for your public age key.");
    println!("You need the matching private age key file to unlock it.");
    println!();
    println!("Tip: You can drag the key file into this terminal and press Enter.");
    println!();

    let identity_file = prompt_existing_file("Private age key file");

    let mode = DecryptionMode::AgeIdentityFile { identity_file };
    extract_flow(config, mode);
}

fn extract_flow(config: &AppConfig, decryption_mode: DecryptionMode) {
    clear_screen();
    print_banner();

    println!("Build: {BUILD_MARKER}\n");
    println!("Open a hidden file and save the secret back out.\n");

    if !deps::core_dependencies_exist() {
        deps::print_deps_missing();
        pause();
        return;
    }

    println!("Step 1 of 2: Choose the file that has a secret hidden inside it.");
    println!("Tip: You can drag the file into this terminal and press Enter.");
    println!();

    let stego_file = prompt_existing_file("File with hidden secret");

    println!("\nStep 2 of 2: Choose where to save the recovered secret.");
    explain_output_folder(config);
    println!("Example: recovered-notes.txt");
    println!();

    let final_output = prompt_output_file("Recovered file name", config);

    if !confirm("\nContinue?") {
        println!("Cancelled.");
        pause();
        return;
    }

    let (_temp_workspace, temp_payload) = match paths::create_private_temp_payload_path() {
        Ok(paths) => paths,
        Err(err) => {
            println!("\nCould not create a private temporary workspace.");
            println!("Error: {err}");
            pause();
            return;
        }
    };

    println!("\n--- [ STEP A: PULL OUT THE HIDDEN ENCRYPTED BLOB ] ---");
    println!("Steghide may ask for the steghide password.");
    println!("If you left that blank when hiding the file, press Enter.");

    let steg_status = steg::extract_payload(&stego_file, &temp_payload);

    if !status_success(steg_status, "Extraction") {
        pause();
        return;
    }

    println!("\n--- [ STEP B: DECRYPT THE SECRET ] ---");

    let age_status = crypto::decrypt_payload(&decryption_mode, &temp_payload, &final_output);

    if age_status.map_or(false, |s| s.success()) {
        println!("\nSuccess! Secret recovered and saved to:");
        println!("  {}", to_absolute_path(&final_output).display());
    } else {
        println!("\nDecryption failed. The password or age key file may be incorrect.");
    }

    println!("Temporary encrypted workspace cleaned up automatically.");
    pause();
}

fn cloud_safe_one_file_flow(config: &AppConfig) {
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

fn cloud_safe_folder_flow(config: &AppConfig) {
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

fn inspect_flow() {
    clear_screen();
    print_banner();

    println!("Build: {BUILD_MARKER}\n");
    println!("Check a file for steghide info.\n");
    println!("This does not open or decrypt anything.");
    println!("It only asks steghide what it can tell about the file.");
    println!();

    if !deps::steghide_exists() {
        deps::print_deps_missing();
        pause();
        return;
    }

    println!("Tip: You can drag the file into this terminal and press Enter.");
    println!();

    let file = prompt_existing_file("File to check");

    let status = steg::inspect_file(&file);

    show_status("Check", status);
    pause();
}

fn check_deps_flow() {
    clear_screen();
    print_banner();

    println!("Build: {BUILD_MARKER}\n");
    println!("Checking the tools MorSteg needs...\n");

    if deps::steghide_exists() {
        println!("[OK] steghide is installed. This hides and extracts the encrypted blob.");
    } else {
        println!("[MISSING] steghide is not installed.");
    }

    if deps::age_exists() {
        println!("[OK] age is installed. This encrypts and decrypts the secret.");
    } else {
        println!("[MISSING] age is not installed.");
    }

    if deps::age_keygen_exists() {
        println!("[OK] age-keygen is installed. This can make age key files.");
    } else {
        println!("[WARN] age-keygen was not found. You only need it if you want to create age keys.");
    }

    if !deps::core_dependencies_exist() {
        println!();
        deps::print_deps_missing();
    }

    pause();
}

fn choose_output_folder_flow(config: &mut AppConfig) {
    clear_screen();
    print_banner();

    println!("Choose where steghide/output files should be saved.\n");
    println!("This controls:");
    println!("  - disguised files created by hiding");
    println!("  - recovered files created by opening");
    println!();
    println!("It does NOT control age key files. Those have their own folder.");
    println!();

    match &config.output_dir {
        Some(dir) => println!("Current steghide/output save folder:\n  {}", dir.display()),
        None => println!("Current steghide/output save folder:\n  the folder you started MorSteg from"),
    }

    println!();
    println!("Lazy mode:");
    println!("  Paste or drag a folder path here and press Enter.");
    println!();
    println!("Other choices:");
    println!("[1] Choose a steghide/output save folder");
    println!("[2] Clear this folder and use the current folder");
    println!("[3] Return to main menu");
    println!();

    let choice = prompt("Folder path or choice");

    match choice.trim() {
        "1" => {
            println!();
            println!("Enter a folder path.");
            println!("Tip: You can drag a folder into this terminal and press Enter.");
            println!();

            let folder = prompt_folder_to_create("Steghide/output save folder");
            set_output_folder(config, folder);
        }
        "2" => {
            config.output_dir = None;
            println!();
            println!("Steghide/output save folder cleared.");
        }
        "3" | "q" | "quit" | "back" => {
            println!();
            println!("No changes made.");
        }
        other => {
            if other.trim().is_empty() {
                println!();
                println!("No changes made.");
            } else {
                let folder = std::path::PathBuf::from(clean_menu_path(other));
                set_output_folder(config, folder);
            }
        }
    }

    pause();
}

fn choose_key_folder_flow(config: &mut AppConfig) {
    clear_screen();
    print_banner();

    println!("Choose where age key files should be saved.\n");
    println!("This controls:");
    println!("  - private age key files");
    println!("  - share-safe public age key files");
    println!();
    println!("It does NOT control disguised steghide output files.");
    println!();

    println!("Current age key folder:");
    println!("  {}", active_key_dir(config).display());
    println!();

    println!("Lazy mode:");
    println!("  Paste or drag a folder path here and press Enter.");
    println!();
    println!("Other choices:");
    println!("[1] Choose an age key folder");
    println!("[2] Reset to the default age key folder");
    println!("[3] Return to main menu");
    println!();

    let choice = prompt("Folder path or choice");

    match choice.trim() {
        "1" => {
            println!();
            println!("Enter a folder path.");
            println!("Tip: You can drag a folder into this terminal and press Enter.");
            println!();

            let folder = prompt_folder_to_create("Age key folder");
            set_key_folder(config, folder);
        }
        "2" => {
            config.key_dir = None;
            println!();
            println!("Age key folder reset to default:");
            println!("  {}", active_key_dir(config).display());
        }
        "3" | "q" | "quit" | "back" => {
            println!();
            println!("No changes made.");
        }
        other => {
            if other.trim().is_empty() {
                println!();
                println!("No changes made.");
            } else {
                let folder = std::path::PathBuf::from(clean_menu_path(other));
                set_key_folder(config, folder);
            }
        }
    }

    pause();
}

fn set_output_folder(config: &mut AppConfig, folder: std::path::PathBuf) {
    if let Err(err) = fs::create_dir_all(&folder) {
        println!("Could not create folder:");
        println!("  {}", folder.display());
        println!("Error: {err}");
    } else {
        config.output_dir = Some(folder);
        println!();
        println!("Steghide/output save folder updated:");
        if let Some(dir) = &config.output_dir {
            println!("  {}", dir.display());
        }
    }
}

fn set_key_folder(config: &mut AppConfig, folder: std::path::PathBuf) {
    if let Err(err) = fs::create_dir_all(&folder) {
        println!("Could not create folder:");
        println!("  {}", folder.display());
        println!("Error: {err}");
    } else {
        config.key_dir = Some(folder);
        println!();
        println!("Age key folder updated:");
        if let Some(dir) = &config.key_dir {
            println!("  {}", dir.display());
        }
    }
}

fn clean_menu_path(input: &str) -> String {
    let mut trimmed = input.trim();

    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        trimmed = &trimmed[1..trimmed.len() - 1];
    }

    let unescaped = trimmed.replace("\\ ", " ");

    if unescaped == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return home;
        }
    }

    if let Some(rest) = unescaped.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{home}/{rest}");
        }
    }

    unescaped
}

fn choose_cloud_folder_flow(config: &mut AppConfig) {
    clear_screen();
    print_banner();

    println!("Choose where cloud-safe packages should be saved.\n");
    println!("This controls:");
    println!("  - .MorSteg.zip packages made from one steghide file");
    println!("  - .MorSteg.zip packages made from a whole folder");
    println!();
    println!("It does NOT control raw steghide output files or age key files.");
    println!();

    println!("Current cloud-safe package folder:");
    println!("  {}", active_cloud_dir(config).display());
    println!();

    println!("Lazy mode:");
    println!("  Paste or drag a folder path here and press Enter.");
    println!();
    println!("Other choices:");
    println!("[1] Choose a cloud-safe package folder");
    println!("[2] Reset to the default cloud-safe folder");
    println!("[3] Return to main menu");
    println!();

    let choice = prompt("Folder path or choice");

    match choice.trim() {
        "1" => {
            println!();
            println!("Enter a folder path.");
            println!("Tip: You can drag a folder into this terminal and press Enter.");
            println!();

            let folder = prompt_folder_to_create("Cloud-safe package folder");
            set_cloud_folder(config, folder);
        }
        "2" => {
            config.cloud_dir = None;
            println!();
            println!("Cloud-safe package folder reset to default:");
            println!("  {}", active_cloud_dir(config).display());
        }
        "3" | "q" | "quit" | "back" => {
            println!();
            println!("No changes made.");
        }
        other => {
            if other.trim().is_empty() {
                println!();
                println!("No changes made.");
            } else {
                let folder = std::path::PathBuf::from(clean_menu_path(other));
                set_cloud_folder(config, folder);
            }
        }
    }

    pause();
}

fn set_cloud_folder(config: &mut AppConfig, folder: std::path::PathBuf) {
    if let Err(err) = fs::create_dir_all(&folder) {
        println!("Could not create folder:");
        println!("  {}", folder.display());
        println!("Error: {err}");
    } else {
        config.cloud_dir = Some(folder);
        println!();
        println!("Cloud-safe package folder updated:");
        if let Some(dir) = &config.cloud_dir {
            println!("  {}", dir.display());
        }
    }
}


fn age_key_helper_flow(config: &AppConfig) {
    loop {
        clear_screen();
        print_banner();

        println!("Age key helper.\n");
        println!("Use this when you want people to lock files for you without sharing a password.");
        println!();
        println!("There are two parts:");
        println!("  Public age key   = safe to share");
        println!("  Private key file = keep secret");
        println!();
        explain_key_folder(config);
        println!();

        println!("What do you want to do?\n");
        println!("[1] Make a post-quantum age key for me");
        println!("[2] Make a normal age key for me");
        println!("[3] Show the public key from one of my key files");
        println!("[4] Explain age keys like I am sleepy");
        println!("[5] Show where private key files should live");
        println!("[6] Return to main menu\n");

        match prompt("Choose").trim() {
            "1" => make_age_key_flow(config, KeyKind::PostQuantum),
            "2" => make_age_key_flow(config, KeyKind::Normal),
            "3" => show_public_key_from_file_flow(),
            "4" => explain_age_keys_sleepy_screen(),
            "5" => show_key_storage_advice_screen(config),
            "6" | "q" | "back" => break,
            _ => {
                println!("\nUnknown option.");
                pause();
            }
        }
    }
}

#[derive(Clone, Copy)]
enum KeyKind {
    Normal,
    PostQuantum,
}

fn make_age_key_flow(config: &AppConfig, kind: KeyKind) {
    clear_screen();
    print_banner();

    if !deps::age_keygen_exists() {
        println!("age-keygen was not found.");
        println!();
        println!("Install age first:");
        println!("  sudo pacman -S age");
        pause();
        return;
    }

    let key_dir = active_key_dir(config);

    if let Err(err) = fs::create_dir_all(&key_dir) {
        println!("Could not create key folder:");
        println!("  {}", key_dir.display());
        println!("Error: {err}");
        pause();
        return;
    }

    let (private_name, public_name) = match kind {
        KeyKind::Normal => ("morsteg-age-key.txt", "morsteg-age-public-key.txt"),
        KeyKind::PostQuantum => ("morsteg-pq-key.txt", "morsteg-pq-public-key.txt"),
    };

    let key_path = key_dir.join(private_name);
    let public_key_path = key_dir.join(public_name);

    println!("MorSteg will make these files:\n");
    println!("PRIVATE KEY FILE, DO NOT SHARE:");
    println!("  {}", key_path.display());
    println!();
    println!("PUBLIC KEY FILE, SAFE TO SHARE:");
    println!("  {}", public_key_path.display());
    println!();

    match kind {
        KeyKind::Normal => {
            println!("This will make a normal age key.");
            println!("Use this if you just want simple key-based encryption.");
        }
        KeyKind::PostQuantum => {
            println!("This will try to make a post-quantum-capable age key.");
            println!("This only works if your installed age supports:");
            println!("  age-keygen -pq");
        }
    }

    println!();
    println!("Important:");
    println!("  Send people the PUBLIC KEY FILE or the public key text.");
    println!("  Never send the PRIVATE KEY FILE.");
    println!();

    if key_path.exists() || public_key_path.exists() {
        println!("One or both key files already exist.");
        println!();

        if key_path.exists() {
            println!("Existing private key:");
            println!("  {}", key_path.display());
        }

        if public_key_path.exists() {
            println!("Existing public key:");
            println!("  {}", public_key_path.display());
        }

        println!();

        if !confirm("Overwrite the existing key files?") {
            println!("Cancelled.");
            pause();
            return;
        }
    } else if !confirm("Create these key files now?") {
        println!("Cancelled.");
        pause();
        return;
    }

    let mut command = Command::new("age-keygen");

    if matches!(kind, KeyKind::PostQuantum) {
        command.arg("-pq");
    }

    let status = command
        .arg("-o")
        .arg(&key_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(status) if status.success() => {
            lock_down_key_permissions(&key_path);

            println!();
            println!("Private key created:");
            println!("  {}", key_path.display());

            match read_public_key_from_identity_file(&key_path) {
                Some(public_key) => {
                    match write_public_key_file(&public_key_path, &public_key) {
                        Ok(()) => {
                            println!();
                            println!("Public key file created:");
                            println!("  {}", public_key_path.display());
                            println!();
                            println!("PUBLIC KEY TEXT:");
                            println!("  {public_key}");
                            println!();
                            println!("Lazy sharing rule:");
                            println!("  Send people this public key file:");
                            println!("    {}", public_key_path.display());
                            println!();
                            println!("  Do NOT send this private key file:");
                            println!("    {}", key_path.display());
                        }
                        Err(err) => {
                            println!();
                            println!("The key was created, but I could not write the separate public key file.");
                            println!("Error: {err}");
                            println!();
                            println!("PUBLIC KEY TEXT:");
                            println!("  {public_key}");
                        }
                    }
                }
                None => {
                    println!();
                    println!("I could not find the public key line automatically.");
                    println!("Open the private key file and look for a line like:");
                    println!("  # public key: age1...");
                    println!();
                    println!("Then copy only the public key part, not the private key file.");
                }
            }

            println!();
            println!("Private key permissions were locked down to 600 when possible.");
        }
        Ok(status) => {
            println!();
            println!("age-keygen failed with status: {status}");

            if matches!(kind, KeyKind::PostQuantum) {
                println!();
                println!("Your installed age may not support post-quantum keys yet.");
                println!("Try option [2] to make a normal age key.");
            }
        }
        Err(err) => {
            println!();
            println!("Could not run age-keygen.");
            println!("Error: {err}");
        }
    }

    pause();
}

fn show_public_key_from_file_flow() {
    clear_screen();
    print_banner();

    println!("Show the public key from one of my key files.\n");
    println!("Choose your private age key file.");
    println!("MorSteg will print the public key and offer to save a separate share-safe public key file.");
    println!();
    println!("Tip: You can drag the key file into this terminal and press Enter.");
    println!();

    let key_file = prompt_existing_file("Private age key file");

    println!();

    match read_public_key_from_identity_file(&key_file) {
        Some(public_key) => {
            println!("PUBLIC KEY TEXT:");
            println!("  {public_key}");
            println!();
            println!("This public key is safe to give to someone else.");
            println!("They can use it to hide a file for you.");
            println!();

            if confirm("Save this as a separate public key file?") {
                let public_path = default_public_key_path_for_private_key(&key_file);

                match write_public_key_file(&public_path, &public_key) {
                    Ok(()) => {
                        println!();
                        println!("Public key file saved:");
                        println!("  {}", public_path.display());
                    }
                    Err(err) => {
                        println!();
                        println!("Could not save public key file.");
                        println!("Error: {err}");
                    }
                }
            }
        }
        None => {
            println!("I could not find a public key line in that file.");
            println!();
            println!("Look for a line like:");
            println!("  # public key: age1...");
            println!("or:");
            println!("  # public key: age1pq1...");
        }
    }

    println!();
    println!("Reminder: do not share the private key file itself.");
    pause();
}

fn explain_age_keys_sleepy_screen() {
    clear_screen();
    print_banner();

    println!("Age keys, sleepy version.\n");
    println!("Think of an age key like a mailbox.\n");

    println!("PUBLIC AGE KEY");
    println!("  This is like a mail slot.");
    println!("  You can give it to people.");
    println!("  They can use it to lock a secret file for you.");
    println!("  They cannot use it to open your secrets.");
    println!();

    println!("PRIVATE AGE KEY FILE");
    println!("  This is like the real mailbox key.");
    println!("  Keep it secret.");
    println!("  This is what opens files made for your public key.");
    println!();

    println!("The lazy workflow:");
    println!("  1. Use Age key helper -> Make a post-quantum age key for me.");
    println!("  2. MorSteg creates a private key file and a public key file.");
    println!("  3. Give someone the public key file.");
    println!("  4. They use MorSteg option [3] to hide a file for you.");
    println!("  5. You use MorSteg option [4] to open it with your private key file.");
    println!();

    println!("Short rule:");
    println!("  Share the public key file.");
    println!("  Never share the private key file.");

    pause();
}

fn show_key_storage_advice_screen(config: &AppConfig) {
    clear_screen();
    print_banner();

    println!("Where age key files should live.\n");

    let key_dir = active_key_dir(config);

    println!("MorSteg is currently saving age keys here:");
    println!("  {}", key_dir.display());
    println!();

    println!("By default, that is usually:");
    println!("  ~/.config/morsteg/keys/");
    println!();

    println!("For lazy sharing, MorSteg creates two files:");
    println!("  morsteg-pq-key.txt         private, do not share");
    println!("  morsteg-pq-public-key.txt  public, safe to share");
    println!();

    println!("To change this folder:");
    println!("  Use main menu option [7].");
    println!();

    println!("Important:");
    println!("  Do not email your private key file.");
    println!("  Do not upload your private key file.");
    println!("  Do not send your private key file in chat.");
    println!("  If someone gets it, they may be able to open files made for that key.");

    pause();
}

fn read_public_key_from_identity_file(path: &Path) -> Option<String> {
    let contents = fs::read_to_string(path).ok()?;

    for line in contents.lines() {
        let trimmed = line.trim();

        if let Some(public_key) = trimmed.strip_prefix("# public key:") {
            let public_key = public_key.trim();

            if !public_key.is_empty() {
                return Some(public_key.to_string());
            }
        }
    }

    None
}

fn write_public_key_file(path: &Path, public_key: &str) -> io::Result<()> {
    let contents = format!(
        "# MorSteg public age key\n\
         # Safe to share. This cannot decrypt files.\n\
         # Give this to someone who wants to hide a file for you.\n\
         {public_key}\n"
    );

    fs::write(path, contents)
}

fn default_public_key_path_for_private_key(private_key: &Path) -> PathBuf {
    let parent = private_key.parent().unwrap_or_else(|| Path::new("."));
    let stem = private_key
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("morsteg-age");

    parent.join(format!("{stem}-public-key.txt"))
}

fn lock_down_key_permissions(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if let Ok(metadata) = fs::metadata(path) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);

            if let Err(err) = fs::set_permissions(path, permissions) {
                println!("Warning: could not set key permissions to 600.");
                println!("Error: {err}");
            }
        }
    }
}

fn status_success(result: io::Result<std::process::ExitStatus>, action: &str) -> bool {
    match result {
        Ok(status) if status.success() => true,
        Ok(status) => {
            println!("\n{action} exited with a non-zero status: {status}");
            false
        }
        Err(err) => {
            println!("\nFailed to run process for {action}.");
            println!("Error: {err}");
            false
        }
    }
}

fn show_status(action: &str, result: io::Result<std::process::ExitStatus>) {
    match result {
        Ok(status) if status.success() => println!("\n{action} completed successfully."),
        Ok(status) => println!("\n{action} exited with a non-zero status: {status}"),
        Err(err) => println!("\nFailed to run process for {action}.\nError: {err}"),
    }
}
