use crate::settings::GLOBAL_SETTINGS;
use crate::config::GLOBAL_CONFIG;
use colored::*;
use std::{
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
        println!(
            "4. Clipboard Case Sensitive:   {}",
            if settings.clipboard_case_sensitive {
                "✅ ON"
            } else {
                "❌ OFF"
            }
        );
        println!(
            "5. Shell Case Sensitive:   {}",
            if settings.shell_case_sensitive {
                "✅ ON"
            } else {
                "❌ OFF"
            }
        );
        println!("6. Clipboard History Size: {}", settings.clipboard_limit);
        println!("7. Shell History Size: {}", settings.shell_limit);
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
            "4" => GLOBAL_SETTINGS.lock().unwrap().toggle_clipboard_case_sensitive(),
            "5" => GLOBAL_SETTINGS.lock().unwrap().toggle_shell_case_sensitive(),
            "6" => {
                print!("Enter new limit: ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                GLOBAL_SETTINGS
                    .lock()
                    .unwrap()
                    .set_clipboard_limit(input.trim().parse().unwrap());
            }
            "7" => {
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

fn edit_string_list(
    title: &str, 
    list: &mut Vec<String>
) -> Result<(), Box<dyn std::error::Error>> {
    
    loop {
        // Clear screen (optional)
        print!("\x1B[2J\x1B[1;1H");

        println!("{}", format!("╔════════════════════════════════════════╗").cyan());
        println!("{}", format!("║   Editing: {}   ║", title).cyan());
        println!("{}", format!("╚════════════════════════════════════════╝").cyan());
        println!();

        if list.is_empty() {
            println!("(List is currently empty)");
        } else {
            println!("Current Exclusions:");
            for (i, item) in list.iter().enumerate() {
                println!("{}. {}", (i + 1).to_string().yellow(), item);
            }
        }
        
        println!("\n═══════════════════════════════════");
        println!("A. Add new item");
        println!("R. Remove item (by number)");
        println!("0. Back to Privacy Menu");
        println!("═══════════════════════════════════");

        print!("Enter action (A/R/0): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim().to_uppercase();

        match choice.as_str() {
            "A" => {
                print!("Enter string to add: ");
                io::stdout().flush()?;
                let mut new_item = String::new();
                io::stdin().read_line(&mut new_item)?;
                let trimmed_item = new_item.trim().to_string();
                if !trimmed_item.is_empty() {
                    list.push(trimmed_item);
                }
            }
            "R" => {
                if list.is_empty() {
                    println!("List is empty, nothing to remove. Press Enter...");
                    let mut _dummy = String::new();
                    io::stdin().read_line(&mut _dummy)?;
                    continue;
                }
                print!("Enter number of item to remove (1-{}): ", list.len());
                io::stdout().flush()?;
                let mut index_input = String::new();
                io::stdin().read_line(&mut index_input)?;
                
                if let Ok(index) = index_input.trim().parse::<usize>() {
                    let zero_based_index = index.saturating_sub(1);
                    if zero_based_index < list.len() {
                        let removed = list.remove(zero_based_index);
                        println!("Removed: {}. Press Enter...", removed);
                    } else {
                        println!("Invalid number. Press Enter...");
                    }
                } else {
                    println!("Invalid input. Press Enter...");
                }
                let mut _dummy = String::new();
                io::stdin().read_line(&mut _dummy)?;
            }
            "0" => return Ok(()),
            _ => {
                println!("Invalid option. Press Enter to continue...");
                let mut _dummy = String::new();
                io::stdin().read_line(&mut _dummy)?;
            }
        }
    }
}


/// Command for editing all privacy configurations.
pub fn show_privacy_settings() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // 1. ACQUIRE READ LOCK and CLONE DATA for display and local modification
        let mut current_privacy = {
            if let Ok(config_guard) = GLOBAL_CONFIG.read() {
                // Must clone to take ownership of the data outside the lock
                config_guard.privacy.clone() 
            } else {
                eprintln!("Failed to read global config. Exiting privacy settings.");
                return Ok(());
            }
        }; 
        // Read lock is automatically released here

        // Clear screen (optional)
        print!("\x1B[2J\x1B[1;1H");

        println!("{}", "╔════════════════════════════════════════╗".cyan());
        println!("{}", "║        JotX Privacy Settings           ║".cyan());
        println!("{}", "╚════════════════════════════════════════╝".cyan());
        println!();

        println!("═══════════════════════════════════");
        println!(
            "1. Contains String Exclusions ({})", 
            current_privacy.excludes_contains_string.len().to_string().yellow()
        );
        println!(
            "2. Starts With Exclusions ({})", 
            current_privacy.excludes_starts_with_string.len().to_string().yellow()
        );
        println!(
            "3. Ends With Exclusions ({})", 
            current_privacy.excludes_ends_with_string.len().to_string().yellow()
        );
        println!(
            "4. Regex Exclusions ({})", 
            current_privacy.excludes_regex.len().to_string().yellow()
        );
        println!(
            "5. Folder Exclusions ({})", 
            current_privacy.exclude_folders.len().to_string().yellow()
        );
        println!("═══════════════════════════════════");
        println!("0. Save and Exit");
        println!();

        // Get user input
        print!("Enter number to edit (0 to exit): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        // This is a mutable reference to the vector we are editing
        let mut list_to_edit: Option<(&str, &mut Vec<String>)> = None;

        match input.trim() {
            "1" => list_to_edit = Some((
                "Contains String Exclusions", 
                &mut current_privacy.excludes_contains_string
            )),
            "2" => list_to_edit = Some((
                "Starts With Exclusions", 
                &mut current_privacy.excludes_starts_with_string
            )),
            "3" => list_to_edit = Some((
                "Ends With Exclusions", 
                &mut current_privacy.excludes_ends_with_string
            )),
            "4" => list_to_edit = Some((
                "Regex Exclusions", 
                &mut current_privacy.excludes_regex
            )),
            "5" => list_to_edit = Some((
                "Folder Exclusions", 
                &mut current_privacy.exclude_folders
            )),
            "0" => {
                // Save the modified config before breaking
                let mut config_guard = GLOBAL_CONFIG.write().unwrap();
                config_guard.update_privacy_settings(current_privacy)?;
                drop(config_guard); // Release write lock
                break;
            }
            _ => {
                println!("Invalid option. Press Enter to continue...");
                let mut _dummy = String::new();
                io::stdin().read_line(&mut _dummy)?;
            }
        }
        
        // If a list was selected, call the helper editor
        if let Some((title, list)) = list_to_edit {
            edit_string_list(title, list)?;
            // After returning from the editor, the loop will re-display the main menu
        }
    }

    println!("Privacy settings saved!");
    Ok(())
}