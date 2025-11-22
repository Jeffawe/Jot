use std::fs;
use std::path::{Path};

/// Creates a new boilerplate plugin script file in the plugins directory.
pub fn create_new_plugin_script(plugin_dir: &Path, name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("Plugin name cannot be empty.".to_string());
    }

    let file_name = format!("{}.rhai", name);
    let path = plugin_dir.join(&file_name);

    if path.exists() {
        return Err(format!(
            "Plugin '{}' already exists at {:?}",
            file_name, path
        ));
    }

    // --- Boilerplate Content ---
    let content = format!(
        r#"// JOTX PLUGIN: {}
// ------------------------------------------------------------------
// This script implements hooks defined in the Rust Plugin trait.
// Uncomment the functions you want to use.
// 
// Rhai Syntax is very similar to Rust/JS.
// Data Types: objects (maps), arrays, numbers, strings, bool.
// ------------------------------------------------------------------

// Hook: Called when a shell command is captured
// Context: CommandContext (read-only)
// Returns: "continue", "stop", or "skip"
// fn on_command_captured(ctx) {{
//     if ctx.command.contains("secret") {{
//         print("Blocking command capture!");
//         return "stop";
//     }}
//     return "continue";
// }}

// Hook: Called after search results are returned
// Arguments: query (string), results (array of SearchResult)
// Returns: The modified array of SearchResult objects
// fn on_search_after(query, results) {{
//     let filtered = [];
//     for res in results {{
//         if res.similarity > 50.0 {{
//             filtered.push(res);
//         }}
//     }}
//     return filtered;
// }}
// Hook: Called on main daemon loop iteration
// Arguments: DaemonContext (read-only)
// Returns: "continue", "stop", or "skip"
// fn on_daemon_tick(ctx) {{
//     // Perform periodic tasks here
//     return "continue";
// }}
// Hook: Called before LLM is invoked
// Arguments: prompt (string), context (LlmContext)
// Returns: "continue", "stop", or "skip"
// fn on_llm_before(prompt, context) {{
//     return "continue";
// }}
// Hook: Called after LLM returns response
// Arguments: prompt (string), response (string), context (LlmContext)
// Returns: "continue", "stop", or "skip"
// fn on_llm_after(prompt, response, context) {{
//     return "continue";
// }}

// NOTE: Ensure your function names and arguments match the contract!
"#,
        name
    );
    // ---------------------------

    fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(format!(
        "✅ Plugin created successfully: {}",
        path.display()
    ))
}
