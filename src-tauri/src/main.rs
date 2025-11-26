mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::ask_command,
            commands::search_command,
            commands::get_settings,
            commands::save_settings,
            commands::get_privacy_config,
            commands::save_privacy_config,
            commands::check_ollama_status,
            commands::download_model,
            commands::remove_model,
            commands::install_ollama,
            commands::start_ollama,
            commands::get_all_paths
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}