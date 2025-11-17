use crate::settings::GLOBAL_SETTINGS;
use std::io::{self, Write};

pub fn print_help() {
    println!("Available commands:");
    println!("  run - Start the clipboard/shell monitor (interactive mode)");
    println!("  help - Show help info");
    println!("  exit - Gracefully stop the running service");
    println!("  status - Show service status");
    println!("  ask - Ask a question");
}


pub fn show_settings() {
    loop {
        // Clear screen (optional)
        print!("\x1B[2J\x1B[1;1H");
        
        // Display current settings
        println!("⚙️  Jotx Settings");
        println!("═══════════════════════════════════");
        
        let settings = GLOBAL_SETTINGS.lock().unwrap();
        println!("1. Capture Clipboard: {}", if settings.capture_clipboard { "✅ ON" } else { "❌ OFF" });
        println!("2. Capture Shell:     {}", if settings.capture_shell { "✅ ON" } else { "❌ OFF" });
        println!("3. Use Shell History With Files:   {}", if settings.capture_shell_history_with_files { "✅ ON" } else { "❌ OFF" });
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
                GLOBAL_SETTINGS.lock().unwrap().set_clipboard_limit(input.trim().parse().unwrap());
            }
            "5" => {
                print!("Enter new limit: ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                GLOBAL_SETTINGS.lock().unwrap().set_shell_limit(input.trim().parse().unwrap());
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