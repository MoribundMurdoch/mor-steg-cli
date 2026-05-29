use crate::deps;
use crate::ui::{BUILD_MARKER, clear_screen, pause, print_banner};

pub fn check_deps_flow() {
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
