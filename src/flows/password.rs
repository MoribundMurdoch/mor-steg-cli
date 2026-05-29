use std::io;

use crate::crypto::{self, DecryptionMode, EncryptionMode};
use crate::deps;
use crate::paths::{
    self, explain_output_folder, prompt_existing_file, prompt_output_file, to_absolute_path,
    AppConfig,
};
use crate::steg;
use crate::ui::{BUILD_MARKER, clear_screen, confirm, pause, print_banner, prompt_nonempty};

pub fn embed_with_recipient_flow(config: &AppConfig) {
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

pub fn embed_flow(config: &AppConfig, encryption_mode: EncryptionMode) {
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

pub fn extract_with_identity_flow(config: &AppConfig) {
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

pub fn extract_flow(config: &AppConfig, decryption_mode: DecryptionMode) {
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

pub fn show_status(action: &str, result: io::Result<std::process::ExitStatus>) {
    match result {
        Ok(status) if status.success() => println!("\n{action} completed successfully."),
        Ok(status) => println!("\n{action} exited with a non-zero status: {status}"),
        Err(err) => println!("\nFailed to run process for {action}.\nError: {err}"),
    }
}
