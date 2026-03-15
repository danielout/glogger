mod parsers;
mod commands;
mod survey_parser;
mod cdn;
mod game_data;
mod cdn_commands;
mod db;
mod settings;
mod chat_parser;
mod chat_commands;
mod log_watchers;
mod coordinator;
mod setup_commands;
mod watch_rules;

use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use tauri::{Emitter, Manager};

use commands::{start_watching, parse_log};
use cdn_commands::{
    GameDataState,
    init_game_data,
    get_cache_status,
    force_refresh_cdn,
    get_item,
    get_item_by_name,
    search_items,
    get_equip_slots,
    get_all_skills,
    get_skill_by_name,
    get_abilities_for_skill,
    get_recipe_by_name,
    get_recipes_for_item,
    get_recipes_using_item,
    search_recipes,
    get_recipes_for_skill,
    get_items_batch,
    get_all_quests,
    search_quests,
    get_quest_by_key,
    get_all_npcs,
    search_npcs,
    get_npcs_in_area,
    search_effects,
    get_effect,
    get_all_player_titles,
    search_player_titles,
    get_icon_path,
    get_ability_sources,
    get_item_sources,
    get_recipe_sources,
    get_quest_sources,
    resolve_effect_descs,
    get_tsys_power_info,
    get_storage_vault_zones,
};
use db::player_commands::{
    add_market_price,
    get_market_prices_for_item,
    add_sale,
    get_recent_sales,
    start_survey_session,
    add_survey_result,
    add_survey_loot,
    complete_survey_session,
    get_survey_sessions,
    save_survey_session_stats,
    get_historical_sessions,
    log_event,
    get_recent_events,
};
use db::player_commands_survey_events::{
    log_survey_event,
    get_survey_events,
    log_survey_loot_item,
    get_survey_loot_items,
    get_speed_bonus_stats,
    get_loot_breakdown,
    get_survey_type_metrics,
};
use db::admin_commands::{
    get_database_stats,
    force_rebuild_cdn_tables,
    purge_player_data,
};
use db::character_commands::{
    import_character_report,
    get_characters,
    get_character_snapshots,
    get_snapshot_skills,
    get_snapshot_npc_favor,
    get_snapshot_recipes,
    get_snapshot_stats,
    get_snapshot_currencies,
    compare_snapshots,
};
use db::inventory_commands::{
    import_inventory_report,
    get_inventory_snapshots,
    get_snapshot_items,
    get_inventory_summary,
};
use settings::{
    SettingsManager,
    load_settings,
    save_settings,
    get_settings_file_path,
    get_server_list,
};
use setup_commands::{
    validate_game_data_path,
    scan_reports_for_characters,
    save_user_character,
    get_user_characters,
    set_active_character,
    complete_setup,
    import_latest_report_for_character,
    import_latest_inventory_for_character,
};
use chat_commands::{
    scan_chat_logs,
    scan_chat_log_file,
    get_chat_messages,
    get_chat_channels,
    get_chat_channel_stats,
    get_chat_stats,
    tail_chat_log,
    get_tell_conversations,
    purge_chat_messages,
    delete_all_chat_messages,
    get_watch_rule_messages,
};
use coordinator::{
    start_player_tailing,
    stop_player_tailing,
    start_chat_tailing,
    stop_chat_tailing,
    get_coordinator_status,
    poll_watchers,
    DataIngestCoordinator,
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

            // Step 1: Get app data directory
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Cannot resolve app data dir");

            // Step 2: Initialize settings FIRST (before database)
            let settings_manager = Arc::new(
                SettingsManager::new(app_data_dir.clone())
                    .expect("Failed to initialize settings")
            );

            // Step 3: Initialize database using settings
            let db_path = settings_manager.get_db_path(&app_data_dir);
            let db_pool = db::init_pool(db_path).expect("Failed to initialize database");

            // Step 4: Initialize DataIngestCoordinator
            let coordinator = Arc::new(Mutex::new(
                DataIngestCoordinator::new(
                    db_pool.clone(),
                    settings_manager.clone(),
                    app_handle.clone(),
                ).expect("Failed to initialize DataIngestCoordinator")
            ));

            // Step 5: Register managed state
            app.manage(settings_manager.clone());
            app.manage(db_pool.clone());
            app.manage(coordinator.clone());

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

                        // Persist CDN data to database (in background, non-blocking)
                        if let Some(pool) = app_handle.try_state::<db::DbPool>() {
                            if let Ok(mut conn) = pool.get() {
                                if let Err(e) = db::cdn_persistence::persist_cdn_data(&mut conn, &data) {
                                    eprintln!("Failed to persist CDN data to database: {e}");
                                } else {
                                    eprintln!("CDN data persisted to database successfully");
                                }
                            }
                        }

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
            get_equip_slots,
            // Skill queries
            get_all_skills,
            get_skill_by_name,
            // Ability queries
            get_abilities_for_skill,
            // Recipe queries
            get_recipe_by_name,
            get_recipes_for_item,
            get_recipes_using_item,
            search_recipes,
            get_recipes_for_skill,
            get_items_batch,
            // Quest queries
            get_all_quests,
            search_quests,
            get_quest_by_key,
            // NPC queries
            get_all_npcs,
            search_npcs,
            get_npcs_in_area,
            // Effect queries
            search_effects,
            get_effect,
            resolve_effect_descs,
            get_tsys_power_info,
            // Player Title queries
            get_all_player_titles,
            search_player_titles,
            // Storage vault queries
            get_storage_vault_zones,
            // Icons
            get_icon_path,
            // Source queries
            get_ability_sources,
            get_item_sources,
            get_recipe_sources,
            get_quest_sources,
            // Player data - Market prices
            add_market_price,
            get_market_prices_for_item,
            // Player data - Sales
            add_sale,
            get_recent_sales,
            // Player data - Surveys
            start_survey_session,
            add_survey_result,
            add_survey_loot,
            complete_survey_session,
            get_survey_sessions,
            // Player data - Survey session stats
            save_survey_session_stats,
            get_historical_sessions,
            // Player data - Survey events
            log_survey_event,
            get_survey_events,
            log_survey_loot_item,
            get_survey_loot_items,
            // Player data - Survey analytics
            get_speed_bonus_stats,
            get_loot_breakdown,
            get_survey_type_metrics,
            // Player data - Event log
            log_event,
            get_recent_events,
            // Database admin
            get_database_stats,
            force_rebuild_cdn_tables,
            purge_player_data,
            // Settings
            load_settings,
            save_settings,
            get_settings_file_path,
            get_server_list,
            // Setup / Onboarding
            validate_game_data_path,
            scan_reports_for_characters,
            save_user_character,
            get_user_characters,
            set_active_character,
            complete_setup,
            import_latest_report_for_character,
            import_latest_inventory_for_character,
            // Chat
            scan_chat_logs,
            scan_chat_log_file,
            get_chat_messages,
            get_chat_channels,
            get_chat_channel_stats,
            get_chat_stats,
            tail_chat_log,
            get_tell_conversations,
            purge_chat_messages,
            delete_all_chat_messages,
            get_watch_rule_messages,
            // Coordinator
            start_player_tailing,
            stop_player_tailing,
            start_chat_tailing,
            stop_chat_tailing,
            get_coordinator_status,
            poll_watchers,
            // Character import
            import_character_report,
            get_characters,
            get_character_snapshots,
            get_snapshot_skills,
            get_snapshot_npc_favor,
            get_snapshot_recipes,
            get_snapshot_stats,
            get_snapshot_currencies,
            compare_snapshots,
            // Inventory import
            import_inventory_report,
            get_inventory_snapshots,
            get_snapshot_items,
            get_inventory_summary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}