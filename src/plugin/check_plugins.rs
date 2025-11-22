use rhai::AST;
use std::{collections::HashMap, fs, path::Path};

// Define all expected function hooks and their required arity (number of arguments)
const EXPECTED_HOOKS: [(&str, usize); 6] = [
    ("on_command_captured", 1), // (context)
    ("on_search_before", 1),    // (query)
    ("on_search_after", 2),     // (query, results)
    ("on_llm_before", 2),       // (prompt, context)
    ("on_llm_after", 3),        // (prompt, response, context)
    ("on_daemon_tick", 1),      // (context)
];

/// Checks the functions exported by a single plugin script.
fn check_single_plugin(path: &Path, engine: &rhai::Engine) -> Result<(), String> {
    let script = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read script: {}", e))?;

    // Compile the script into the AST to check functions
    let ast: AST = engine.compile(&script)
        .map_err(|e| format!("Compilation Error: {}", e))?;

    let mut found_hooks = HashMap::new();

    // 1. Collect all functions defined in the script
    for fn_def in ast.iter_functions() {
        let name = fn_def.name;
        let arity = fn_def.params.len();
        found_hooks.insert(name.to_string(), arity);
    }
    
    println!("\n🔍 Checking script: {}", path.file_name().unwrap_or_default().to_string_lossy());

    // 2. Check if the found functions match the expected contract
    for (expected_name, expected_arity) in EXPECTED_HOOKS.iter() {
        if let Some(actual_arity) = found_hooks.get(*expected_name) {
            if actual_arity == expected_arity {
                println!("  ✅ Found hook: {} (Args: {})", expected_name, expected_arity);
            } else {
                println!("  ❌ Arity Mismatch for {}: Expected {} arguments, found {}", 
                    expected_name, expected_arity, actual_arity);
            }
        }
    }
    
    // 3. (Optional) Check for unused/unknown functions
    for (name, _) in found_hooks.iter() {
        if !EXPECTED_HOOKS.iter().any(|(n, _)| n == name) {
            println!("  ⚠️ Warning: Function '{}' is defined but not a recognized hook.", name);
        }
    }

    Ok(())
}

/// Main function to check all or a specific plugin script.
pub fn check_plugin_functions(plugin_dir: &Path, target_name: Option<&str>) -> Result<(), String> {
    
    // Use the engine creator from your setup, ensure it's thread safe (sync feature)
    let engine = crate::plugin::script_engine::create_engine(); 

    if let Some(name) = target_name {
        // --- jotx plugin --check init ---
        let path = plugin_dir.join(format!("{}.rhai", name));
        if path.exists() {
            check_single_plugin(&path, &engine)?;
        } else {
            println!("No plugin found with the name '{}'. Skipping check.", name);
        }
    } else {
        // --- jotx plugin --check (all) ---
        let mut checked_count = 0;
        
        if let Ok(entries) = fs::read_dir(plugin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "rhai") {
                    check_single_plugin(&path, &engine)?;
                    checked_count += 1;
                }
            }
        }

        if checked_count == 0 {
            println!("No .rhai plugin scripts found in {}.", plugin_dir.display());
        }
    }
    
    Ok(())
}