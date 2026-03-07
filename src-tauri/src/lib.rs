mod parsers;
mod commands;
mod survey_parser;
mod cdn;
mod game_data;
mod cdn_commands;

use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::Emitter;

use commands::{start_watching, parse_log};
use cdn_commands::{
    GameDataState,
    init_game_data,
    get_cache_status,
    force_refresh_cdn,
    get_item,
    get_item_by_name,
    search_items,
    get_all_skills,
    get_skill_by_name,
    get_abilities_for_skill,
    get_recipes_for_item,
    get_recipes_using_item,
    get_icon_path,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let game_data_state: GameDataState = Arc::new(RwLock::new(game_data::GameData::empty()));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(game_data_state.clone())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let state = game_data_state.clone();

            // Kick off CDN init in the background — non-blocking.
            // The frontend should listen for a "game-data-ready" event
            // or poll get_cache_status() to know when data is available.
            tauri::async_runtime::spawn(async move {
                match init_game_data(&app_handle, &state).await {
                    Ok(()) => {
                        let data = state.read().await;
                        eprintln!(
                            "Game data ready: v{} — {} items, {} skills, {} recipes, {} npcs, {} effects, {} areas",
                            data.version,
                            data.items.len(),
                            data.skills.len(),
                            data.recipes.len(),
                            data.npcs.len(),
                            data.effects.len(),
                            data.areas.len(),
                        );
                        // Notify the frontend that data is loaded
                        app_handle.emit("game-data-ready", data.version).ok();
                    }
                    Err(e) => {
                        eprintln!("Game data init failed: {e}");
                        app_handle.emit("game-data-error", e).ok();
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Log watching
            start_watching,
            parse_log,
            // CDN management
            get_cache_status,
            force_refresh_cdn,
            // Item queries
            get_item,
            get_item_by_name,
            search_items,
            // Skill queries
            get_all_skills,
            get_skill_by_name,
            // Ability queries
            get_abilities_for_skill,
            // Recipe queries
            get_recipes_for_item,
            get_recipes_using_item,
            // Icons
            get_icon_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}