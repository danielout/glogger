/// Tauri commands for querying persisted game state

use tauri::State;
use serde::{Deserialize, Serialize};
use super::DbPool;

// ── Response Types ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct GameStateSkill {
    pub skill_id: i64,
    pub skill_name: String,
    pub level: i32,
    pub base_level: i32,
    pub bonus_levels: i32,
    pub xp: i64,
    pub tnl: i64,
    pub max_level: i32,
    pub last_confirmed_at: String,
    pub source: String,
}

#[derive(Serialize)]
pub struct GameStateAttribute {
    pub attribute_name: String,
    pub value: f64,
    pub last_confirmed_at: String,
}

#[derive(Serialize)]
pub struct GameStateActiveSkills {
    pub skill1_id: i64,
    pub skill1_name: String,
    pub skill2_id: i64,
    pub skill2_name: String,
    pub last_confirmed_at: String,
}

#[derive(Serialize)]
pub struct GameStateWorld {
    pub weather: Option<GameStateWeather>,
    pub combat: Option<GameStateCombat>,
    pub mount: Option<GameStateMount>,
}

#[derive(Serialize)]
pub struct GameStateWeather {
    pub weather_name: String,
    pub is_active: bool,
    pub last_confirmed_at: String,
}

#[derive(Serialize)]
pub struct GameStateCombat {
    pub in_combat: bool,
    pub last_confirmed_at: String,
}

#[derive(Serialize)]
pub struct GameStateMount {
    pub is_mounted: bool,
    pub last_confirmed_at: String,
}

#[derive(Serialize)]
pub struct GameStateInventoryItem {
    pub instance_id: i64,
    pub item_name: String,
    pub item_type_id: Option<i32>,
    pub stack_size: i32,
    pub slot_index: i32,
    pub last_confirmed_at: String,
    pub source: String,
}

#[derive(Serialize)]
pub struct GameStateRecipe {
    pub recipe_id: i32,
    pub completion_count: i32,
    pub last_confirmed_at: String,
    pub source: String,
}

#[derive(Serialize)]
pub struct GameStateEquipmentSlot {
    pub slot: String,
    pub appearance_key: String,
    pub last_confirmed_at: String,
}

#[derive(Serialize)]
pub struct GameStateFavor {
    pub npc_key: String,
    pub npc_name: String,
    pub cumulative_delta: f64,
    pub favor_tier: Option<String>,
    pub last_confirmed_at: String,
    pub source: String,
}

#[derive(Serialize)]
pub struct GameStateCurrency {
    pub currency_name: String,
    pub amount: f64,
    pub last_confirmed_at: String,
    pub source: String,
}

#[derive(Serialize)]
pub struct GameStateEffect {
    pub effect_instance_id: i64,
    pub effect_name: Option<String>,
    pub source_entity_id: i64,
    pub last_confirmed_at: String,
}

#[derive(Serialize)]
pub struct GameStateStorageItem {
    pub vault_key: String,
    pub instance_id: i64,
    pub item_name: String,
    pub item_type_id: Option<i32>,
    pub stack_size: i32,
    pub slot_index: i32,
    pub last_confirmed_at: String,
    pub source: String,
}

// ── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_game_state_skills(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<GameStateSkill>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let mut stmt = conn.prepare(
        "SELECT skill_id, skill_name, level, base_level, bonus_levels, xp, tnl, max_level, last_confirmed_at, source
         FROM game_state_skills WHERE character_name = ?1 AND server_name = ?2
         ORDER BY skill_name"
    ).map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt.query_map(rusqlite::params![character_name, server_name], |row| {
        Ok(GameStateSkill {
            skill_id: row.get(0)?,
            skill_name: row.get(1)?,
            level: row.get(2)?,
            base_level: row.get(3)?,
            bonus_levels: row.get(4)?,
            xp: row.get(5)?,
            tnl: row.get(6)?,
            max_level: row.get(7)?,
            last_confirmed_at: row.get(8)?,
            source: row.get(9)?,
        })
    }).map_err(|e| format!("Query error: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))
}

#[tauri::command]
pub fn get_game_state_attributes(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<GameStateAttribute>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let mut stmt = conn.prepare(
        "SELECT attribute_name, value, last_confirmed_at
         FROM game_state_attributes WHERE character_name = ?1 AND server_name = ?2
         ORDER BY attribute_name"
    ).map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt.query_map(rusqlite::params![character_name, server_name], |row| {
        Ok(GameStateAttribute {
            attribute_name: row.get(0)?,
            value: row.get(1)?,
            last_confirmed_at: row.get(2)?,
        })
    }).map_err(|e| format!("Query error: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))
}

#[tauri::command]
pub fn get_game_state_active_skills(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Option<GameStateActiveSkills>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let result = conn.query_row(
        "SELECT skill1_id, skill1_name, skill2_id, skill2_name, last_confirmed_at
         FROM game_state_active_skills WHERE character_name = ?1 AND server_name = ?2",
        rusqlite::params![character_name, server_name],
        |row| {
            Ok(GameStateActiveSkills {
                skill1_id: row.get(0)?,
                skill1_name: row.get(1)?,
                skill2_id: row.get(2)?,
                skill2_name: row.get(3)?,
                last_confirmed_at: row.get(4)?,
            })
        },
    );

    match result {
        Ok(r) => Ok(Some(r)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Query error: {e}")),
    }
}

#[tauri::command]
pub fn get_game_state_world(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<GameStateWorld, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;

    // Weather (singleton, not per-character)
    let weather = conn.query_row(
        "SELECT weather_name, is_active, last_confirmed_at FROM game_state_weather WHERE id = 1",
        [],
        |row| {
            Ok(GameStateWeather {
                weather_name: row.get(0)?,
                is_active: row.get::<_, i32>(1)? != 0,
                last_confirmed_at: row.get(2)?,
            })
        },
    ).ok();

    // Combat
    let combat = conn.query_row(
        "SELECT in_combat, last_confirmed_at FROM game_state_combat WHERE character_name = ?1 AND server_name = ?2",
        rusqlite::params![character_name, server_name],
        |row| {
            Ok(GameStateCombat {
                in_combat: row.get::<_, i32>(0)? != 0,
                last_confirmed_at: row.get(1)?,
            })
        },
    ).ok();

    // Mount
    let mount = conn.query_row(
        "SELECT is_mounted, last_confirmed_at FROM game_state_mount WHERE character_name = ?1 AND server_name = ?2",
        rusqlite::params![character_name, server_name],
        |row| {
            Ok(GameStateMount {
                is_mounted: row.get::<_, i32>(0)? != 0,
                last_confirmed_at: row.get(1)?,
            })
        },
    ).ok();

    Ok(GameStateWorld { weather, combat, mount })
}

#[tauri::command]
pub fn get_game_state_inventory(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<GameStateInventoryItem>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let mut stmt = conn.prepare(
        "SELECT instance_id, item_name, item_type_id, stack_size, slot_index, last_confirmed_at, source
         FROM game_state_inventory WHERE character_name = ?1 AND server_name = ?2
         ORDER BY slot_index"
    ).map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt.query_map(rusqlite::params![character_name, server_name], |row| {
        Ok(GameStateInventoryItem {
            instance_id: row.get(0)?,
            item_name: row.get(1)?,
            item_type_id: row.get(2)?,
            stack_size: row.get(3)?,
            slot_index: row.get(4)?,
            last_confirmed_at: row.get(5)?,
            source: row.get(6)?,
        })
    }).map_err(|e| format!("Query error: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))
}

#[tauri::command]
pub fn get_game_state_recipes(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<GameStateRecipe>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let mut stmt = conn.prepare(
        "SELECT recipe_id, completion_count, last_confirmed_at, source
         FROM game_state_recipes WHERE character_name = ?1 AND server_name = ?2
         ORDER BY recipe_id"
    ).map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt.query_map(rusqlite::params![character_name, server_name], |row| {
        Ok(GameStateRecipe {
            recipe_id: row.get(0)?,
            completion_count: row.get(1)?,
            last_confirmed_at: row.get(2)?,
            source: row.get(3)?,
        })
    }).map_err(|e| format!("Query error: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))
}

#[tauri::command]
pub fn get_game_state_equipment(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<GameStateEquipmentSlot>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let mut stmt = conn.prepare(
        "SELECT slot, appearance_key, last_confirmed_at
         FROM game_state_equipment WHERE character_name = ?1 AND server_name = ?2
         ORDER BY slot"
    ).map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt.query_map(rusqlite::params![character_name, server_name], |row| {
        Ok(GameStateEquipmentSlot {
            slot: row.get(0)?,
            appearance_key: row.get(1)?,
            last_confirmed_at: row.get(2)?,
        })
    }).map_err(|e| format!("Query error: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))
}

#[tauri::command]
pub fn get_game_state_favor(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<GameStateFavor>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let mut stmt = conn.prepare(
        "SELECT npc_key, npc_name, cumulative_delta, favor_tier, last_confirmed_at, source
         FROM game_state_favor WHERE character_name = ?1 AND server_name = ?2
         ORDER BY npc_name"
    ).map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt.query_map(rusqlite::params![character_name, server_name], |row| {
        Ok(GameStateFavor {
            npc_key: row.get(0)?,
            npc_name: row.get(1)?,
            cumulative_delta: row.get(2)?,
            favor_tier: row.get(3)?,
            last_confirmed_at: row.get(4)?,
            source: row.get(5)?,
        })
    }).map_err(|e| format!("Query error: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))
}

#[tauri::command]
pub fn get_game_state_currencies(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<GameStateCurrency>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let mut stmt = conn.prepare(
        "SELECT currency_name, amount, last_confirmed_at, source
         FROM game_state_currencies WHERE character_name = ?1 AND server_name = ?2
         ORDER BY currency_name"
    ).map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt.query_map(rusqlite::params![character_name, server_name], |row| {
        Ok(GameStateCurrency {
            currency_name: row.get(0)?,
            amount: row.get(1)?,
            last_confirmed_at: row.get(2)?,
            source: row.get(3)?,
        })
    }).map_err(|e| format!("Query error: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))
}

#[tauri::command]
pub fn get_game_state_effects(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<GameStateEffect>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let mut stmt = conn.prepare(
        "SELECT effect_instance_id, effect_name, source_entity_id, last_confirmed_at
         FROM game_state_effects WHERE character_name = ?1 AND server_name = ?2
         ORDER BY effect_instance_id"
    ).map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt.query_map(rusqlite::params![character_name, server_name], |row| {
        Ok(GameStateEffect {
            effect_instance_id: row.get(0)?,
            effect_name: row.get(1)?,
            source_entity_id: row.get(2)?,
            last_confirmed_at: row.get(3)?,
        })
    }).map_err(|e| format!("Query error: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))
}

#[tauri::command]
pub fn get_game_state_storage(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<GameStateStorageItem>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let mut stmt = conn.prepare(
        "SELECT vault_key, instance_id, item_name, item_type_id, stack_size, slot_index, last_confirmed_at, source
         FROM game_state_storage WHERE character_name = ?1 AND server_name = ?2
         ORDER BY vault_key, item_name"
    ).map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt.query_map(rusqlite::params![character_name, server_name], |row| {
        Ok(GameStateStorageItem {
            vault_key: row.get(0)?,
            instance_id: row.get(1)?,
            item_name: row.get(2)?,
            item_type_id: row.get(3)?,
            stack_size: row.get(4)?,
            slot_index: row.get(5)?,
            last_confirmed_at: row.get(6)?,
            source: row.get(7)?,
        })
    }).map_err(|e| format!("Query error: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))
}

// ── Tracked Skills ─────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct TrackedSkill {
    pub skill_name: String,
    pub sort_order: i32,
}

#[derive(Deserialize)]
pub struct TrackedSkillEntry {
    pub skill_name: String,
    pub sort_order: i32,
}

#[tauri::command]
pub fn get_tracked_skills(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<TrackedSkill>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let mut stmt = conn.prepare(
        "SELECT skill_name, sort_order FROM tracked_skills
         WHERE character_name = ?1 AND server_name = ?2
         ORDER BY sort_order, skill_name"
    ).map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt.query_map(rusqlite::params![character_name, server_name], |row| {
        Ok(TrackedSkill {
            skill_name: row.get(0)?,
            sort_order: row.get(1)?,
        })
    }).map_err(|e| format!("Query error: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))
}

#[tauri::command]
pub fn set_tracked_skills(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
    skills: Vec<TrackedSkillEntry>,
) -> Result<(), String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;

    conn.execute(
        "DELETE FROM tracked_skills WHERE character_name = ?1 AND server_name = ?2",
        rusqlite::params![character_name, server_name],
    ).map_err(|e| format!("Delete error: {e}"))?;

    let mut stmt = conn.prepare(
        "INSERT INTO tracked_skills (character_name, server_name, skill_name, sort_order)
         VALUES (?1, ?2, ?3, ?4)"
    ).map_err(|e| format!("Prepare error: {e}"))?;

    for entry in &skills {
        stmt.execute(rusqlite::params![
            character_name, server_name, entry.skill_name, entry.sort_order
        ]).map_err(|e| format!("Insert error: {e}"))?;
    }

    Ok(())
}
