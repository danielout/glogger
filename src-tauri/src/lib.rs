mod parsers;
mod commands;
mod survey_parser;
mod survey_persistence;
mod player_event_parser;
mod cdn;
mod game_data;
mod cdn_commands;
mod db;
mod settings;
mod chat_parser;
mod chat_status_parser;
mod chat_commands;
mod log_watchers;
mod coordinator;
mod game_state;
mod setup_commands;
mod watch_rules;
mod replay;

use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::RwLock;
use tauri::{Emitter, Manager};
use chrono::Local;

use commands::parse_log;
use replay::replay_dual_logs;
use cdn_commands::{
    GameDataState,
    init_game_data,
    get_cache_status,
    force_refresh_cdn,
    // Unified entity resolvers
    resolve_item,
    resolve_items_batch,
    resolve_skill,
    resolve_recipe,
    resolve_quest,
    resolve_npc,
    resolve_area,
    // Query/filter commands (not replaced by resolvers)
    search_items,
    get_items_by_keyword,
    get_all_item_keywords,
    get_equip_slots,
    get_all_skills,
    get_abilities_for_skill,
    get_recipes_for_item,
    get_recipes_using_item,
    search_recipes,
    get_recipes_for_skill,
    get_all_quests,
    search_quests,
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
    get_storage_vault_metadata,
    get_xp_table_for_skill,
};
use db::player_commands::{
    add_market_price,
    get_market_prices_for_item,
    add_sale,
    get_recent_sales,
    save_survey_session_stats,
    patch_survey_session,
    get_historical_sessions,
    update_survey_session,
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
    get_zone_analytics,
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
use db::gourmand_commands::{
    get_all_foods,
    import_gourmand_report,
    import_cooks_helper_file,
    get_gourmand_eaten_foods,
    export_text_file,
    import_latest_gourmand_report,
};
use db::survey_commands::{
    get_all_survey_types,
};
use db::farming_commands::{
    save_farming_session,
    get_farming_sessions,
    update_farming_session,
    delete_farming_session,
};
use db::crafting_commands::{
    create_crafting_project,
    get_crafting_projects,
    get_crafting_project,
    update_crafting_project,
    delete_crafting_project,
    add_project_entry,
    update_project_entry,
    remove_project_entry,
    reorder_project_entries,
    duplicate_crafting_project,
    check_material_availability,
    get_work_orders_from_snapshot,
};
use db::game_state_commands::{
    get_game_state_skills,
    get_game_state_attributes,
    get_game_state_active_skills,
    get_game_state_world,
    get_game_state_inventory,
    get_game_state_recipes,
    get_game_state_equipment,
    get_game_state_favor,
    get_game_state_currencies,
    get_game_state_effects,
    get_game_state_storage,
};
use db::market_commands::{
    get_market_values,
    get_market_value,
    set_market_value,
    delete_market_value,
    export_market_values,
    import_market_values,
};
use db::aggregate_commands::{
    get_aggregate_inventory,
    get_aggregate_wealth,
    get_aggregate_skills,
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
    delete_character,
    complete_setup,
    import_latest_report_for_character,
    import_latest_inventory_for_character,
    import_reports_for_server,
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

/// Timestamped log line for startup diagnostics.
macro_rules! startup_log {
    ($($arg:tt)*) => {
        eprintln!("[{}] {}", Local::now().format("%H:%M:%S%.3f"), format!($($arg)*));
    };
}

/// Frontend can push a message into the stderr startup log stream.
#[tauri::command]
fn log_startup(message: String) {
    startup_log!("{}", message);
}

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

            let current_version = app.config().version.clone();
            startup_log!("glogger v{} starting up", current_version.as_deref().unwrap_or("unknown"));

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
            startup_log!("Settings loaded");

            // Step 3: Nuke database on prototype version upgrade
            let db_path = settings_manager.get_db_path(&app_data_dir);
            {
                let stored_version = settings_manager.get().last_app_version;
                let version_changed = match &stored_version {
                    Some(v) => current_version.as_ref().map_or(true, |cv| cv != v),
                    None => false, // First run, no nuke needed
                };
                if version_changed {
                    startup_log!(
                        "Prototype version changed ({} -> {}), deleting old database",
                        stored_version.as_deref().unwrap_or("unknown"),
                        current_version.as_deref().unwrap_or("unknown"),
                    );
                    // Delete main db file and WAL/SHM sidecars
                    let _ = std::fs::remove_file(&db_path);
                    let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
                    let _ = std::fs::remove_file(db_path.with_extension("db-shm"));
                }
                // Persist the current version so next launch knows
                let mut updated = settings_manager.get();
                updated.last_app_version = current_version.clone();
                settings_manager.update(updated).expect("Failed to save version to settings");
            }

            // Step 4: Initialize database (fresh if just nuked)
            let db_pool = db::init_pool(db_path).expect("Failed to initialize database");
            startup_log!("Database initialized");

            // Step 5: Initialize DataIngestCoordinator
            let coordinator = Arc::new(Mutex::new(
                DataIngestCoordinator::new(
                    db_pool.clone(),
                    settings_manager.clone(),
                    app_handle.clone(),
                    state.clone(),
                ).expect("Failed to initialize DataIngestCoordinator")
            ));
            startup_log!("Coordinator initialized");

            // Step 6: Register managed state
            app.manage(settings_manager.clone());
            app.manage(db_pool.clone());
            app.manage(coordinator.clone());

            startup_log!("Splash screen displayed (frontend rendering)");

            // Kick off CDN init in the background — non-blocking.
            // The frontend should listen for a "game-data-ready" event
            // or poll get_cache_status() to know when data is available.
            startup_log!("Starting game data load (background)");
            tauri::async_runtime::spawn(async move {
                let t0 = Instant::now();
                match init_game_data(&app_handle, &state).await {
                    Ok(()) => {
                        let data = state.read().await;
                        startup_log!(
                            "Game data ready: v{} — {} items, {} skills, {} recipes, {} npcs, {} effects, {} areas (took {:.1}s)",
                            data.version,
                            data.items.len(),
                            data.skills.len(),
                            data.recipes.len(),
                            data.npcs.len(),
                            data.effects.len(),
                            data.areas.len(),
                            t0.elapsed().as_secs_f64(),
                        );

                        // Persist CDN data to database (in background, non-blocking)
                        if let Some(pool) = app_handle.try_state::<db::DbPool>() {
                            if let Ok(mut conn) = pool.get() {
                                if let Err(e) = db::cdn_persistence::persist_cdn_data(&mut conn, &data) {
                                    startup_log!("Failed to persist CDN data to database: {e}");
                                } else {
                                    startup_log!("CDN data persisted to database");
                                }
                            }
                        }

                        // Notify the frontend that data is loaded
                        app_handle.emit("game-data-ready", data.version).ok();
                    }
                    Err(e) => {
                        startup_log!("Game data init FAILED: {e}");
                        app_handle.emit("game-data-error", e).ok();
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Startup diagnostics
            log_startup,
            // Log parsing (manual upload)
            parse_log,
            replay_dual_logs,
            // CDN management
            get_cache_status,
            force_refresh_cdn,
            // Unified entity resolvers
            resolve_item,
            resolve_items_batch,
            resolve_skill,
            resolve_recipe,
            resolve_quest,
            resolve_npc,
            resolve_area,
            // Item queries
            search_items,
            get_items_by_keyword,
            get_all_item_keywords,
            get_equip_slots,
            // Skill queries
            get_all_skills,
            // Ability queries
            get_abilities_for_skill,
            // Recipe queries
            get_recipes_for_item,
            get_recipes_using_item,
            search_recipes,
            get_recipes_for_skill,
            // Quest queries
            get_all_quests,
            search_quests,
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
            get_storage_vault_metadata,
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
            // Player data - Survey session stats
            save_survey_session_stats,
            patch_survey_session,
            get_historical_sessions,
            update_survey_session,
            // Player data - Survey events
            log_survey_event,
            get_survey_events,
            log_survey_loot_item,
            get_survey_loot_items,
            // Player data - Survey analytics
            get_speed_bonus_stats,
            get_loot_breakdown,
            get_survey_type_metrics,
            get_zone_analytics,
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
            delete_character,
            complete_setup,
            import_latest_report_for_character,
            import_latest_inventory_for_character,
            import_reports_for_server,
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
            // Gourmand tracker
            get_all_foods,
            import_gourmand_report,
            import_cooks_helper_file,
            get_gourmand_eaten_foods,
            export_text_file,
            import_latest_gourmand_report,
            get_all_survey_types,
            // Farming calculator
            save_farming_session,
            get_farming_sessions,
            update_farming_session,
            delete_farming_session,
            // Crafting helper
            create_crafting_project,
            get_crafting_projects,
            get_crafting_project,
            update_crafting_project,
            delete_crafting_project,
            add_project_entry,
            update_project_entry,
            remove_project_entry,
            reorder_project_entries,
            duplicate_crafting_project,
            check_material_availability,
            get_work_orders_from_snapshot,
            // CDN - XP tables
            get_xp_table_for_skill,
            // Game state queries
            get_game_state_skills,
            get_game_state_attributes,
            get_game_state_active_skills,
            get_game_state_world,
            get_game_state_inventory,
            get_game_state_recipes,
            get_game_state_equipment,
            get_game_state_favor,
            get_game_state_currencies,
            get_game_state_effects,
            get_game_state_storage,
            get_market_values,
            get_market_value,
            set_market_value,
            delete_market_value,
            export_market_values,
            import_market_values,
            get_aggregate_inventory,
            get_aggregate_wealth,
            get_aggregate_skills,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                // Save watcher positions before the process dies
                if let Some(coordinator) = app_handle.try_state::<Arc<Mutex<DataIngestCoordinator>>>() {
                    let mut coord = coordinator.lock().unwrap();
                    if let Err(e) = coord.stop_player_log_tailing() {
                        startup_log!("[shutdown] Failed to save player log position: {e}");
                    }
                    if let Err(e) = coord.stop_chat_log_tailing() {
                        startup_log!("[shutdown] Failed to save chat log position: {e}");
                    }
                    startup_log!("[shutdown] Watcher positions saved.");
                }
            }
        });
}