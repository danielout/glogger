mod cdn;
mod cdn_commands;
mod cdn_diff;
mod chat_commands;
mod chat_parser;
mod chat_combat_parser;
mod chat_resuscitate_parser;
mod chat_status_parser;
mod commands;
mod coordinator;
mod debug_capture;
mod db;
mod game_data;
mod game_state;
mod log_watchers;
mod parsers;
mod player_event_parser;
mod replay;
mod report_stats;
mod settings;
mod setup_commands;
mod shop_log_parser;
mod stall_aggregations;
mod stall_year_resolver;
mod survey;
mod external_fetch;
mod gst_manager;
mod trip_router;
mod watch_rules;
mod zone_graph;

use chrono::Local;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{Emitter, Manager};
use tokio::sync::RwLock;

use cdn_commands::{
    check_cdn_version,
    force_refresh_cdn,
    get_abilities_for_skill,
    get_ability_families_for_skill,
    get_ability_family,
    get_ability_sources,
    get_skills_with_ability_counts,
    search_ability_families,
    get_all_item_keywords,
    get_all_lorebooks,
    get_all_npcs,
    get_all_player_titles,
    get_all_quests,
    get_all_skills,
    get_cache_status,
    get_combat_skills,
    get_cp_recipes_for_slot,
    get_effect,
    get_equip_slots,
    get_icon_path,
    get_item_sources,
    get_items_by_keyword,
    get_lorebook_categories,
    get_recipe_ingredient_keywords,
    get_npcs_in_area,
    get_quest_sources,
    get_recipe_sources,
    get_recipes_for_item,
    get_recipes_for_skill,
    get_recipes_producing_items,
    get_brewing_recipes,
    get_brewing_ingredients,
    get_recipes_using_item,
    get_vendor_purchasable_item_ids,
    get_npc_vendor_items,
    get_vendor_item_counts,
    get_vendors_for_item,
    get_storage_vault_metadata,
    get_storage_vault_zones,
    get_all_tsys,
    get_tsys_power_info,
    get_tsys_power_info_batch,
    get_tsys_powers_for_slot,
    get_tsys_profiles,
    get_tsys_for_ability,
    get_abilities_for_tsys,
    get_tsys_ability_map,
    search_tsys,
    get_xp_table_for_skill,
    init_game_data,
    resolve_ability,
    resolve_area,
    resolve_effect_descs,
    // Unified entity resolvers
    resolve_item,
    resolve_items_batch,
    resolve_npc,
    resolve_quest,
    resolve_recipe,
    resolve_skill,
    search_effects,
    search_lorebooks,
    // Query/filter commands (not replaced by resolvers)
    search_items,
    search_npcs,
    search_player_titles,
    search_quests,
    search_recipes,
    // Cross-reference commands
    get_npcs_wanting_item,
    get_npcs_training_skill,
    get_quests_for_npc,
    get_quests_for_skill,
    get_quests_by_moon_phase,
    get_current_moon_phase,
    get_recipes_for_keyword,
    find_recipe_items_in_inventory,
    cdn_diff_summary,
    cdn_diff_file,
    GameDataState,
};
use db::brewing_commands::{
    get_brewing_discoveries,
    import_brewing_discoveries_csv,
    delete_brewing_discovery,
    scan_all_snapshots_for_brewing,
    scan_snapshot_for_brewing_discoveries,
};
use chat_commands::{
    delete_all_chat_messages, get_chat_channel_stats, get_chat_channels, get_chat_messages,
    get_chat_messages_around, get_chat_stats, get_tell_conversations, get_watch_rule_messages,
    purge_chat_messages, scan_chat_log_file, scan_chat_logs, tail_chat_log,
};
use commands::parse_log;
use coordinator::{
    debug_capture_discard, debug_capture_save, debug_capture_start, debug_capture_status,
    debug_capture_stop,
    get_coordinator_status, poll_watchers, start_background_polling, stop_background_polling,
    start_chat_tailing, start_player_tailing,
    stop_chat_tailing, stop_player_tailing,
    spawn_polling_thread, DataIngestCoordinator, PollingHandle,
};
use db::admin_commands::{force_rebuild_cdn_tables, get_database_stats, purge_player_data};
use db::aggregate_commands::{get_aggregate_inventory, get_aggregate_skills, get_aggregate_wealth};
use db::build_planner_commands::{
    clear_build_preset_slot_item, clone_build_preset, create_build_preset, delete_build_preset,
    export_build_preset, get_build_preset_abilities, get_build_preset_cp_recipes,
    get_build_preset_mods, get_build_preset_slot_items, get_build_presets,
    import_build_preset, set_build_preset_abilities, set_build_preset_cp_recipes,
    set_build_preset_mods, set_build_preset_slot_item, update_build_preset,
    update_build_preset_slot_props,
};
use db::character_commands::{
    compare_snapshots, get_character_snapshots, get_characters, get_snapshot_active_quests,
    get_snapshot_currencies, get_snapshot_npc_favor, get_snapshot_recipes, get_snapshot_skills,
    get_snapshot_stats, import_character_report,
};
use db::death_commands::{get_character_deaths, get_death_damage_sources};
use db::resuscitate_commands::get_character_resuscitations;
use db::crafting_commands::{
    add_project_entry, batch_update_entry_expansions, check_material_availability,
    create_crafting_project, delete_crafting_project, duplicate_crafting_project,
    get_crafting_project, get_crafting_projects, get_work_orders_from_snapshot,
    remove_project_entry, reorder_project_entries, update_crafting_project,
    update_project_entry,
};
use db::farming_commands::{
    delete_farming_session, get_farming_sessions, save_farming_session, update_farming_session,
};
use db::game_state_commands::{
    get_game_state_active_skills, get_game_state_attributes, get_game_state_currencies,
    get_game_state_effects, get_game_state_equipment, get_game_state_favor,
    get_game_state_inventory, get_game_state_recipes, get_game_state_skills,
    get_character_report_stats, get_game_state_books, get_game_state_storage,
    get_game_state_vendor, get_game_state_world, get_gift_log, get_milking_timers,
    add_manual_gift, remove_last_gift, get_tracked_skills, set_tracked_skills,
    get_computed_stats,
    get_attribute_extremes,
    get_player_milking_leaderboard,
    get_teleportation_binds,
    set_mushroom_circles,
};
use db::gourmand_commands::{
    export_text_file, get_all_foods, get_gourmand_eaten_foods, import_cooks_helper_file,
    import_gourmand_report, import_latest_gourmand_report,
};
use db::inventory_commands::{
    get_inventory_snapshots, get_inventory_summary, get_snapshot_items, import_inventory_report,
};
use db::timer_commands::{delete_user_timer, get_user_timers, save_user_timer};
use db::market_commands::{
    delete_market_value, export_market_values, get_market_value, get_market_values,
    import_market_values, set_market_value,
};
use db::player_commands::{
    add_market_price, add_sale, get_market_prices_for_item, get_recent_events, get_recent_sales,
    log_event,
};
use replay::replay_dual_logs;
use trip_router::plan_trip;
use external_fetch::{fetch_github_releases, fetch_pg_news};
use gst_manager::{gst_check_status, gst_download, gst_launch};
use settings::{
    get_server_list, get_settings_file_path, load_settings, save_settings, SettingsManager,
};
use setup_commands::{
    complete_setup, delete_character, get_user_characters, import_latest_inventory_for_character,
    import_latest_report_for_character, import_reports_for_server, save_user_character,
    scan_reports_for_characters, set_active_character, validate_game_data_path,
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
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
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

            // Step 3: Track app version (database is preserved across upgrades via migrations)
            let db_path = settings_manager.get_db_path(&app_data_dir);
            {
                let stored_version = settings_manager.get().last_app_version;
                if stored_version.as_ref() != current_version.as_ref() {
                    startup_log!(
                        "App version changed ({} -> {}), migrations will handle schema updates",
                        stored_version.as_deref().unwrap_or("first run"),
                        current_version.as_deref().unwrap_or("unknown"),
                    );
                    // Invalidate cached indices so CDN data is re-indexed with new code.
                    // This ensures bug fixes to index-building logic (e.g. ability family
                    // construction) take effect without waiting for a CDN version bump.
                    let indices_path = app_data_dir.join("data").join("indices.json");
                    if indices_path.exists() {
                        startup_log!("Removing cached indices.json to force re-index");
                        let _ = std::fs::remove_file(&indices_path);
                    }
                }
                let mut updated = settings_manager.get();
                updated.last_app_version = current_version.clone();
                settings_manager.update(updated).expect("Failed to save version to settings");
            }

            // Step 4: Initialize database (migrations run automatically)
            // Pass timezone offset for one-time migration to fix historical timestamps
            let tz_offset = {
                let s = settings_manager.get();
                s.manual_timezone_override.or(s.timezone_offset_seconds)
            };
            let db_pool = db::init_pool(db_path, tz_offset).expect("Failed to initialize database");
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

            // Step 5b: Spawn background polling thread (starts paused,
            // frontend calls start_background_polling after catch-up)
            let polling_handle = Arc::new(Mutex::new(
                spawn_polling_thread(coordinator.clone(), std::time::Duration::from_millis(500))
            ));
            startup_log!("Background polling thread spawned (paused)");

            // Step 6: Register managed state
            app.manage(settings_manager.clone());
            app.manage(db_pool.clone());
            app.manage(coordinator.clone());
            app.manage(polling_handle.clone());
            app.manage(db::stall_tracker_commands::StallOpsLock::default());

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

                        // Persist CDN data to database only if version changed
                        if let Some(pool) = app_handle.try_state::<db::DbPool>() {
                            if let Ok(mut conn) = pool.get() {
                                let db_version = db::queries::cdn_data::get_cdn_version(&conn).ok().flatten();
                                if db_version == Some(data.version) {
                                    startup_log!("CDN v{} already persisted to database, skipping", data.version);
                                } else {
                                    cdn_commands::emit_startup_detail(&app_handle, 0, "Saving to database...");
                                    if let Err(e) = db::cdn_persistence::persist_cdn_data(&mut conn, &data) {
                                        startup_log!("Failed to persist CDN data to database: {e}");
                                    } else {
                                        startup_log!("CDN data persisted to database");
                                    }
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
            check_cdn_version,
            force_refresh_cdn,
            cdn_diff_summary,
            cdn_diff_file,
            // Unified entity resolvers
            resolve_item,
            resolve_items_batch,
            resolve_skill,
            resolve_recipe,
            resolve_quest,
            resolve_npc,
            resolve_ability,
            resolve_area,
            // Item queries
            search_items,
            get_items_by_keyword,
            get_recipe_ingredient_keywords,
            get_all_item_keywords,
            get_equip_slots,
            // Skill queries
            get_all_skills,
            // Ability queries
            get_abilities_for_skill,
            get_ability_families_for_skill,
            get_ability_family,
            search_ability_families,
            get_skills_with_ability_counts,
            // Recipe queries
            get_recipes_for_item,
            get_recipes_using_item,
            search_recipes,
            get_recipes_for_skill,
            get_recipes_producing_items,
            // Brewing queries
            get_brewing_recipes,
            get_brewing_ingredients,
            get_brewing_discoveries,
            scan_snapshot_for_brewing_discoveries,
            scan_all_snapshots_for_brewing,
            import_brewing_discoveries_csv,
            delete_brewing_discovery,
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
            get_tsys_power_info_batch,
            // TSys browser queries
            get_all_tsys,
            search_tsys,
            get_tsys_profiles,
            get_tsys_for_ability,
            get_abilities_for_tsys,
            get_tsys_ability_map,
            // Player Title queries
            get_all_player_titles,
            search_player_titles,
            // Lorebook queries
            get_all_lorebooks,
            get_lorebook_categories,
            search_lorebooks,
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
            get_vendor_purchasable_item_ids,
            get_npc_vendor_items,
            get_vendor_item_counts,
            get_vendors_for_item,
            // Cross-reference queries
            get_npcs_wanting_item,
            get_npcs_training_skill,
            get_quests_for_npc,
            get_quests_for_skill,
            get_quests_by_moon_phase,
            get_current_moon_phase,
            get_recipes_for_keyword,
            find_recipe_items_in_inventory,
            // Player data - Market prices
            add_market_price,
            get_market_prices_for_item,
            // Player data - Sales
            add_sale,
            get_recent_sales,
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
            get_chat_messages_around,
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
            start_background_polling,
            stop_background_polling,
            // Debug capture
            debug_capture_start,
            debug_capture_stop,
            debug_capture_save,
            debug_capture_discard,
            debug_capture_status,
            // Survey tracker (Phase 5)
            survey::commands::survey_tracker_status,
            survey::commands::survey_tracker_start_session,
            survey::commands::survey_tracker_end_session,
            survey::commands::survey_tracker_recent_sessions,
            survey::commands::survey_tracker_session_detail,
            survey::commands::survey_tracker_historical_sessions,
            survey::commands::survey_tracker_update_session_notes,
            survey::commands::survey_tracker_update_session_name,
            survey::commands::survey_tracker_update_session_times,
            survey::commands::survey_tracker_delete_session,
            survey::commands::survey_tracker_analytics,
            survey::commands::survey_tracker_item_cost_analysis,
            // Character import
            import_character_report,
            get_characters,
            get_character_snapshots,
            get_snapshot_skills,
            get_snapshot_npc_favor,
            get_snapshot_recipes,
            get_snapshot_stats,
            get_snapshot_currencies,
            get_snapshot_active_quests,
            compare_snapshots,
            // Character deaths
            get_character_deaths,
            get_death_damage_sources,
            // Resuscitations
            get_character_resuscitations,
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
            batch_update_entry_expansions,
            remove_project_entry,
            reorder_project_entries,
            duplicate_crafting_project,
            check_material_availability,
            get_work_orders_from_snapshot,
            // CDN - XP tables
            get_xp_table_for_skill,
            // CDN - Build planner queries
            get_combat_skills,
            get_tsys_powers_for_slot,
            get_cp_recipes_for_slot,
            // Build planner persistence
            create_build_preset,
            clone_build_preset,
            get_build_presets,
            update_build_preset,
            delete_build_preset,
            get_build_preset_mods,
            set_build_preset_mods,
            set_build_preset_slot_item,
            clear_build_preset_slot_item,
            get_build_preset_slot_items,
            update_build_preset_slot_props,
            set_build_preset_abilities,
            get_build_preset_abilities,
            get_build_preset_cp_recipes,
            set_build_preset_cp_recipes,
            export_build_preset,
            import_build_preset,
            // Game state queries
            get_game_state_skills,
            get_game_state_attributes,
            get_game_state_active_skills,
            get_game_state_world,
            get_game_state_inventory,
            get_game_state_recipes,
            get_game_state_equipment,
            get_game_state_favor,
            get_gift_log,
            add_manual_gift,
            remove_last_gift,
            get_game_state_currencies,
            get_game_state_effects,
            get_game_state_storage,
            get_game_state_vendor,
            get_game_state_books,
            get_milking_timers,
            get_character_report_stats,
            get_computed_stats,
            get_attribute_extremes,
            get_player_milking_leaderboard,
            get_tracked_skills,
            set_tracked_skills,
            get_market_values,
            get_market_value,
            set_market_value,
            delete_market_value,
            export_market_values,
            import_market_values,
            get_aggregate_inventory,
            get_aggregate_wealth,
            get_aggregate_skills,
            // Stall Tracker
            db::stall_tracker_commands::get_stall_events,
            db::stall_tracker_commands::get_stall_stats,
            db::stall_tracker_commands::get_stall_revenue,
            db::stall_tracker_commands::get_stall_inventory,
            db::stall_tracker_commands::get_stall_filter_options,
            db::stall_tracker_commands::toggle_stall_event_ignored,
            db::stall_tracker_commands::clear_stall_events,
            db::stall_tracker_commands::import_shop_log_file,
            db::stall_tracker_commands::export_shop_log_files,
            db::stall_tracker_commands::seed_stall_events_dev,
            // Words of Power
            db::words_of_power_commands::get_words_of_power,
            db::words_of_power_commands::add_word_of_power,
            db::words_of_power_commands::delete_word_of_power,
            // User timers
            get_user_timers,
            save_user_timer,
            delete_user_timer,
            // Teleportation binds
            get_teleportation_binds,
            set_mushroom_circles,
            // Trip routing
            plan_trip,
            // External content
            fetch_github_releases,
            fetch_pg_news,
            // GorgonSurveyTracker management
            gst_check_status,
            gst_download,
            gst_launch,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                // Stop the background polling thread first so it doesn't
                // fight with the coordinator lock during shutdown.
                if let Some(ph) = app_handle.try_state::<Arc<Mutex<PollingHandle>>>() {
                    ph.lock().unwrap().shutdown();
                    startup_log!("[shutdown] Polling thread stopped.");
                }

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
