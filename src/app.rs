use crate::crypto::{DecryptionMode, EncryptionMode};
use crate::flows::{age_keys, checks, cloud_safe, inspect, password, save_locations};
use crate::paths::{active_cloud_dir, active_key_dir, AppConfig};
use crate::ui::{BUILD_MARKER, clear_screen, pause, print_banner, prompt};

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

        print_main_menu();

        match prompt("Choose").trim() {
            "1" => password::embed_flow(&config, EncryptionMode::AgePassphrase),
            "2" => password::extract_flow(&config, DecryptionMode::AgePassphrase),
            "3" => password::embed_with_recipient_flow(&config),
            "4" => password::extract_with_identity_flow(&config),
            "5" => age_keys::age_key_helper_flow(&config),
            "6" => save_locations::choose_output_folder_flow(&mut config),
            "7" => save_locations::choose_key_folder_flow(&mut config),
            "8" => save_locations::choose_cloud_folder_flow(&mut config),
            "9" => cloud_safe::cloud_safe_one_file_flow(&config),
            "10" => cloud_safe::cloud_safe_folder_flow(&config),
            "11" => inspect::inspect_flow(),
            "12" => checks::check_deps_flow(),
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

fn print_main_menu() {
    println!();
    println!("What do you want to do?\n");

    println!("{:<47}{}", "Easy password mode", "Age key mode");
    println!("{:<47}{}", "[1] Hide a file using a password", "[3] Hide for someone using age key");
    println!("{:<47}{}", "[2] Open a hidden file with password", "[4] Open using my private age key");
    println!("{:<47}{}", "", "[5] Age key helper");
    println!();

    println!("{:<47}{}", "Save locations", "Cloud safety");
    println!("{:<47}{}", "[6] Steghide/output save folder", "[9] Make one file cloud-safe");
    println!("{:<47}{}", "[7] Age key save folder", "[10] Make a folder cloud-safe");
    println!("{:<47}{}", "[8] Cloud-safe package folder", "");
    println!();

    println!("Tools");
    println!("[11] Check a file for hidden steghide info");
    println!("[12] Check if MorSteg has the tools it needs");
    println!("[13] Quit\n");

    println!("Not sure? Use [1] to hide a file and [2] to open it later.\n");
}
