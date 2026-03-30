use super::DbPool;
use serde::{Deserialize, Serialize};
/// Build planner persistence commands
use tauri::State;

// ── Input types ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateBuildPresetInput {
    pub character_id: String,
    pub name: String,
    pub skill_primary: Option<String>,
    pub skill_secondary: Option<String>,
    pub target_level: Option<i32>,
    pub target_rarity: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateBuildPresetInput {
    pub id: i64,
    pub name: String,
    pub skill_primary: Option<String>,
    pub skill_secondary: Option<String>,
    pub target_level: i32,
    pub target_rarity: String,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct BuildPresetModInput {
    pub equip_slot: String,
    pub power_name: String,
    pub tier: Option<i32>,
    pub is_augment: bool,
    pub sort_order: i32,
}

#[derive(Deserialize)]
pub struct SetSlotItemInput {
    pub preset_id: i64,
    pub equip_slot: String,
    pub item_id: i64,
    pub item_name: Option<String>,
}

#[derive(Deserialize)]
pub struct BuildPresetAbilityInput {
    pub bar: String,
    pub slot_position: i32,
    pub ability_id: i64,
    pub ability_name: Option<String>,
}

// ── Output types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct BuildPresetAbility {
    pub id: i64,
    pub preset_id: i64,
    pub bar: String,
    pub slot_position: i32,
    pub ability_id: i64,
    pub ability_name: Option<String>,
}

#[derive(Serialize)]
pub struct BuildPresetSlotItem {
    pub preset_id: i64,
    pub equip_slot: String,
    pub item_id: i64,
    pub item_name: Option<String>,
}

#[derive(Serialize)]
pub struct BuildPreset {
    pub id: i64,
    pub character_id: String,
    pub name: String,
    pub skill_primary: Option<String>,
    pub skill_secondary: Option<String>,
    pub target_level: i32,
    pub target_rarity: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct BuildPresetMod {
    pub id: i64,
    pub preset_id: i64,
    pub equip_slot: String,
    pub power_name: String,
    pub tier: Option<i32>,
    pub is_augment: bool,
    pub sort_order: i32,
}

// ── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn create_build_preset(
    db: State<'_, DbPool>,
    input: CreateBuildPresetInput,
) -> Result<i64, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    conn.execute(
        "INSERT INTO build_presets (character_id, name, skill_primary, skill_secondary, target_level, target_rarity)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            input.character_id,
            input.name,
            input.skill_primary,
            input.skill_secondary,
            input.target_level.unwrap_or(90),
            input.target_rarity.as_deref().unwrap_or("Epic"),
        ],
    ).map_err(|e| format!("Failed to create build preset: {e}"))?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn get_build_presets(
    db: State<'_, DbPool>,
    character_id: String,
) -> Result<Vec<BuildPreset>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, character_id, name, skill_primary, skill_secondary,
                target_level, target_rarity, notes,
                datetime(created_at), datetime(updated_at)
         FROM build_presets
         WHERE character_id = ?1
         ORDER BY updated_at DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt
        .query_map([&character_id], |row| {
            Ok(BuildPreset {
                id: row.get(0)?,
                character_id: row.get(1)?,
                name: row.get(2)?,
                skill_primary: row.get(3)?,
                skill_secondary: row.get(4)?,
                target_level: row.get(5)?,
                target_rarity: row.get(6)?,
                notes: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut presets = Vec::new();
    for row in rows {
        presets.push(row.map_err(|e| format!("Row parse error: {e}"))?);
    }
    Ok(presets)
}

#[tauri::command]
pub fn update_build_preset(
    db: State<'_, DbPool>,
    input: UpdateBuildPresetInput,
) -> Result<(), String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    conn.execute(
        "UPDATE build_presets
         SET name = ?1, skill_primary = ?2, skill_secondary = ?3,
             target_level = ?4, target_rarity = ?5, notes = ?6,
             updated_at = datetime('now')
         WHERE id = ?7",
        rusqlite::params![
            input.name,
            input.skill_primary,
            input.skill_secondary,
            input.target_level,
            input.target_rarity,
            input.notes,
            input.id,
        ],
    )
    .map_err(|e| format!("Failed to update build preset: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn delete_build_preset(db: State<'_, DbPool>, preset_id: i64) -> Result<(), String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    conn.execute("DELETE FROM build_presets WHERE id = ?1", [preset_id])
        .map_err(|e| format!("Failed to delete build preset: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn get_build_preset_mods(
    db: State<'_, DbPool>,
    preset_id: i64,
) -> Result<Vec<BuildPresetMod>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, preset_id, equip_slot, power_name, tier, is_augment, sort_order
         FROM build_preset_mods
         WHERE preset_id = ?1
         ORDER BY equip_slot, sort_order",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt
        .query_map([preset_id], |row| {
            Ok(BuildPresetMod {
                id: row.get(0)?,
                preset_id: row.get(1)?,
                equip_slot: row.get(2)?,
                power_name: row.get(3)?,
                tier: row.get(4)?,
                is_augment: row.get::<_, i32>(5).map(|v| v != 0)?,
                sort_order: row.get(6)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut mods = Vec::new();
    for row in rows {
        mods.push(row.map_err(|e| format!("Row parse error: {e}"))?);
    }
    Ok(mods)
}

/// Replace all mods for a build preset (simpler than individual CRUD).
#[tauri::command]
pub fn set_build_preset_mods(
    db: State<'_, DbPool>,
    preset_id: i64,
    mods: Vec<BuildPresetModInput>,
) -> Result<(), String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    // Delete existing mods
    conn.execute(
        "DELETE FROM build_preset_mods WHERE preset_id = ?1",
        [preset_id],
    )
    .map_err(|e| format!("Failed to clear existing mods: {e}"))?;

    // Insert new mods
    let mut stmt = conn.prepare(
        "INSERT INTO build_preset_mods (preset_id, equip_slot, power_name, tier, is_augment, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
    ).map_err(|e| format!("Failed to prepare insert: {e}"))?;

    for m in &mods {
        stmt.execute(rusqlite::params![
            preset_id,
            m.equip_slot,
            m.power_name,
            m.tier,
            m.is_augment as i32,
            m.sort_order,
        ])
        .map_err(|e| format!("Failed to insert mod: {e}"))?;
    }

    // Touch preset updated_at
    conn.execute(
        "UPDATE build_presets SET updated_at = datetime('now') WHERE id = ?1",
        [preset_id],
    )
    .ok();

    Ok(())
}

// ── Slot item commands ─────────────────────────────────────────────────────

/// Set or replace the base item for a specific slot in a build preset.
#[tauri::command]
pub fn set_build_preset_slot_item(
    db: State<'_, DbPool>,
    input: SetSlotItemInput,
) -> Result<(), String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    conn.execute(
        "INSERT INTO build_preset_slot_items (preset_id, equip_slot, item_id, item_name)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(preset_id, equip_slot) DO UPDATE SET item_id = ?3, item_name = ?4",
        rusqlite::params![
            input.preset_id,
            input.equip_slot,
            input.item_id,
            input.item_name,
        ],
    )
    .map_err(|e| format!("Failed to set slot item: {e}"))?;

    conn.execute(
        "UPDATE build_presets SET updated_at = datetime('now') WHERE id = ?1",
        [input.preset_id],
    )
    .ok();

    Ok(())
}

/// Clear the base item for a specific slot in a build preset.
#[tauri::command]
pub fn clear_build_preset_slot_item(
    db: State<'_, DbPool>,
    preset_id: i64,
    equip_slot: String,
) -> Result<(), String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    conn.execute(
        "DELETE FROM build_preset_slot_items WHERE preset_id = ?1 AND equip_slot = ?2",
        rusqlite::params![preset_id, equip_slot],
    )
    .map_err(|e| format!("Failed to clear slot item: {e}"))?;

    Ok(())
}

/// Get all slot items for a build preset.
#[tauri::command]
pub fn get_build_preset_slot_items(
    db: State<'_, DbPool>,
    preset_id: i64,
) -> Result<Vec<BuildPresetSlotItem>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT preset_id, equip_slot, item_id, item_name
         FROM build_preset_slot_items
         WHERE preset_id = ?1",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt
        .query_map([preset_id], |row| {
            Ok(BuildPresetSlotItem {
                preset_id: row.get(0)?,
                equip_slot: row.get(1)?,
                item_id: row.get(2)?,
                item_name: row.get(3)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| format!("Row parse error: {e}"))?);
    }
    Ok(items)
}

// ── Ability bar commands ───────────────────────────────────────────────────

/// Replace all abilities for a specific bar in a build preset.
#[tauri::command]
pub fn set_build_preset_abilities(
    db: State<'_, DbPool>,
    preset_id: i64,
    bar: String,
    abilities: Vec<BuildPresetAbilityInput>,
) -> Result<(), String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    // Delete existing abilities for this bar
    conn.execute(
        "DELETE FROM build_preset_abilities WHERE preset_id = ?1 AND bar = ?2",
        rusqlite::params![preset_id, bar],
    )
    .map_err(|e| format!("Failed to clear abilities: {e}"))?;

    // Insert new abilities
    let mut stmt = conn.prepare(
        "INSERT INTO build_preset_abilities (preset_id, bar, slot_position, ability_id, ability_name)
         VALUES (?1, ?2, ?3, ?4, ?5)"
    ).map_err(|e| format!("Failed to prepare insert: {e}"))?;

    for a in &abilities {
        stmt.execute(rusqlite::params![
            preset_id,
            a.bar,
            a.slot_position,
            a.ability_id,
            a.ability_name,
        ])
        .map_err(|e| format!("Failed to insert ability: {e}"))?;
    }

    conn.execute(
        "UPDATE build_presets SET updated_at = datetime('now') WHERE id = ?1",
        [preset_id],
    )
    .ok();

    Ok(())
}

/// Get all abilities for a build preset across all bars.
#[tauri::command]
pub fn get_build_preset_abilities(
    db: State<'_, DbPool>,
    preset_id: i64,
) -> Result<Vec<BuildPresetAbility>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, preset_id, bar, slot_position, ability_id, ability_name
         FROM build_preset_abilities
         WHERE preset_id = ?1
         ORDER BY bar, slot_position",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt
        .query_map([preset_id], |row| {
            Ok(BuildPresetAbility {
                id: row.get(0)?,
                preset_id: row.get(1)?,
                bar: row.get(2)?,
                slot_position: row.get(3)?,
                ability_id: row.get(4)?,
                ability_name: row.get(5)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut abilities = Vec::new();
    for row in rows {
        abilities.push(row.map_err(|e| format!("Row parse error: {e}"))?);
    }
    Ok(abilities)
}
