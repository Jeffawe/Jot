mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::ask_command,
            commands::search_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}