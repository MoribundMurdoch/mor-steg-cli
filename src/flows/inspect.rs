use crate::deps;
use crate::paths::prompt_existing_file;
use crate::steg;
use crate::ui::{BUILD_MARKER, clear_screen, pause, print_banner};

use super::password::show_status;

pub fn inspect_flow() {
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
