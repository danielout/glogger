/// Tauri commands for first-time setup and character management
///
/// Handles game folder validation, character discovery from Reports,
/// and user_characters CRUD for the startup flow.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbPool;
use crate::settings::SettingsManager;

// ── Response Types ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct GameDataPathValidation {
    pub path_exists: bool,
    pub player_log_found: bool,
    pub chat_logs_found: bool,
    pub reports_found: bool,
}

#[derive(Serialize, Clone)]
pub struct DiscoveredCharacter {
    pub character_name: String,
    pub server_name: String,
    pub report_count: usize,
    pub latest_report_time: Option<String>,
}

#[derive(Serialize)]
pub struct UserCharacter {
    pub id: i64,
    pub character_name: String,
    pub server_name: String,
    pub source: String,
    pub is_active: bool,
    pub latest_report_time: Option<String>,
    pub last_login_time: Option<String>,
}

// ── Minimal JSON struct for reading report headers ──────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ReportHeader {
    character: String,
    server_name: String,
    timestamp: Option<String>,
    report: Option<String>,
}

// ── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn validate_game_data_path(path: String) -> GameDataPathValidation {
    let base = Path::new(&path);
    GameDataPathValidation {
        path_exists: base.exists() && base.is_dir(),
        player_log_found: base.join("Player.log").exists(),
        chat_logs_found: base.join("ChatLogs").is_dir(),
        reports_found: base.join("Reports").is_dir(),
    }
}

#[tauri::command]
pub fn scan_reports_for_characters(path: String) -> Result<Vec<DiscoveredCharacter>, String> {
    let reports_dir = Path::new(&path).join("Reports");

    if !reports_dir.is_dir() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(&reports_dir)
        .map_err(|e| format!("Failed to read Reports directory: {e}"))?;

    // Accumulate by (character_name, server_name)
    let mut char_map: HashMap<(String, String), (usize, Option<String>)> = HashMap::new();

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Only process Character_*.json files
        if !file_name.starts_with("Character_") || !file_name.ends_with(".json") {
            continue;
        }

        let file_path = entry.path();
        let contents = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let header: ReportHeader = match serde_json::from_str(&contents) {
            Ok(h) => h,
            Err(_) => continue,
        };

        // Only count CharacterSheet reports
        if header.report.as_deref() != Some("CharacterSheet") {
            continue;
        }

        let key = (header.character.clone(), header.server_name.clone());
        let entry = char_map.entry(key).or_insert((0, None));
        entry.0 += 1;

        // Track the latest timestamp
        if let Some(ref ts) = header.timestamp {
            if entry.1.as_ref().map_or(true, |existing| ts > existing) {
                entry.1 = Some(ts.clone());
            }
        }
    }

    let mut result: Vec<DiscoveredCharacter> = char_map
        .into_iter()
        .map(|((name, server), (count, latest))| DiscoveredCharacter {
            character_name: name,
            server_name: server,
            report_count: count,
            latest_report_time: latest,
        })
        .collect();

    // Sort by latest report time descending, then by name
    result.sort_by(|a, b| {
        b.latest_report_time
            .cmp(&a.latest_report_time)
            .then_with(|| a.character_name.cmp(&b.character_name))
    });

    Ok(result)
}

#[tauri::command]
pub fn save_user_character(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
    source: String,
) -> Result<(), String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    conn.execute(
        "INSERT INTO user_characters (character_name, server_name, source)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(character_name, server_name) DO UPDATE SET
            source = CASE
                WHEN excluded.source = 'report' THEN 'report'
                WHEN user_characters.source = 'report' THEN 'report'
                ELSE excluded.source
            END,
            updated_at = CURRENT_TIMESTAMP",
        rusqlite::params![character_name, server_name, source],
    ).map_err(|e| format!("Failed to save user character: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn get_user_characters(
    db: State<'_, DbPool>,
) -> Result<Vec<UserCharacter>, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn.prepare(
        "SELECT id, character_name, server_name, source, is_active,
                latest_report_time, last_login_time
         FROM user_characters
         ORDER BY is_active DESC, updated_at DESC"
    ).map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt.query_map([], |row| {
        Ok(UserCharacter {
            id: row.get(0)?,
            character_name: row.get(1)?,
            server_name: row.get(2)?,
            source: row.get(3)?,
            is_active: row.get(4)?,
            latest_report_time: row.get(5)?,
            last_login_time: row.get(6)?,
        })
    }).map_err(|e| format!("Query failed: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read results: {e}"))
}

#[tauri::command]
pub fn set_active_character(
    db: State<'_, DbPool>,
    settings_manager: State<'_, Arc<SettingsManager>>,
    character_name: String,
    server_name: String,
) -> Result<(), String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    // Clear all active flags
    conn.execute("UPDATE user_characters SET is_active = 0", [])
        .map_err(|e| format!("Failed to clear active flags: {e}"))?;

    // Set the selected character as active
    conn.execute(
        "UPDATE user_characters SET is_active = 1, updated_at = CURRENT_TIMESTAMP
         WHERE character_name = ?1 AND server_name = ?2",
        rusqlite::params![character_name, server_name],
    ).map_err(|e| format!("Failed to set active character: {e}"))?;

    // Update settings
    let mut settings = settings_manager.get();
    settings.active_character_name = Some(character_name);
    settings.active_server_name = Some(server_name);
    settings_manager.update(settings)?;

    Ok(())
}

#[tauri::command]
pub fn complete_setup(
    settings_manager: State<'_, Arc<SettingsManager>>,
) -> Result<(), String> {
    let mut settings = settings_manager.get();
    settings.setup_completed = true;
    settings_manager.update(settings)
}

/// Find the latest Character_*.json report file for a given character in the Reports folder
/// and import it if not already in the database.
#[tauri::command]
pub fn import_latest_report_for_character(
    db: State<'_, DbPool>,
    settings_manager: State<'_, Arc<SettingsManager>>,
    character_name: String,
) -> Result<Option<crate::db::character_commands::ImportResult>, String> {
    let settings = settings_manager.get();
    let game_data_path = &settings.game_data_path;
    if game_data_path.is_empty() {
        return Err("Game data path not configured".into());
    }

    let reports_dir = Path::new(game_data_path).join("Reports");
    if !reports_dir.is_dir() {
        return Ok(None);
    }

    let entries = std::fs::read_dir(&reports_dir)
        .map_err(|e| format!("Failed to read Reports directory: {e}"))?;

    // Find all Character_*.json files for this character, pick the one with latest timestamp
    let mut best_file: Option<(String, std::path::PathBuf)> = None; // (timestamp, path)

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !file_name.starts_with("Character_") || !file_name.ends_with(".json") {
            continue;
        }

        let file_path = entry.path();
        let contents = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let header: ReportHeader = match serde_json::from_str(&contents) {
            Ok(h) => h,
            Err(_) => continue,
        };

        if header.report.as_deref() != Some("CharacterSheet") {
            continue;
        }

        if header.character != character_name {
            continue;
        }

        if let Some(ref ts) = header.timestamp {
            if best_file.as_ref().map_or(true, |(existing_ts, _)| ts > existing_ts) {
                best_file = Some((ts.clone(), file_path));
            }
        }
    }

    let Some((_, file_path)) = best_file else {
        return Ok(None);
    };

    let file_path_str = file_path.to_string_lossy().to_string();
    let result = crate::db::character_commands::import_character_report_internal(
        &db, &file_path_str,
    )?;

    // If it was a duplicate, return None (already imported)
    if result.was_duplicate {
        return Ok(None);
    }

    Ok(Some(result))
}

/// Find the latest *_items_*.json inventory report file for a given character
/// in the Reports folder and import it if not already in the database.
#[tauri::command]
pub fn import_latest_inventory_for_character(
    db: State<'_, DbPool>,
    settings_manager: State<'_, Arc<SettingsManager>>,
    character_name: String,
) -> Result<Option<crate::db::inventory_commands::InventoryImportResult>, String> {
    let settings = settings_manager.get();
    let game_data_path = &settings.game_data_path;
    if game_data_path.is_empty() {
        return Err("Game data path not configured".into());
    }

    let reports_dir = Path::new(game_data_path).join("Reports");
    if !reports_dir.is_dir() {
        return Ok(None);
    }

    let entries = std::fs::read_dir(&reports_dir)
        .map_err(|e| format!("Failed to read Reports directory: {e}"))?;

    // Find all *_items_*.json files for this character, pick latest timestamp
    let mut best_file: Option<(String, std::path::PathBuf)> = None;

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !file_name.ends_with(".json") || !file_name.contains("_items_") {
            continue;
        }

        let file_path = entry.path();
        let contents = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let header: ReportHeader = match serde_json::from_str(&contents) {
            Ok(h) => h,
            Err(_) => continue,
        };

        if header.report.as_deref() != Some("Storage") {
            continue;
        }

        if header.character != character_name {
            continue;
        }

        if let Some(ref ts) = header.timestamp {
            if best_file.as_ref().map_or(true, |(existing_ts, _)| ts > existing_ts) {
                best_file = Some((ts.clone(), file_path));
            }
        }
    }

    let Some((_, file_path)) = best_file else {
        return Ok(None);
    };

    let file_path_str = file_path.to_string_lossy().to_string();
    let result = crate::db::inventory_commands::import_inventory_report_internal(
        &db, &file_path_str,
    )?;

    if result.was_duplicate {
        return Ok(None);
    }

    Ok(Some(result))
}
