use crate::settings::GLOBAL_SETTINGS;
use colored::*;
use std::process::Command;
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

pub fn show_settings() {
    loop {
        // Clear screen (optional)
        print!("\x1B[2J\x1B[1;1H");

        println!("{}", "╔════════════════════════════════════════╗".cyan());
        println!("{}", "║        JotX Settings.                  ║".cyan());
        println!("{}", "╚════════════════════════════════════════╝".cyan());
        println!();

        println!("═══════════════════════════════════");
        let settings = GLOBAL_SETTINGS.lock().unwrap();
        println!(
            "1. Capture Clipboard: {}",
            if settings.capture_clipboard {
                "✅ ON"
            } else {
                "❌ OFF"
            }
        );
        println!(
            "2. Capture Shell:     {}",
            if settings.capture_shell {
                "✅ ON"
            } else {
                "❌ OFF"
            }
        );
        println!(
            "3. Use Shell History With Files:   {}",
            if settings.capture_shell_history_with_files {
                "✅ ON"
            } else {
                "❌ OFF"
            }
        );
        println!("4. Clipboard History Size: {}", settings.clipboard_limit);
        println!("5. Shell History Size: {}", settings.shell_limit);
        println!("═══════════════════════════════════");
        println!("0. Exit");
        println!();
        drop(settings); // Release lock before reading input

        // Get user input
        print!("Enter number to toggle (0 to exit): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => GLOBAL_SETTINGS.lock().unwrap().toggle_clipboard(),
            "2" => GLOBAL_SETTINGS.lock().unwrap().toggle_shell(),
            "3" => GLOBAL_SETTINGS.lock().unwrap().toggle_shell_history(),
            "4" => {
                print!("Enter new limit: ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                GLOBAL_SETTINGS
                    .lock()
                    .unwrap()
                    .set_clipboard_limit(input.trim().parse().unwrap());
            }
            "5" => {
                print!("Enter new limit: ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                GLOBAL_SETTINGS
                    .lock()
                    .unwrap()
                    .set_shell_limit(input.trim().parse().unwrap());
            }
            "0" => break,
            _ => {
                println!("Invalid option. Press Enter to continue...");
                let mut _dummy = String::new();
                io::stdin().read_line(&mut _dummy).unwrap();
            }
        }
    }

    println!("Settings saved!");
}

pub fn get_working_directory() -> String {
    let pwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| String::from(""));
    pwd
}

pub fn get_plugin_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    let plugin_dir = PathBuf::from(home).join(".jotx").join("plugins");
    plugin_dir
}

fn load_repo_path() -> PathBuf {
    let path_file = dirs::home_dir().unwrap().join(".jotx").join("path");

    let content = fs::read_to_string(&path_file).expect("Failed to read ~/.jotx/path");

    PathBuf::from(content.trim())
}

pub fn run_make(target: &str) {
    let repo = load_repo_path();

    let status = Command::new("make")
        .arg(target)
        .current_dir(&repo)
        .status()
        .expect("failed to run make command");

    if !status.success() {
        eprintln!("❌ make {} failed", target);
    }
}
