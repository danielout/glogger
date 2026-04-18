use super::DbPool;
use crate::cdn_commands::GameDataState;
use rusqlite::params;
use serde::Serialize;
use serde_json::Value;
use tauri::State;

// ── Response types ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct BrewingDiscovery {
    pub id: i64,
    pub character: String,
    pub recipe_id: u32,
    pub ingredient_ids: Vec<u32>,
    pub power: String,
    pub power_tier: i64,
    pub effect_label: Option<String>,
    pub race_restriction: Option<String>,
    pub item_name: Option<String>,
    pub first_seen_at: String,
    pub last_seen_at: String,
}

#[derive(Debug, Serialize)]
pub struct BrewingScanResult {
    pub new_discoveries: u32,
    pub updated_discoveries: u32,
    pub total_brewing_items: u32,
}

// ── Race detection from power names ─────────────────────────────────────────

const RACE_MARKERS: &[(&str, &str)] = &[
    ("Elf", "Elf"),
    ("Rakshasa", "Rakshasa"),
    ("Orc", "Orc"),
    ("Dwarf", "Dwarf"),
    ("Fae", "Fae"),
    ("Human", "Human"),
    ("Lycanthrope", "Lycanthrope"),
];

fn detect_race_restriction(power: &str) -> Option<String> {
    for (marker, race) in RACE_MARKERS {
        if power.contains(marker) {
            return Some(race.to_string());
        }
    }
    None
}

// ── Effect label extraction ─────────────────────────────────────────────────

/// Extract the effect label from the drink name by comparing to the base recipe name.
/// E.g., "Partier's Dwarven Stout" vs base "Dwarven Stout" → "Partier's"
/// E.g., "Dwarven Stout of Elfinity" vs base "Dwarven Stout" → "of Elfinity"
fn extract_effect_label(item_name: &str, base_name: &str) -> Option<String> {
    let item_lower = item_name.to_lowercase();
    let base_lower = base_name.to_lowercase();

    // Check for suffix: "Base Name of Something"
    if let Some(pos) = item_lower.find(&base_lower) {
        let prefix = item_name[..pos].trim();
        let suffix_start = pos + base_name.len();
        let suffix = item_name[suffix_start..].trim();

        if !prefix.is_empty() && !suffix.is_empty() {
            return Some(format!("{} ... {}", prefix, suffix));
        }
        if !prefix.is_empty() {
            return Some(prefix.to_string());
        }
        if !suffix.is_empty() {
            return Some(suffix.to_string());
        }
    }

    // Fallback: if names are identical, no label
    if item_lower == base_lower {
        return None;
    }

    // Can't determine — return the full name as the label
    Some(item_name.to_string())
}

// ── Scan a snapshot's raw JSON for brewing discoveries ──────────────────────

#[tauri::command]
pub async fn scan_snapshot_for_brewing_discoveries(
    snapshot_id: i64,
    db: State<'_, DbPool>,
    game_data: State<'_, GameDataState>,
) -> Result<BrewingScanResult, String> {
    let data = game_data.read().await;
    let conn = db.get().map_err(|e| format!("DB error: {e}"))?;
    scan_snapshot_internal(snapshot_id, &conn, &data)
}

// ── Query discoveries ───────────────────────────────────────────────────────

#[tauri::command]
pub fn get_brewing_discoveries(
    character: String,
    recipe_id: Option<u32>,
    db: State<'_, DbPool>,
) -> Result<Vec<BrewingDiscovery>, String> {
    let conn = db.get().map_err(|e| format!("DB error: {e}"))?;

    if let Some(rid) = recipe_id {
        let mut stmt = conn
            .prepare(
                "SELECT id, character, recipe_id, ingredient_ids, power, power_tier,
                        effect_label, race_restriction, item_name, first_seen_at, last_seen_at
                 FROM brewing_discoveries
                 WHERE character = ?1 AND recipe_id = ?2
                 ORDER BY first_seen_at DESC",
            )
            .map_err(|e| format!("Query error: {e}"))?;

        let discoveries: Vec<BrewingDiscovery> = stmt
            .query_map(params![character, rid], map_discovery_row)
            .map_err(|e| format!("Query error: {e}"))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(discoveries)
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT id, character, recipe_id, ingredient_ids, power, power_tier,
                        effect_label, race_restriction, item_name, first_seen_at, last_seen_at
                 FROM brewing_discoveries
                 WHERE character = ?1
                 ORDER BY first_seen_at DESC",
            )
            .map_err(|e| format!("Query error: {e}"))?;

        let discoveries: Vec<BrewingDiscovery> = stmt
            .query_map(params![character], map_discovery_row)
            .map_err(|e| format!("Query error: {e}"))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(discoveries)
    }
}

fn map_discovery_row(row: &rusqlite::Row) -> rusqlite::Result<BrewingDiscovery> {
    let ingredient_ids_json: String = row.get(3)?;
    let ingredient_ids: Vec<u32> =
        serde_json::from_str(&ingredient_ids_json).unwrap_or_default();

    Ok(BrewingDiscovery {
        id: row.get(0)?,
        character: row.get(1)?,
        recipe_id: row.get::<_, u32>(2)?,
        ingredient_ids,
        power: row.get(4)?,
        power_tier: row.get(5)?,
        effect_label: row.get(6)?,
        race_restriction: row.get(7)?,
        item_name: row.get(8)?,
        first_seen_at: row.get(9)?,
        last_seen_at: row.get(10)?,
    })
}

/// Scan ALL snapshots for a character to backfill brewing discoveries.
#[tauri::command]
pub async fn scan_all_snapshots_for_brewing(
    character: String,
    db: State<'_, DbPool>,
    game_data: State<'_, GameDataState>,
) -> Result<BrewingScanResult, String> {
    let data = game_data.read().await;
    let conn = db.get().map_err(|e| format!("DB error: {e}"))?;

    // Get all snapshot IDs for this character
    let mut stmt = conn
        .prepare(
            "SELECT id FROM character_item_snapshots WHERE character_name = ?1 ORDER BY snapshot_timestamp ASC",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let snapshot_ids: Vec<i64> = stmt
        .query_map(params![character], |row| row.get(0))
        .map_err(|e| format!("Query error: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    drop(stmt);

    let mut total_new = 0u32;
    let mut total_updated = 0u32;
    let mut total_items = 0u32;

    for snapshot_id in snapshot_ids {
        let result = scan_snapshot_internal(snapshot_id, &conn, &data)?;
        total_new += result.new_discoveries;
        total_updated += result.updated_discoveries;
        total_items += result.total_brewing_items;
    }

    Ok(BrewingScanResult {
        new_discoveries: total_new,
        updated_discoveries: total_updated,
        total_brewing_items: total_items,
    })
}

/// Internal scan logic shared between single-snapshot and bulk-scan commands.
fn scan_snapshot_internal(
    snapshot_id: i64,
    conn: &rusqlite::Connection,
    data: &crate::game_data::GameData,
) -> Result<BrewingScanResult, String> {
    // 1. Get snapshot metadata + raw JSON
    let (character, raw_json, timestamp): (String, String, String) = conn
        .query_row(
            "SELECT character_name, raw_json, snapshot_timestamp FROM character_item_snapshots WHERE id = ?1",
            params![snapshot_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("Snapshot not found: {e}"))?;

    // 2. Parse raw JSON
    let report: Value =
        serde_json::from_str(&raw_json).map_err(|e| format!("Failed to parse JSON: {e}"))?;

    let items = match report.get("Items").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(BrewingScanResult { new_discoveries: 0, updated_discoveries: 0, total_brewing_items: 0 }),
    };

    // 3. Build result_item_id → recipe mapping
    let mut result_to_recipe: std::collections::HashMap<u32, &crate::game_data::brewing::BrewingRecipe> =
        std::collections::HashMap::new();
    for recipe in &data.brewing_recipes {
        if let Some(result_id) = recipe.result_item_id {
            result_to_recipe.insert(result_id, recipe);
        }
    }

    // 4. Scan and insert
    let mut new_discoveries = 0u32;
    let mut updated_discoveries = 0u32;
    let mut total_brewing_items = 0u32;

    let mut insert_stmt = conn
        .prepare(
            "INSERT INTO brewing_discoveries (
                character, recipe_id, ingredient_ids, power, power_tier,
                effect_label, race_restriction, item_name, first_seen_at, last_seen_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)
             ON CONFLICT(character, recipe_id, ingredient_ids) DO UPDATE SET
                last_seen_at = ?9",
        )
        .map_err(|e| format!("Failed to prepare insert: {e}"))?;

    for item in items {
        let ingredient_ids = match item.get("IngredientItemTypeIds").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => continue,
        };
        let tsys_powers = match item.get("TSysPowers").and_then(|v| v.as_array()) {
            Some(arr) if !arr.is_empty() => arr,
            _ => continue,
        };
        let type_id = match item.get("TypeID").and_then(|v| v.as_u64()) {
            Some(id) => id as u32,
            None => continue,
        };

        // Only count items crafted by this character — skip other players' brews
        let crafter = item.get("Crafter").and_then(|v| v.as_str()).unwrap_or("");
        if !crafter.eq_ignore_ascii_case(&character) {
            continue;
        }

        let recipe = match result_to_recipe.get(&type_id) {
            Some(r) => r,
            None => continue,
        };

        total_brewing_items += 1;

        let mut ing_ids: Vec<u32> = ingredient_ids
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n as u32))
            .collect();
        ing_ids.sort();
        let ing_ids_json = serde_json::to_string(&ing_ids).unwrap_or_default();

        let power = match tsys_powers[0].get("Power").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => continue,
        };
        let power_tier = tsys_powers[0]
            .get("Tier")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let item_name = item.get("Name").and_then(|v| v.as_str()).unwrap_or("");
        let base_name = data
            .items
            .get(&type_id)
            .map(|i| i.name.as_str())
            .unwrap_or("");

        let effect_label = extract_effect_label(item_name, base_name);
        let race_restriction = detect_race_restriction(power);

        let changes = insert_stmt
            .execute(params![
                character,
                recipe.recipe_id,
                ing_ids_json,
                power,
                power_tier,
                effect_label,
                race_restriction,
                item_name,
                timestamp,
            ])
            .map_err(|e| format!("Failed to insert discovery: {e}"))?;

        if changes > 0 {
            let disc_id = conn.last_insert_rowid();
            let is_new: bool = conn
                .query_row(
                    "SELECT first_seen_at = last_seen_at FROM brewing_discoveries WHERE rowid = ?1",
                    params![disc_id],
                    |row| row.get(0),
                )
                .unwrap_or(true);

            if is_new {
                new_discoveries += 1;
            } else {
                updated_discoveries += 1;
            }
        }
    }

    Ok(BrewingScanResult {
        new_discoveries,
        updated_discoveries,
        total_brewing_items,
    })
}

// ── CSV import ──────────────────────────────────────────────────────────────

/// Import brewing discoveries from a CSV file.
///
/// CSV format (header required):
///   ingredient1,ingredient2,ingredient3,ingredient4,power,power_tier,item_name,type_id
///
/// Ingredient columns use display names (e.g., "Corn", "Groxmax Powder").
/// ingredient4 can be empty for 3-ingredient recipes.
/// power is the TSysPower internal name (e.g., "BrewingLumberjack").
/// item_name is the full drink name (e.g., "Lumberjack's Dwarven Stout").
/// type_id is the CDN item TypeID of the result drink.
#[tauri::command]
pub async fn import_brewing_discoveries_csv(
    file_path: String,
    character: String,
    db: State<'_, DbPool>,
    game_data: State<'_, GameDataState>,
) -> Result<BrewingScanResult, String> {
    let data = game_data.read().await;
    let conn = db.get().map_err(|e| format!("DB error: {e}"))?;

    let csv_content =
        std::fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file: {e}"))?;

    // Build item name → ID lookup (case-insensitive)
    let mut name_to_id: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for (id, item) in &data.items {
        name_to_id.insert(item.name.to_lowercase(), *id);
        if let Some(ref iname) = item.internal_name {
            name_to_id.insert(iname.to_lowercase(), *id);
        }
    }

    // Build result_item_id → recipe mapping
    let mut result_to_recipe: std::collections::HashMap<u32, &crate::game_data::brewing::BrewingRecipe> =
        std::collections::HashMap::new();
    for recipe in &data.brewing_recipes {
        if let Some(result_id) = recipe.result_item_id {
            result_to_recipe.insert(result_id, recipe);
        }
    }

    let mut insert_stmt = conn
        .prepare(
            "INSERT INTO brewing_discoveries (
                character, recipe_id, ingredient_ids, power, power_tier,
                effect_label, race_restriction, item_name, first_seen_at, last_seen_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)
             ON CONFLICT(character, recipe_id, ingredient_ids) DO UPDATE SET
                last_seen_at = ?9",
        )
        .map_err(|e| format!("Failed to prepare insert: {e}"))?;

    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let mut new_discoveries = 0u32;
    let mut updated_discoveries = 0u32;
    let mut total_brewing_items = 0u32;
    let mut skipped_lines = Vec::new();

    for (line_num, line) in csv_content.lines().enumerate() {
        // Skip header
        if line_num == 0 && line.to_lowercase().contains("ingredient") {
            continue;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 6 {
            skipped_lines.push(format!("Line {}: too few fields", line_num + 1));
            continue;
        }

        // Resolve ingredient names to IDs
        let mut ingredient_ids: Vec<u32> = Vec::new();
        let mut resolve_failed = false;
        for i in 0..4 {
            let name = fields.get(i).map(|s| s.trim()).unwrap_or("");
            if name.is_empty() {
                continue;
            }
            match name_to_id.get(&name.to_lowercase()) {
                Some(id) => ingredient_ids.push(*id),
                None => {
                    skipped_lines.push(format!(
                        "Line {}: unknown ingredient \"{}\"",
                        line_num + 1,
                        name
                    ));
                    resolve_failed = true;
                    break;
                }
            }
        }
        if resolve_failed || ingredient_ids.is_empty() {
            continue;
        }

        let power = fields.get(4).map(|s| s.trim()).unwrap_or("");
        let power_tier: i64 = fields
            .get(5)
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        let item_name = fields.get(6).map(|s| s.trim()).unwrap_or("");
        let type_id: Option<u32> = fields.get(7).and_then(|s| s.trim().parse().ok());

        if power.is_empty() {
            skipped_lines.push(format!("Line {}: missing power", line_num + 1));
            continue;
        }

        // Resolve recipe from type_id
        let recipe_id = if let Some(tid) = type_id {
            result_to_recipe.get(&tid).map(|r| r.recipe_id)
        } else {
            None
        };

        let recipe_id = match recipe_id {
            Some(id) => id,
            None => {
                skipped_lines.push(format!(
                    "Line {}: could not match type_id {} to a brewing recipe",
                    line_num + 1,
                    type_id.unwrap_or(0)
                ));
                continue;
            }
        };

        total_brewing_items += 1;

        ingredient_ids.sort();
        let ing_ids_json = serde_json::to_string(&ingredient_ids).unwrap_or_default();

        // Get base name for effect label extraction
        let base_name = type_id
            .and_then(|tid| data.items.get(&tid))
            .map(|i| i.name.as_str())
            .unwrap_or("");
        let effect_label = extract_effect_label(item_name, base_name);
        let race_restriction = detect_race_restriction(power);

        let changes = insert_stmt
            .execute(params![
                character,
                recipe_id,
                ing_ids_json,
                power,
                power_tier,
                effect_label,
                race_restriction,
                item_name,
                timestamp,
            ])
            .map_err(|e| format!("Failed to insert discovery: {e}"))?;

        if changes > 0 {
            let disc_id = conn.last_insert_rowid();
            let is_new: bool = conn
                .query_row(
                    "SELECT first_seen_at = last_seen_at FROM brewing_discoveries WHERE rowid = ?1",
                    params![disc_id],
                    |row| row.get(0),
                )
                .unwrap_or(true);

            if is_new {
                new_discoveries += 1;
            } else {
                updated_discoveries += 1;
            }
        }
    }

    if !skipped_lines.is_empty() {
        eprintln!(
            "Brewing CSV import: {} lines skipped:\n{}",
            skipped_lines.len(),
            skipped_lines.join("\n")
        );
    }

    Ok(BrewingScanResult {
        new_discoveries,
        updated_discoveries,
        total_brewing_items,
    })
}
