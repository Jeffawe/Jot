use x_win::get_active_window;

use crate::types::{SimplifiedWindowInfo, SimpleProcessInfo};

pub fn get_context() -> Result<SimplifiedWindowInfo, Box<dyn std::error::Error>> {
    // 1. Call the original function to get the full data
    let active_window = get_active_window()?;

    // 2. Map the active_window data to your new structure

    // First, map the nested ProcessInfo
    let simple_info = SimpleProcessInfo {
        process_id: active_window.info.process_id,
        path: active_window.info.path,
        name: active_window.info.name,
        exec_name: active_window.info.exec_name,
    };

    // Second, map the main WindowInfo
    let simplified_info = SimplifiedWindowInfo {
        id: active_window.id,
        os: active_window.os,
        title: active_window.title,
        info: simple_info,
    };

    Ok(simplified_info)
}

pub fn print_context() {
    match get_context() {
        Ok(simplified_window) => {
            // This will print ONLY the fields in your SimplifiedWindowInfo
            println!("active window: {:#?}", simplified_window);
        }
        Err(e) => {
            println!("error occurred while getting the active window: {}", e);
        }
    }
}