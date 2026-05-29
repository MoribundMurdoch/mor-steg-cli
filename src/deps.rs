use std::process::{Command, Stdio};

pub fn steghide_exists() -> bool {
    command_exists("steghide", "--version")
}

pub fn age_exists() -> bool {
    command_exists("age", "--version")
}

pub fn age_keygen_exists() -> bool {
    command_exists("age-keygen", "--version")
}

pub fn core_dependencies_exist() -> bool {
    steghide_exists() && age_exists()
}

pub fn print_deps_missing() {
    println!("Required system tools are missing.\n");
    println!("Arch Linux:");
    println!("  sudo pacman -S age");
    println!("  paru -S steghide");
    println!("  yay -S steghide");
    println!();
    println!("Debian/Ubuntu:");
    println!("  sudo apt install steghide age");
}

fn command_exists(binary: &str, version_arg: &str) -> bool {
    Command::new(binary)
        .arg(version_arg)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_or(false, |s| s.success())
}
