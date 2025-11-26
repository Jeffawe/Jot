use std::{vec};

use crate::{
    config::{GLOBAL_CONFIG, PrivacyConfig, get_config_path}, db::get_db_path, settings::{GLOBAL_SETTINGS, Settings}, types::{OllamaStatus, PathInfo}
};
use crate::llm::GLOBAL_LLM;

pub fn load_settings() -> Result<Settings, String> {
    match GLOBAL_SETTINGS.lock() {
        Ok(settings) => {
            let s = Settings {
                capture_clipboard: settings.capture_clipboard,
                capture_shell: settings.capture_shell,
                capture_shell_history_with_files: settings.capture_shell_history_with_files,
                shell_case_sensitive: settings.shell_case_sensitive,
                clipboard_case_sensitive: settings.clipboard_case_sensitive,
                clipboard_limit: settings.clipboard_limit,
                shell_limit: settings.shell_limit,
            };
            Ok(s)
        }
        Err(e) => Err(format!("Failed to load settings: {}", e)),
    }
}

pub fn save_settings(updated: &Settings) -> Result<(), String> {
    match GLOBAL_SETTINGS.try_lock() {
        Ok(mut settings) => {
            settings.capture_clipboard = updated.capture_clipboard;
            settings.capture_shell = updated.capture_shell;
            settings.capture_shell_history_with_files = updated.capture_shell_history_with_files;
            settings.shell_case_sensitive = updated.shell_case_sensitive;
            settings.clipboard_case_sensitive = updated.clipboard_case_sensitive;
            settings.clipboard_limit = updated.clipboard_limit;
            settings.shell_limit = updated.shell_limit;

            settings
                .save()
                .map_err(|e| format!("Failed to save settings: {}", e))
        }
        Err(e) => Err(format!("Failed to save settings: {}", e)),
    }
}

pub fn load_privacy_config() -> Result<PrivacyConfig, String> {
    match GLOBAL_CONFIG.read() {
        Ok(config) => Ok(config.privacy.clone()),
        Err(e) => Err(format!("Failed to load privacy config: {}", e)),
    }
}

pub fn save_privacy_config(updated: PrivacyConfig) -> Result<(), String> {
    match GLOBAL_CONFIG.try_write() {
        Ok(mut config) => {
            config.privacy = updated;
            config
                .save()
                .map_err(|e| format!("Failed to save privacy config: {}", e))
        }
        Err(e) => Err(format!("Failed to save privacy config: {}", e)),
    }
}

pub async fn is_ollama_running() -> Result<OllamaStatus, String> {
    match GLOBAL_LLM.try_lock() {
        Ok(llm_manager) => {
            let is_running = llm_manager.is_ollama_running().await;
            let models = llm_manager.get_models();
            let result = OllamaStatus {
                installed: is_running,
                running: is_running,
                models: models,
            };
            Ok(result)
        }
        Err(_) => {
            return Err("Failed to access LLM manager".to_string());
        }
    }
}

pub fn get_paths() -> Result<Vec<PathInfo>, String> {
    let path1 = PathInfo {
        label: "Config File".to_string(),
        path: get_config_path().to_string_lossy().to_string(),
    };

    let path2 = PathInfo {
        label: "DB File".to_string(),
        path: get_db_path().to_string_lossy().to_string(),
    };

    Ok(vec![path1, path2])
}