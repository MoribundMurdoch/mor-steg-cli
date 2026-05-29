use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::deps;
use crate::paths::{
    active_key_dir, explain_key_folder, prompt_existing_file, AppConfig,
};
use crate::ui::{clear_screen, confirm, pause, print_banner, prompt};

#[derive(Clone, Copy)]
enum KeyKind {
    Normal,
    PostQuantum,
}

pub fn age_key_helper_flow(config: &AppConfig) {
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
