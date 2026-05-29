use std::io::{self, Write};

pub const BUILD_MARKER: &str =
    "MOR-STEG WIZARD BUILD 2026-05-29 (Split + PQ-Capable Edition)";

pub fn print_banner() {
    println!("╔════════════════════════════════════════════════╗");
    println!("║                    MOR-STEG                    ║");
    println!("║       Friendly Steganography Helper            ║");
    println!("╚════════════════════════════════════════════════╝");
    println!();
}

pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    let _ = io::stdout().flush();
}

pub fn prompt(label: &str) -> String {
    print!("{label}: ");
    let _ = io::stdout().flush();

    let mut input = String::new();

    match io::stdin().read_line(&mut input) {
        Ok(_) => input.trim().to_string(),
        Err(_) => String::new(),
    }
}

pub fn prompt_nonempty(label: &str) -> String {
    loop {
        let value = prompt(label);

        if !value.trim().is_empty() {
            return value;
        }

        println!("Please enter a value.");
    }
}

pub fn confirm(label: &str) -> bool {
    loop {
        let answer = prompt(&format!("{label} [y/n]"));

        match answer.to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("Please type y or n."),
        }
    }
}

pub fn pause() {
    println!("\nPress Enter to return to the menu...");
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}
