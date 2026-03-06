mod parsers;
mod commands;

use commands::{start_watching, parse_log};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![start_watching, parse_log])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}