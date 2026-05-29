use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

#[derive(Debug, Clone)]
pub enum EncryptionMode {
    AgePassphrase,
    AgeRecipient { recipient: String },
}

#[derive(Debug, Clone)]
pub enum DecryptionMode {
    AgePassphrase,
    AgeIdentityFile { identity_file: std::path::PathBuf },
}

pub fn encrypt_payload(
    mode: &EncryptionMode,
    secret_file: &Path,
    encrypted_payload: &Path,
) -> io::Result<ExitStatus> {
    let mut command = Command::new("age");

    match mode {
        EncryptionMode::AgePassphrase => {
            command.arg("-p");
        }
        EncryptionMode::AgeRecipient { recipient } => {
            command.arg("-r").arg(recipient);
        }
    }

    command
        .arg("-o")
        .arg(encrypted_payload)
        .arg(secret_file)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

pub fn decrypt_payload(
    mode: &DecryptionMode,
    encrypted_payload: &Path,
    final_output: &Path,
) -> io::Result<ExitStatus> {
    let mut command = Command::new("age");

    command.arg("-d");

    match mode {
        DecryptionMode::AgePassphrase => {}
        DecryptionMode::AgeIdentityFile { identity_file } => {
            command.arg("-i").arg(identity_file);
        }
    }

    command
        .arg("-o")
        .arg(final_output)
        .arg(encrypted_payload)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

pub fn print_mode_note(mode: &EncryptionMode) {
    match mode {
        EncryptionMode::AgePassphrase => {
            println!("Mode: age passphrase encryption.");
            println!("Mor-SteG is not reading or storing this password.");
        }
        EncryptionMode::AgeRecipient { recipient } => {
            println!("Mode: age recipient encryption.");
            if recipient.starts_with("age1pq1") {
                println!("Recipient looks like an age post-quantum recipient.");
                println!("Mor-SteG still labels this as PQ-capable, not magically quantum-proof.");
            } else {
                println!("Recipient does not look like an age post-quantum recipient.");
                println!("This may still be a normal age recipient, but Mor-SteG will not call it PQ.");
            }
        }
    }
}
