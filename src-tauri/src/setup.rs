use jotx::setup::{setup_hooks, uninstall, full_setup};
use std::{fs, path::PathBuf, process::Command};

pub fn is_setup_complete() -> bool {
    let jotx_dir = PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".jotx");
    let setup_marker = jotx_dir.join(".setup_complete");

    // Check if setup marker exists and ollama is installed
    setup_marker.exists()
        && Command::new("ollama")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
}

pub fn mark_setup_complete() -> Result<(), Box<dyn std::error::Error>> {
    let jotx_dir = PathBuf::from(std::env::var("HOME")?).join(".jotx");
    fs::create_dir_all(&jotx_dir)?;
    fs::write(jotx_dir.join(".setup_complete"), "")?;
    Ok(())
}

// Simple version for GUI
pub fn setup_jotx(force: bool) -> Result<(), Box<dyn std::error::Error>> {
    if !is_setup_complete() {
        match full_setup(force, true) {
            Ok(_) => {
                if let Err(e) = mark_setup_complete() {
                    return Err(e);  
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    } else {
        Ok(())
    }
}

#[tauri::command]
pub fn check_setup_status() -> bool {
    is_setup_complete()
}

#[tauri::command]
pub async fn run_setup() -> Result<String, String> {
    setup_jotx(true) // force=true since user confirmed in GUI
        .map_err(|e| format!("Setup failed: {}", e))?;

    Ok("Setup completed successfully!".to_string())
}

#[tauri::command]
pub fn setup_hooks_gui() -> Result<(), String> {
    setup_hooks()
        .map_err(|e| format!("Setup failed: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn uninstall_jotx() -> Result<(), String> {
    uninstall(true)
        .map_err(|e| format!("Uninstall failed: {}", e))?;
    Ok(())
}
