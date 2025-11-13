use crate::clipboard::clip_mon::GLOBAL_CLIP_MON;
use crate::shell::shell_mon::GLOBAL_SHELL_MON;

pub fn ask(query: &str) {
    if query.is_empty() {  // No parentheses!
        println!("No query provided.");
        return;
    }
    
    if query.to_lowercase().contains("get last clip history") {
        if let Ok(monitor) = GLOBAL_CLIP_MON.lock() {
            // You need to add a print_history method to ClipMon
            // Or access history directly:
            println!("Clipboard History ({} entries):", monitor.history.len());
            for entry in &monitor.history {
                println!("  [{}] {} - {}", 
                    entry.timestamp, 
                    entry.context.info.name,
                    entry.content
                );
            }
        } else {
            println!("Failed to access clipboard history.");
        }
    }

    if query.to_lowercase().contains("get last shell history") {
        if let Ok(monitor) = GLOBAL_SHELL_MON.lock() {
            // You need to add a print_history method to ClipMon
            // Or access history directly:
            println!("Clipboard History ({} entries):", monitor.history.len());
            for entry in &monitor.history {
                println!("  [{}] {} - {}", 
                    entry.timestamp, 
                    entry.context.info.name,
                    entry.content
                );
            }
        } else {
            println!("Failed to access shell history.");
        }
    }
    
    println!("Your query is: {}", query);
}