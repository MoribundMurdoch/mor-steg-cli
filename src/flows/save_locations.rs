use std::fs;

use crate::paths::{
    active_cloud_dir, active_key_dir, prompt_folder_to_create, AppConfig,
};
use crate::ui::{clear_screen, pause, print_banner, prompt};

pub fn choose_output_folder_flow(config: &mut AppConfig) {
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

pub fn choose_key_folder_flow(config: &mut AppConfig) {
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

pub fn choose_cloud_folder_flow(config: &mut AppConfig) {
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
