use jotx::ask::{ask_gui, search_gui};
use jotx::types::{GUISearchResult, PathInfo};
use jotx::utils::{load_settings, is_ollama_running};

#[tauri::command]
pub async fn ask_command(
    query: String,
    directory: String
) -> Result<Vec<GUISearchResult>, String> {
    match ask_gui(&query, &directory).await {
        Ok(response) => Ok(response),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn search_command(
    query: String,
    directory: String
) -> Result<Vec<GUISearchResult>, String> {
    match search_gui(&query, &directory) {
        Ok(results) => Ok(results),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn get_settings() -> Result<jotx::settings::Settings, String> {
    load_settings()
}

#[tauri::command]
pub fn save_settings(updated: jotx::settings::Settings) -> Result<(), String> {
    jotx::utils::save_settings(&updated)
}

#[tauri::command]
pub fn get_privacy_config() -> Result<jotx::config::PrivacyConfig, String> {
    jotx::utils::load_privacy_config()
}

#[tauri::command]
pub fn save_privacy_config(updated: jotx::config::PrivacyConfig) -> Result<(), String> {
    jotx::utils::save_privacy_config(updated)
}

#[tauri::command]
pub async fn check_ollama_status() -> Result<jotx::types::OllamaStatus, String> {
    match is_ollama_running().await {
        Ok(status) => Ok(status),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn remove_model(model: String) -> Result<(), String> {
    match jotx::llm::remove_model_with_string(&model) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn download_model(model: String) -> Result<(), String> {
    match jotx::llm::download_model_with_string(&model) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn install_ollama() -> Result<(), String> {
    match jotx::llm::install_ollama() {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn start_ollama() -> Result<(), String> {
    match jotx::llm::start_ollama_service() {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn get_all_paths() -> Result<Vec<PathInfo>, String> {
    jotx::utils::get_paths()
}

