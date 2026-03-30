use rusqlite::params;
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;
use tauri::State;

use super::DbPool;
use crate::settings::SettingsManager;

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct FoodItemInfo {
    pub item_id: u32,
    pub name: String,
    pub icon_id: Option<u32>,
    pub food_category: String,
    pub food_level: i64,
    pub gourmand_req: Option<i64>,
    pub effect_descs: Vec<String>,
    pub keywords: Vec<String>,
    pub value: Option<f32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct GourmandFoodEntry {
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Serialize, Clone)]
pub struct GourmandImportResult {
    pub foods_imported: usize,
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Get all food items from the pre-parsed foods table
#[tauri::command]
pub fn get_all_foods(db: State<'_, DbPool>) -> Result<Vec<FoodItemInfo>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn.prepare(
        "SELECT item_id, name, icon_id, food_category, food_level, gourmand_req, effect_descs, keywords, value
         FROM foods
         ORDER BY food_level ASC, name ASC"
    ).map_err(|e| format!("Query prepare error: {e}"))?;

    let foods = stmt
        .query_map([], |row| {
            let effects_str: String = row.get(6)?;
            let keywords_str: String = row.get(7)?;

            let effect_descs: Vec<String> = serde_json::from_str(&effects_str).unwrap_or_default();
            let keywords: Vec<String> = serde_json::from_str(&keywords_str).unwrap_or_default();

            Ok(FoodItemInfo {
                item_id: row.get(0)?,
                name: row.get(1)?,
                icon_id: row.get(2)?,
                food_category: row.get(3)?,
                food_level: row.get(4)?,
                gourmand_req: row.get(5)?,
                effect_descs,
                keywords,
                value: row.get(8)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?;

    let result: Vec<FoodItemInfo> = foods.filter_map(|r| r.ok()).collect();
    Ok(result)
}

/// Import a gourmand report from a text file, persist as "last known" snapshot
#[tauri::command]
pub fn import_gourmand_report(
    db: State<'_, DbPool>,
    file_path: String,
) -> Result<GourmandImportResult, String> {
    let content =
        std::fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file: {e}"))?;

    let entries = parse_gourmand_report(&content)?;

    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    // Replace all existing data with new import (single snapshot)
    conn.execute("DELETE FROM gourmand_eaten_foods", [])
        .map_err(|e| format!("Failed to clear old data: {e}"))?;

    let mut stmt = conn
        .prepare("INSERT INTO gourmand_eaten_foods (food_name, times_eaten) VALUES (?1, ?2)")
        .map_err(|e| format!("Prepare error: {e}"))?;

    for entry in &entries {
        stmt.execute(params![&entry.name, entry.count])
            .map_err(|e| format!("Insert error: {e}"))?;
    }

    Ok(GourmandImportResult {
        foods_imported: entries.len(),
    })
}

/// Get the last-imported eaten foods from the database
#[tauri::command]
pub fn get_gourmand_eaten_foods(db: State<'_, DbPool>) -> Result<Vec<GourmandFoodEntry>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn
        .prepare("SELECT food_name, times_eaten FROM gourmand_eaten_foods ORDER BY food_name ASC")
        .map_err(|e| format!("Query prepare error: {e}"))?;

    let entries = stmt
        .query_map([], |row| {
            Ok(GourmandFoodEntry {
                name: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?;

    let result: Vec<GourmandFoodEntry> = entries.filter_map(|r| r.ok()).collect();
    Ok(result)
}

/// Import a player's gourmand report (the in-game SkillReport .txt file).
/// Parses the eaten food names and returns them without persisting to the database.
/// Used by the Cook's Helper to determine what a player has already eaten.
#[tauri::command]
pub fn import_cooks_helper_file(file_path: String) -> Result<Vec<String>, String> {
    let content =
        std::fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file: {e}"))?;

    let entries = parse_gourmand_report(&content)?;
    if entries.is_empty() {
        return Err(
            "No food entries found in the file. Make sure this is a gourmand skill report."
                .to_string(),
        );
    }

    Ok(entries.into_iter().map(|e| e.name).collect())
}

/// Write text content to a file (used for uneaten food export)
#[tauri::command]
pub fn export_text_file(file_path: String, content: String) -> Result<(), String> {
    std::fs::write(&file_path, &content).map_err(|e| format!("Failed to write file: {e}"))
}

/// Scan the Books folder for the latest gourmand report and auto-import it.
/// Returns Some(result) if a new report was imported, None otherwise.
#[tauri::command]
pub fn import_latest_gourmand_report(
    db: State<'_, DbPool>,
    settings_manager: State<'_, Arc<SettingsManager>>,
) -> Result<Option<GourmandImportResult>, String> {
    let settings = settings_manager.get();
    let game_data_path = &settings.game_data_path;
    if game_data_path.is_empty() {
        return Ok(None);
    }

    let books_dir = Path::new(game_data_path).join("Books");
    if !books_dir.is_dir() {
        return Ok(None);
    }

    let best_file = find_latest_gourmand_report(&books_dir)?;
    let file_path = match best_file {
        Some(p) => p,
        None => return Ok(None),
    };

    let content =
        std::fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file: {e}"))?;

    let entries = parse_gourmand_report(&content)?;
    if entries.is_empty() {
        return Ok(None);
    }

    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    // Check if we already have this exact data (avoid unnecessary re-imports)
    let existing_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM gourmand_eaten_foods", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    if existing_count == entries.len() as i64 {
        // Same count — likely the same report. Skip re-import.
        return Ok(None);
    }

    // Replace all existing data
    conn.execute("DELETE FROM gourmand_eaten_foods", [])
        .map_err(|e| format!("Failed to clear old data: {e}"))?;

    let mut stmt = conn
        .prepare("INSERT INTO gourmand_eaten_foods (food_name, times_eaten) VALUES (?1, ?2)")
        .map_err(|e| format!("Prepare error: {e}"))?;

    for entry in &entries {
        stmt.execute(params![&entry.name, entry.count])
            .map_err(|e| format!("Insert error: {e}"))?;
    }

    Ok(Some(GourmandImportResult {
        foods_imported: entries.len(),
    }))
}

/// Find the latest SkillReport_*.txt file in the Books folder that is a gourmand report
fn find_latest_gourmand_report(books_dir: &Path) -> Result<Option<std::path::PathBuf>, String> {
    let entries =
        std::fs::read_dir(books_dir).map_err(|e| format!("Failed to read Books directory: {e}"))?;

    let mut best: Option<(String, std::path::PathBuf)> = None;

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !file_name.starts_with("SkillReport_") || !file_name.ends_with(".txt") {
            continue;
        }

        let file_path = entry.path();

        // Read the first line to check if it's a gourmand report
        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let first_line = content.lines().next().unwrap_or("");
        if first_line.trim() != "Foods Consumed:" {
            continue;
        }

        // Use the filename as the sort key (contains timestamp: SkillReport_YYMMDD_HHMMSS.txt)
        if best
            .as_ref()
            .map_or(true, |(existing, _)| file_name > *existing)
        {
            best = Some((file_name, file_path));
        }
    }

    Ok(best.map(|(_, path)| path))
}

// ── Report parsing ────────────────────────────────────────────────────────────

/// Parse a gourmand report text file into food entries.
///
/// Expected format:
/// ```text
/// Foods Consumed:
///
///   All-Flavor Chicken (HAS MEAT) (HAS DAIRY): 2
///   Almonds: 1
/// ```
fn parse_gourmand_report(content: &str) -> Result<Vec<GourmandFoodEntry>, String> {
    let mut entries = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip header, blank lines, and non-food lines
        if trimmed.is_empty() || trimmed == "Foods Consumed:" {
            continue;
        }

        // Find the last colon — count is after it
        let last_colon = match trimmed.rfind(':') {
            Some(idx) => idx,
            None => continue,
        };

        let count_str = trimmed[last_colon + 1..].trim();
        let count: u32 = match count_str.parse() {
            Ok(n) => n,
            Err(_) => continue,
        };

        let name_part = trimmed[..last_colon].trim();

        // Strip parenthetical tags like (HAS MEAT), (HAS DAIRY), (HAS EGGS)
        let name = strip_food_tags(name_part);

        if !name.is_empty() {
            entries.push(GourmandFoodEntry { name, count });
        }
    }

    Ok(entries)
}

/// Strip parenthetical tags like "(HAS MEAT)" from a food name
fn strip_food_tags(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut depth = 0;

    for ch in name.chars() {
        match ch {
            '(' => depth += 1,
            ')' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ if depth == 0 => result.push(ch),
            _ => {}
        }
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_food_tags() {
        assert_eq!(
            strip_food_tags("All-Flavor Chicken (HAS MEAT) (HAS DAIRY)"),
            "All-Flavor Chicken"
        );
        assert_eq!(strip_food_tags("Almonds"), "Almonds");
        assert_eq!(
            strip_food_tags("Cottage Pie (HAS MEAT) (HAS DAIRY)"),
            "Cottage Pie"
        );
    }

    #[test]
    fn test_parse_gourmand_report() {
        let content = "Foods Consumed:\n\n  All-Flavor Chicken (HAS MEAT) (HAS DAIRY): 2\n  Almonds: 1\n  Cactus Juice: 38\n";
        let entries = parse_gourmand_report(content).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].name, "All-Flavor Chicken");
        assert_eq!(entries[0].count, 2);
        assert_eq!(entries[1].name, "Almonds");
        assert_eq!(entries[1].count, 1);
        assert_eq!(entries[2].name, "Cactus Juice");
        assert_eq!(entries[2].count, 38);
    }
}
