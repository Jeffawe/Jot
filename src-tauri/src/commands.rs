use jotx::ask::{ask_gui, search_gui};
use jotx::types::GUISearchResult;

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