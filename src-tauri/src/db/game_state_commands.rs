use super::DbPool;
use serde::{Deserialize, Serialize};
/// Tauri commands for querying persisted game state
use tauri::State;

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
    pub area: Option<GameStateArea>,
}

#[derive(Serialize)]
pub struct GameStateArea {
    pub area_name: String,
    pub last_confirmed_at: String,
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

    let rows = stmt
        .query_map(rusqlite::params![character_name, server_name], |row| {
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
        })
        .map_err(|e| format!("Query error: {e}"))?;

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
    let mut stmt = conn
        .prepare(
            "SELECT attribute_name, value, last_confirmed_at
         FROM game_state_attributes WHERE character_name = ?1 AND server_name = ?2
         ORDER BY attribute_name",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![character_name, server_name], |row| {
            Ok(GameStateAttribute {
                attribute_name: row.get(0)?,
                value: row.get(1)?,
                last_confirmed_at: row.get(2)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?;

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

    // Area
    let area = conn.query_row(
        "SELECT area_name, last_confirmed_at FROM game_state_area WHERE character_name = ?1 AND server_name = ?2",
        rusqlite::params![character_name, server_name],
        |row| {
            Ok(GameStateArea {
                area_name: row.get(0)?,
                last_confirmed_at: row.get(1)?,
            })
        },
    ).ok();

    Ok(GameStateWorld {
        weather,
        combat,
        mount,
        area,
    })
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

    let rows = stmt
        .query_map(rusqlite::params![character_name, server_name], |row| {
            Ok(GameStateInventoryItem {
                instance_id: row.get(0)?,
                item_name: row.get(1)?,
                item_type_id: row.get(2)?,
                stack_size: row.get(3)?,
                slot_index: row.get(4)?,
                last_confirmed_at: row.get(5)?,
                source: row.get(6)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?;

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
    let mut stmt = conn
        .prepare(
            "SELECT recipe_id, completion_count, last_confirmed_at, source
         FROM game_state_recipes WHERE character_name = ?1 AND server_name = ?2
         ORDER BY recipe_id",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![character_name, server_name], |row| {
            Ok(GameStateRecipe {
                recipe_id: row.get(0)?,
                completion_count: row.get(1)?,
                last_confirmed_at: row.get(2)?,
                source: row.get(3)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?;

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
    let mut stmt = conn
        .prepare(
            "SELECT slot, appearance_key, last_confirmed_at
         FROM game_state_equipment WHERE character_name = ?1 AND server_name = ?2
         ORDER BY slot",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![character_name, server_name], |row| {
            Ok(GameStateEquipmentSlot {
                slot: row.get(0)?,
                appearance_key: row.get(1)?,
                last_confirmed_at: row.get(2)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?;

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
    let mut stmt = conn
        .prepare(
            "SELECT npc_key, npc_name, cumulative_delta, favor_tier, last_confirmed_at, source
         FROM game_state_favor WHERE character_name = ?1 AND server_name = ?2
         ORDER BY npc_name",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![character_name, server_name], |row| {
            Ok(GameStateFavor {
                npc_key: row.get(0)?,
                npc_name: row.get(1)?,
                cumulative_delta: row.get(2)?,
                favor_tier: row.get(3)?,
                last_confirmed_at: row.get(4)?,
                source: row.get(5)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?;

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
    let mut stmt = conn
        .prepare(
            "SELECT currency_name, amount, last_confirmed_at, source
         FROM game_state_currencies WHERE character_name = ?1 AND server_name = ?2
         ORDER BY currency_name",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![character_name, server_name], |row| {
            Ok(GameStateCurrency {
                currency_name: row.get(0)?,
                amount: row.get(1)?,
                last_confirmed_at: row.get(2)?,
                source: row.get(3)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?;

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
    let mut stmt = conn
        .prepare(
            "SELECT effect_instance_id, effect_name, source_entity_id, last_confirmed_at
         FROM game_state_effects WHERE character_name = ?1 AND server_name = ?2
         ORDER BY effect_instance_id",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![character_name, server_name], |row| {
            Ok(GameStateEffect {
                effect_instance_id: row.get(0)?,
                effect_name: row.get(1)?,
                source_entity_id: row.get(2)?,
                last_confirmed_at: row.get(3)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?;

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

    let rows = stmt
        .query_map(rusqlite::params![character_name, server_name], |row| {
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
        })
        .map_err(|e| format!("Query error: {e}"))?;

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
    let mut stmt = conn
        .prepare(
            "SELECT skill_name, sort_order FROM tracked_skills
         WHERE character_name = ?1 AND server_name = ?2
         ORDER BY sort_order, skill_name",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![character_name, server_name], |row| {
            Ok(TrackedSkill {
                skill_name: row.get(0)?,
                sort_order: row.get(1)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?;

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
    )
    .map_err(|e| format!("Delete error: {e}"))?;

    let mut stmt = conn
        .prepare(
            "INSERT INTO tracked_skills (character_name, server_name, skill_name, sort_order)
         VALUES (?1, ?2, ?3, ?4)",
        )
        .map_err(|e| format!("Prepare error: {e}"))?;

    for entry in &skills {
        stmt.execute(rusqlite::params![
            character_name,
            server_name,
            entry.skill_name,
            entry.sort_order
        ])
        .map_err(|e| format!("Insert error: {e}"))?;
    }

    Ok(())
}

// ── Gift Log ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct GiftLogEntry {
    pub npc_key: String,
    pub npc_name: String,
    pub gifted_at: String,
    pub favor_delta: f64,
}

#[tauri::command]
pub fn get_gift_log(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<GiftLogEntry>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let mut stmt = conn
        .prepare(
            "SELECT npc_key, npc_name, gifted_at, favor_delta
             FROM game_state_gift_log
             WHERE character_name = ?1 AND server_name = ?2
             ORDER BY gifted_at DESC",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![character_name, server_name], |row| {
            Ok(GiftLogEntry {
                npc_key: row.get(0)?,
                npc_name: row.get(1)?,
                gifted_at: row.get(2)?,
                favor_delta: row.get(3)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))
}

#[tauri::command]
pub fn add_manual_gift(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
    npc_key: String,
    npc_name: String,
) -> Result<(), String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    conn.execute(
        "INSERT INTO game_state_gift_log (character_name, server_name, npc_key, npc_name, gifted_at, favor_delta)
         VALUES (?1, ?2, ?3, ?4, ?5, 0.0)",
        rusqlite::params![character_name, server_name, npc_key, npc_name, now],
    )
    .map_err(|e| format!("Insert error: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn remove_last_gift(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
    npc_key: String,
    week_start: String,
) -> Result<(), String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    conn.execute(
        "DELETE FROM game_state_gift_log WHERE id = (
            SELECT id FROM game_state_gift_log
            WHERE character_name = ?1 AND server_name = ?2 AND npc_key = ?3 AND gifted_at >= ?4
            ORDER BY gifted_at DESC LIMIT 1
        )",
        rusqlite::params![character_name, server_name, npc_key, week_start],
    )
    .map_err(|e| format!("Delete error: {e}"))?;
    Ok(())
}

// ── Vendor State ───────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct GameStateVendor {
    pub npc_key: String,
    pub vendor_gold_available: Option<i64>,
    pub vendor_gold_max: Option<i64>,
    pub vendor_gold_timer_start: Option<String>,
    pub last_interaction_at: Option<String>,
    pub last_sell_at: Option<String>,
    pub last_confirmed_at: String,
}

#[tauri::command]
pub fn get_game_state_vendor(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<GameStateVendor>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let mut stmt = conn
        .prepare(
            "SELECT npc_key, vendor_gold_available, vendor_gold_max, vendor_gold_timer_start, last_interaction_at, last_sell_at, last_confirmed_at
             FROM game_state_npc_vendor WHERE character_name = ?1 AND server_name = ?2
             ORDER BY npc_key",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![character_name, server_name], |row| {
            Ok(GameStateVendor {
                npc_key: row.get(0)?,
                vendor_gold_available: row.get(1)?,
                vendor_gold_max: row.get(2)?,
                vendor_gold_timer_start: row.get(3)?,
                last_interaction_at: row.get(4)?,
                last_sell_at: row.get(5)?,
                last_confirmed_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))
}

// ── Book reports ────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct GameStateBook {
    pub book_type: String,
    pub title: String,
    pub content: String,
    pub captured_at: String,
}

#[tauri::command]
pub fn get_game_state_books(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
    book_type: Option<String>,
) -> Result<Vec<GameStateBook>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;

    if let Some(bt) = book_type {
        let mut stmt = conn
            .prepare(
                "SELECT book_type, title, content, captured_at
                 FROM game_state_books
                 WHERE character_name = ?1 AND server_name = ?2 AND book_type = ?3
                 ORDER BY captured_at DESC",
            )
            .map_err(|e| format!("Query error: {e}"))?;

        let rows = stmt
            .query_map(rusqlite::params![character_name, server_name, bt], |row| {
                Ok(GameStateBook {
                    book_type: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    captured_at: row.get(3)?,
                })
            })
            .map_err(|e| format!("Query error: {e}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Row error: {e}"))
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT book_type, title, content, captured_at
                 FROM game_state_books
                 WHERE character_name = ?1 AND server_name = ?2
                 ORDER BY captured_at DESC",
            )
            .map_err(|e| format!("Query error: {e}"))?;

        let rows = stmt
            .query_map(rusqlite::params![character_name, server_name], |row| {
                Ok(GameStateBook {
                    book_type: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    captured_at: row.get(3)?,
                })
            })
            .map_err(|e| format!("Query error: {e}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Row error: {e}"))
    }
}

// ── Milking timers ──────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct MilkingTimer {
    pub cow_name: String,
    pub zone: String,
    pub last_milked_at: String,
}

#[tauri::command]
pub fn get_milking_timers(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<MilkingTimer>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let mut stmt = conn
        .prepare(
            "SELECT cow_name, zone, last_milked_at
             FROM milking_timers
             WHERE character_name = ?1 AND server_name = ?2
             ORDER BY zone, cow_name",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![character_name, server_name], |row| {
            Ok(MilkingTimer {
                cow_name: row.get(0)?,
                zone: row.get(1)?,
                last_milked_at: row.get(2)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))
}

// ── Character stats ─────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct CharacterReportStat {
    pub category: String,
    pub stat_name: String,
    pub stat_value: String,
    pub updated_at: String,
}

#[tauri::command]
pub fn get_character_report_stats(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
    category: Option<String>,
) -> Result<Vec<CharacterReportStat>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;

    if let Some(cat) = category {
        let mut stmt = conn
            .prepare(
                "SELECT category, stat_name, stat_value, updated_at
                 FROM character_report_stats
                 WHERE character_name = ?1 AND server_name = ?2 AND category = ?3
                 ORDER BY category, stat_name",
            )
            .map_err(|e| format!("Query error: {e}"))?;

        let rows = stmt
            .query_map(rusqlite::params![character_name, server_name, cat], |row| {
                Ok(CharacterReportStat {
                    category: row.get(0)?,
                    stat_name: row.get(1)?,
                    stat_value: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })
            .map_err(|e| format!("Query error: {e}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Row error: {e}"))
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT category, stat_name, stat_value, updated_at
                 FROM character_report_stats
                 WHERE character_name = ?1 AND server_name = ?2
                 ORDER BY category, stat_name",
            )
            .map_err(|e| format!("Query error: {e}"))?;

        let rows = stmt
            .query_map(rusqlite::params![character_name, server_name], |row| {
                Ok(CharacterReportStat {
                    category: row.get(0)?,
                    stat_name: row.get(1)?,
                    stat_value: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })
            .map_err(|e| format!("Query error: {e}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Row error: {e}"))
    }
}

// ── Player Milking Leaderboard ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct PlayerMilkingEntry {
    pub player_name: String,
    pub count: i64,
}

#[tauri::command]
pub fn get_player_milking_leaderboard(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
    direction: String,
) -> Result<Vec<PlayerMilkingEntry>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let mut stmt = conn
        .prepare(
            "SELECT other_player, COUNT(*) as cnt
             FROM player_milking_log
             WHERE character_name = ?1 AND server_name = ?2 AND direction = ?3
             GROUP BY other_player
             ORDER BY cnt DESC, other_player ASC",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![character_name, server_name, direction], |row| {
            Ok(PlayerMilkingEntry {
                player_name: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))
}

// ── Computed Stats ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SkillCraftingBreakdown {
    pub skill_name: String,
    pub total_crafted: i64,
    pub crafting_seconds: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AttributeExtreme {
    pub attribute_name: String,
    pub current_value: f64,
    pub min_value: f64,
    pub max_value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComputedStats {
    pub total_level: i64,
    pub total_base_level: i64,
    pub total_bonus_levels: i64,
    pub skill_count: i64,
    pub total_xp_earned: i64,
    pub items_crafted: i64,
    pub items_distilled: i64,
    pub items_deconstructed: i64,
    pub times_teleported: i64,
    pub items_dyed: i64,
    pub total_crafting_seconds: f64,
    // Rate stats (None if time_played not available from /age report)
    pub hours_played: Option<f64>,
    pub xp_per_hour: Option<f64>,
    pub kills_per_hour: Option<f64>,
    pub deaths_per_hour: Option<f64>,
    // Per-skill crafting breakdowns
    pub crafting_by_skill: Vec<SkillCraftingBreakdown>,
}

/// Parse a "X days Y hours" string into total hours.
/// Handles formats like: "40 days 13 hours", "5 hours", "120 days", "1 days 0 hours"
fn parse_time_played_hours(value: &str) -> Option<f64> {
    let mut total_hours: f64 = 0.0;
    let parts: Vec<&str> = value.split_whitespace().collect();
    let mut i = 0;
    while i + 1 < parts.len() {
        if let Ok(num) = parts[i].parse::<f64>() {
            let unit = parts[i + 1].to_lowercase();
            if unit.starts_with("day") {
                total_hours += num * 24.0;
            } else if unit.starts_with("hour") {
                total_hours += num;
            } else if unit.starts_with("minute") {
                total_hours += num / 60.0;
            }
            i += 2;
        } else {
            i += 1;
        }
    }
    if total_hours > 0.0 { Some(total_hours) } else { None }
}

#[tauri::command]
pub fn get_computed_stats(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<ComputedStats, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;

    // ── Skill totals ────────────────────────────────────────────────
    let (total_level, total_base_level, total_bonus_levels, skill_count, total_xp_earned): (
        i64, i64, i64, i64, i64,
    ) = conn
        .query_row(
            "SELECT
                COALESCE(SUM(level + bonus_levels), 0),
                COALESCE(SUM(level), 0),
                COALESCE(SUM(bonus_levels), 0),
                COUNT(*),
                COALESCE(SUM(xp), 0)
             FROM game_state_skills
             WHERE character_name = ?1 AND server_name = ?2",
            rusqlite::params![character_name, server_name],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .map_err(|e| format!("Skill totals query error: {e}"))?;

    // ── Recipe-based stats ──────────────────────────────────────────

    // Items crafted: recipes whose skill is NOT a utility/knowledge skill
    let items_crafted: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(gr.completion_count), 0)
             FROM game_state_recipes gr
             JOIN recipes r ON r.id = gr.recipe_id
             WHERE gr.character_name = ?1 AND gr.server_name = ?2
               AND r.skill IS NOT NULL
               AND r.skill NOT IN ('Lore', 'Teleportation', 'Augmentation', 'Transmutation')",
            rusqlite::params![character_name, server_name],
            |row| row.get(0),
        )
        .map_err(|e| format!("Items crafted query error: {e}"))?;

    // Items distilled: Transmutation recipes that aren't repair crafts
    let items_distilled: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(gr.completion_count), 0)
             FROM game_state_recipes gr
             JOIN recipes r ON r.id = gr.recipe_id
             WHERE gr.character_name = ?1 AND gr.server_name = ?2
               AND r.skill = 'Transmutation'
               AND COALESCE(r.name, '') NOT LIKE '%Repair%'
               AND COALESCE(r.action_label, '') NOT LIKE '%Repair%'",
            rusqlite::params![character_name, server_name],
            |row| row.get(0),
        )
        .map_err(|e| format!("Items distilled query error: {e}"))?;

    // Items deconstructed: recipes with internal names like "DecomposeHead", "DecomposeChest", etc.
    let items_deconstructed: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(gr.completion_count), 0)
             FROM game_state_recipes gr
             JOIN recipes r ON r.id = gr.recipe_id
             WHERE gr.character_name = ?1 AND gr.server_name = ?2
               AND COALESCE(json_extract(r.raw_json, '$.InternalName'), '') LIKE 'Decompose%'",
            rusqlite::params![character_name, server_name],
            |row| row.get(0),
        )
        .map_err(|e| format!("Items deconstructed query error: {e}"))?;

    // Times teleported: Teleportation recipes that aren't binding-related
    let times_teleported: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(gr.completion_count), 0)
             FROM game_state_recipes gr
             JOIN recipes r ON r.id = gr.recipe_id
             WHERE gr.character_name = ?1 AND gr.server_name = ?2
               AND r.skill = 'Teleportation'
               AND COALESCE(r.name, '') NOT LIKE '%Bind%'
               AND COALESCE(r.name, '') NOT LIKE '%Binding%'",
            rusqlite::params![character_name, server_name],
            |row| row.get(0),
        )
        .map_err(|e| format!("Times teleported query error: {e}"))?;

    // Items dyed: recipes with internal names like "DyeLeatherArmor1", "DyeMetalArmor2", etc.
    let items_dyed: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(gr.completion_count), 0)
             FROM game_state_recipes gr
             JOIN recipes r ON r.id = gr.recipe_id
             WHERE gr.character_name = ?1 AND gr.server_name = ?2
               AND COALESCE(json_extract(r.raw_json, '$.InternalName'), '') LIKE 'Dye%Armor%'",
            rusqlite::params![character_name, server_name],
            |row| row.get(0),
        )
        .map_err(|e| format!("Items dyed query error: {e}"))?;

    // Total crafting time: sum of (completion_count * usage_delay) for all recipes
    let total_crafting_seconds: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(gr.completion_count * COALESCE(r.usage_delay, 0)), 0.0)
             FROM game_state_recipes gr
             JOIN recipes r ON r.id = gr.recipe_id
             WHERE gr.character_name = ?1 AND gr.server_name = ?2",
            rusqlite::params![character_name, server_name],
            |row| row.get(0),
        )
        .map_err(|e| format!("Crafting time query error: {e}"))?;

    // ── Per-skill crafting breakdowns ───────────────────────────────
    let crafting_by_skill = {
        let mut stmt = conn
            .prepare(
                "SELECT r.skill, SUM(gr.completion_count), SUM(gr.completion_count * COALESCE(r.usage_delay, 0))
                 FROM game_state_recipes gr
                 JOIN recipes r ON r.id = gr.recipe_id
                 WHERE gr.character_name = ?1 AND gr.server_name = ?2
                   AND r.skill IS NOT NULL
                 GROUP BY r.skill
                 ORDER BY SUM(gr.completion_count) DESC",
            )
            .map_err(|e| format!("Crafting breakdown query error: {e}"))?;

        let rows = stmt
            .query_map(rusqlite::params![character_name, server_name], |row| {
                Ok(SkillCraftingBreakdown {
                    skill_name: row.get(0)?,
                    total_crafted: row.get(1)?,
                    crafting_seconds: row.get(2)?,
                })
            })
            .map_err(|e| format!("Crafting breakdown query error: {e}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Crafting breakdown row error: {e}"))?
    };

    // ── Rate stats from report stats ────────────────────────────────
    // Look up time_played, kills, deaths from character_report_stats (from /age report)
    let hours_played = {
        let time_str: Option<String> = conn
            .query_row(
                "SELECT stat_value FROM character_report_stats
                 WHERE character_name = ?1 AND server_name = ?2
                   AND category = 'age' AND stat_name = 'time_played'",
                rusqlite::params![character_name, server_name],
                |row| row.get(0),
            )
            .ok();
        time_str.and_then(|s| parse_time_played_hours(&s))
    };

    let report_kills: Option<f64> = conn
        .query_row(
            "SELECT stat_value FROM character_report_stats
             WHERE character_name = ?1 AND server_name = ?2
               AND category = 'age' AND stat_name = 'kills'",
            rusqlite::params![character_name, server_name],
            |row| {
                let val: String = row.get(0)?;
                Ok(val.replace(',', "").parse::<f64>().ok())
            },
        )
        .ok()
        .flatten();

    let report_deaths: Option<f64> = conn
        .query_row(
            "SELECT stat_value FROM character_report_stats
             WHERE character_name = ?1 AND server_name = ?2
               AND category = 'age' AND stat_name = 'deaths'",
            rusqlite::params![character_name, server_name],
            |row| {
                let val: String = row.get(0)?;
                Ok(val.replace(',', "").parse::<f64>().ok())
            },
        )
        .ok()
        .flatten();

    let xp_per_hour = hours_played.map(|h| total_xp_earned as f64 / h);
    let kills_per_hour = match (hours_played, report_kills) {
        (Some(h), Some(k)) if h > 0.0 => Some(k / h),
        _ => None,
    };
    let deaths_per_hour = match (hours_played, report_deaths) {
        (Some(h), Some(d)) if h > 0.0 => Some(d / h),
        _ => None,
    };

    Ok(ComputedStats {
        total_level,
        total_base_level,
        total_bonus_levels,
        skill_count,
        total_xp_earned,
        items_crafted,
        items_distilled,
        items_deconstructed,
        times_teleported,
        items_dyed,
        total_crafting_seconds,
        hours_played,
        xp_per_hour,
        kills_per_hour,
        deaths_per_hour,
        crafting_by_skill,
    })
}

#[tauri::command]
pub fn get_attribute_extremes(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<AttributeExtreme>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let mut stmt = conn
        .prepare(
            "SELECT attribute_name, value, COALESCE(min_value, value), COALESCE(max_value, value)
             FROM game_state_attributes
             WHERE character_name = ?1 AND server_name = ?2
             ORDER BY attribute_name",
        )
        .map_err(|e| format!("Attribute extremes query error: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![character_name, server_name], |row| {
            Ok(AttributeExtreme {
                attribute_name: row.get(0)?,
                current_value: row.get(1)?,
                min_value: row.get(2)?,
                max_value: row.get(3)?,
            })
        })
        .map_err(|e| format!("Attribute extremes query error: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Attribute extremes row error: {e}"))
}

// ── Teleportation bind queries ───────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct TeleportationBinds {
    pub primary_bind: Option<String>,
    pub secondary_bind: Option<String>,
    pub mushroom_circle_1: Option<String>,
    pub mushroom_circle_2: Option<String>,
    pub last_updated: Option<String>,
}

/// Get the player's known teleportation bind locations.
#[tauri::command]
pub async fn get_teleportation_binds(
    character: String,
    server: String,
    db: State<'_, DbPool>,
) -> Result<TeleportationBinds, String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT primary_bind, secondary_bind, mushroom_circle_1, mushroom_circle_2, last_updated
             FROM game_state_teleportation
             WHERE character_name = ?1 AND server_name = ?2",
        )
        .map_err(|e| e.to_string())?;

    let result = stmt
        .query_row(rusqlite::params![character, server], |row| {
            Ok(TeleportationBinds {
                primary_bind: row.get(0)?,
                secondary_bind: row.get(1)?,
                mushroom_circle_1: row.get(2)?,
                mushroom_circle_2: row.get(3)?,
                last_updated: row.get(4)?,
            })
        })
        .ok();

    Ok(result.unwrap_or(TeleportationBinds {
        primary_bind: None,
        secondary_bind: None,
        mushroom_circle_1: None,
        mushroom_circle_2: None,
        last_updated: None,
    }))
}

/// Manually set mushroom circle attunements (can't be parsed from logs).
#[tauri::command]
pub async fn set_mushroom_circles(
    character: String,
    server: String,
    circle_1: Option<String>,
    circle_2: Option<String>,
    db: State<'_, DbPool>,
) -> Result<(), String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    let dt = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO game_state_teleportation
            (character_name, server_name, mushroom_circle_1, mushroom_circle_2, last_updated)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(character_name, server_name) DO UPDATE SET
            mushroom_circle_1 = excluded.mushroom_circle_1,
            mushroom_circle_2 = excluded.mushroom_circle_2,
            last_updated = excluded.last_updated",
        rusqlite::params![character, server, circle_1, circle_2, dt],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_played_hours() {
        assert_eq!(parse_time_played_hours("40 days 13 hours"), Some(40.0 * 24.0 + 13.0));
        assert_eq!(parse_time_played_hours("5 hours"), Some(5.0));
        assert_eq!(parse_time_played_hours("120 days"), Some(120.0 * 24.0));
        assert_eq!(parse_time_played_hours("1 days 0 hours"), Some(24.0));
        assert_eq!(parse_time_played_hours("0 days 0 hours"), None); // 0 total → None
        assert_eq!(parse_time_played_hours(""), None);
        assert_eq!(parse_time_played_hours("garbage"), None);
        assert_eq!(parse_time_played_hours("71 days"), Some(71.0 * 24.0));
    }
}
