use crate::db::GLOBAL_DB;
use crate::types::{EntryType, QueryParams};

pub fn ask(query: &str) {
    if query.is_empty() {
        // No parentheses!
        println!("No query provided.");
        return;
    }

    if query.to_lowercase().contains("get last clip history") {
        if let Ok(monitor) = GLOBAL_DB.lock() {
            // You need to add a print_history method to ClipMon
            // Or access history directly:
            let queries = QueryParams {
                entry_type: Some(EntryType::Clipboard),
                ..Default::default()
            };

            if let Ok(entries) = monitor.query_entries(queries) {
                println!("Clipboard History ({} entries):", entries.len());

                for entry in entries {
                    println!("  [{}] - {}", entry.timestamp, entry.content);
                }
            }
        } else {
            println!("Failed to access clipboard history.");
        }
    }

    if query.to_lowercase().contains("get last shell history") {
        if let Ok(monitor) = GLOBAL_DB.lock() {
            // You need to add a print_history method to ClipMon
            // Or access history directly:
            let queries = QueryParams {
                entry_type: Some(EntryType::Shell),
                ..Default::default()
            };

            if let Ok(entries) = monitor.query_entries(queries) {
                println!("Shell History ({} entries):", entries.len());

                for entry in entries {
                    println!("  [{}] - {}", entry.timestamp, entry.content);
                }
            }
        } else {
            println!("Failed to access clipboard history.");
        }
    }

    println!("Your query is: {}", query);
}
