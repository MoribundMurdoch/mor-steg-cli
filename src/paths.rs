use std::io;
use std::path::{Path, PathBuf};

use crate::ui::{confirm, prompt};

#[derive(Debug, Default)]
pub struct AppConfig {
    pub output_dir: Option<PathBuf>,
    pub key_dir: Option<PathBuf>,
}

pub fn default_key_dir() -> PathBuf {
    if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg_config_home).join("mor-steg").join("keys");
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("mor-steg")
            .join("keys");
    }

    PathBuf::from(".").join("mor-steg-keys")
}

pub fn active_key_dir(config: &AppConfig) -> PathBuf {
    config.key_dir.clone().unwrap_or_else(default_key_dir)
}

pub fn explain_output_folder(config: &AppConfig) {
    match &config.output_dir {
        Some(dir) => {
            println!("Default output folder is set to:");
            println!("  {}", dir.display());
            println!("If you type only a filename, Mor-SteG will save it there.");
            println!("You can still type a full path to save somewhere else.");
        }
        None => {
            if let Ok(cwd) = std::env::current_dir() {
                println!("No default output folder is set.");
                println!("If you type only a filename, it will save in:");
                println!("  {}", cwd.display());
            } else {
                println!("No default output folder is set.");
                println!("If you type only a filename, it will save in the current folder.");
            }
            println!("Use the main menu save-folder option to choose an output folder first.");
        }
    }
}

pub fn explain_key_folder(config: &AppConfig) {
    let dir = active_key_dir(config);

    println!("Age key files will save in:");
    println!("  {}", dir.display());
    println!();
    println!("Private key files stay there.");
    println!("Public key files made by Mor-SteG are safe to share.");
}

pub fn prompt_existing_file(label: &str) -> PathBuf {
    loop {
        let input = prompt(label);

        if input.is_empty() {
            println!("Please enter a file path.");
            continue;
        }

        let path = PathBuf::from(expand_tilde(&input));

        if path.is_file() {
            return path;
        }

        println!("That file does not exist or is not a regular file. Try again.");
    }
}

pub fn prompt_existing_folder(label: &str) -> PathBuf {
    loop {
        let input = prompt(label);

        if input.is_empty() {
            println!("Please enter a folder path.");
            continue;
        }

        let path = PathBuf::from(expand_tilde(&input));

        if path.is_dir() {
            return path;
        }

        println!("That folder does not exist. Try again.");
    }
}

pub fn prompt_folder_to_create(label: &str) -> PathBuf {
    loop {
        let input = prompt(label);

        if input.is_empty() {
            println!("Please enter a folder path.");
            continue;
        }

        let path = PathBuf::from(expand_tilde(&input));

        if path.exists() {
            if path.is_dir() {
                return path;
            }

            println!("That path exists but is not a folder.");
            continue;
        }

        println!("That folder does not exist:");
        println!("  {}", path.display());

        if confirm("Create it?") {
            return path;
        }
    }
}

pub fn prompt_output_file(label: &str, config: &AppConfig) -> PathBuf {
    loop {
        let input = prompt(label);

        if input.is_empty() {
            println!("Please enter an output filename or path.");
            continue;
        }

        let typed_path = PathBuf::from(expand_tilde(&input));
        let path = resolve_output_path(typed_path, config);

        if path.exists() {
            println!("That path already exists:");
            println!("  {}", path.display());

            if confirm("Overwrite?") {
                return path;
            }

            continue;
        }

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                println!("The parent directory does not exist:");
                println!("  {}", parent.display());
                continue;
            }
        }

        return path;
    }
}

pub fn resolve_output_path(typed_path: PathBuf, config: &AppConfig) -> PathBuf {
    if typed_path.is_absolute() {
        return typed_path;
    }

    let has_parent = typed_path
        .parent()
        .map(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(false);

    if has_parent {
        return typed_path;
    }

    if let Some(output_dir) = &config.output_dir {
        return output_dir.join(typed_path);
    }

    typed_path
}

pub fn to_absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else if let Ok(cwd) = std::env::current_dir() {
        cwd.join(path)
    } else {
        path.to_path_buf()
    }
}

pub fn create_private_temp_payload_path() -> io::Result<(tempfile::TempDir, PathBuf)> {
    let temp_workspace = tempfile::Builder::new()
        .prefix("mor-steg-")
        .tempdir()?;

    let temp_payload = temp_workspace.path().join("payload.age");

    Ok((temp_workspace, temp_payload))
}

fn expand_tilde(input: &str) -> String {
    let input = clean_terminal_path(input);

    if input.starts_with('~') {
        let home_env = if cfg!(windows) { "USERPROFILE" } else { "HOME" };

        if let Ok(home) = std::env::var(home_env) {
            if input == "~" {
                return home;
            }

            if let Some(rest) = input.strip_prefix("~/") {
                return format!("{home}/{rest}");
            }

            if let Some(rest) = input.strip_prefix("~\\") {
                return format!("{home}\\{rest}");
            }
        }
    }

    input
}

fn clean_terminal_path(input: &str) -> String {
    let mut trimmed = input.trim();

    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        trimmed = &trimmed[1..trimmed.len() - 1];
    }

    trimmed.replace("\\ ", " ")
}
