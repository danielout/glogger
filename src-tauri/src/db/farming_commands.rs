/// Farming session persistence commands

use tauri::State;
use serde::{Deserialize, Serialize};
use super::DbPool;

// ── Input types ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct FarmingSkillInput {
    pub skill_name: String,
    pub xp_gained: i64,
    pub levels_gained: i32,
}

#[derive(Deserialize)]
pub struct FarmingItemInput {
    pub item_name: String,
    pub net_quantity: i32,
}

#[derive(Deserialize)]
pub struct FarmingFavorInput {
    pub npc_name: String,
    pub npc_id: Option<i64>,
    pub delta: f64,
}

#[derive(Deserialize)]
pub struct SaveFarmingSessionInput {
    pub name: String,
    pub notes: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub elapsed_seconds: i64,
    pub total_paused_seconds: i64,
    pub vendor_gold: i64,
    pub skills: Vec<FarmingSkillInput>,
    pub items: Vec<FarmingItemInput>,
    pub favors: Vec<FarmingFavorInput>,
}

// ── Output types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct FarmingSkillRecord {
    pub skill_name: String,
    pub xp_gained: i64,
    pub levels_gained: i32,
}

#[derive(Serialize)]
pub struct FarmingItemRecord {
    pub item_name: String,
    pub net_quantity: i32,
}

#[derive(Serialize)]
pub struct FarmingFavorRecord {
    pub npc_name: String,
    pub npc_id: Option<i64>,
    pub delta: f64,
}

#[derive(Serialize)]
pub struct HistoricalFarmingSession {
    pub id: i64,
    pub name: String,
    pub notes: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub elapsed_seconds: i64,
    pub total_paused_seconds: i64,
    pub vendor_gold: i64,
    pub created_at: String,
    pub skills: Vec<FarmingSkillRecord>,
    pub items: Vec<FarmingItemRecord>,
    pub favors: Vec<FarmingFavorRecord>,
}

// ── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn save_farming_session(
    db: State<'_, DbPool>,
    input: SaveFarmingSessionInput,
) -> Result<i64, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    // Insert session row
    conn.execute(
        "INSERT INTO farming_sessions (name, notes, start_time, end_time, elapsed_seconds, total_paused_seconds, vendor_gold)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            input.name,
            input.notes,
            input.start_time,
            input.end_time,
            input.elapsed_seconds,
            input.total_paused_seconds,
            input.vendor_gold,
        ],
    ).map_err(|e| format!("Failed to save farming session: {e}"))?;

    let session_id = conn.last_insert_rowid();

    // Insert skill records
    for skill in &input.skills {
        conn.execute(
            "INSERT INTO farming_session_skills (session_id, skill_name, xp_gained, levels_gained)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![session_id, skill.skill_name, skill.xp_gained, skill.levels_gained],
        ).map_err(|e| format!("Failed to save farming skill: {e}"))?;
    }

    // Insert item records
    for item in &input.items {
        conn.execute(
            "INSERT INTO farming_session_items (session_id, item_name, net_quantity)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![session_id, item.item_name, item.net_quantity],
        ).map_err(|e| format!("Failed to save farming item: {e}"))?;
    }

    // Insert favor records
    for favor in &input.favors {
        conn.execute(
            "INSERT INTO farming_session_favors (session_id, npc_name, npc_id, delta)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![session_id, favor.npc_name, favor.npc_id, favor.delta],
        ).map_err(|e| format!("Failed to save farming favor: {e}"))?;
    }

    Ok(session_id)
}

#[tauri::command]
pub fn get_farming_sessions(
    db: State<'_, DbPool>,
    limit: Option<usize>,
) -> Result<Vec<HistoricalFarmingSession>, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;
    let limit = limit.unwrap_or(50) as i64;

    // Fetch session rows
    let mut stmt = conn.prepare(
        "SELECT id, name, notes, start_time, end_time, elapsed_seconds,
                total_paused_seconds, vendor_gold, datetime(created_at) as created_at
         FROM farming_sessions
         ORDER BY created_at DESC
         LIMIT ?1"
    ).map_err(|e| format!("Failed to prepare query: {e}"))?;

    let session_rows = stmt.query_map([limit], |row| {
        Ok(HistoricalFarmingSession {
            id: row.get(0)?,
            name: row.get(1)?,
            notes: row.get(2)?,
            start_time: row.get(3)?,
            end_time: row.get(4)?,
            elapsed_seconds: row.get(5)?,
            total_paused_seconds: row.get(6)?,
            vendor_gold: row.get(7)?,
            created_at: row.get(8)?,
            skills: Vec::new(),
            items: Vec::new(),
            favors: Vec::new(),
        })
    }).map_err(|e| format!("Query failed: {e}"))?;

    let mut sessions: Vec<HistoricalFarmingSession> = Vec::new();
    for row in session_rows {
        sessions.push(row.map_err(|e| format!("Row parse error: {e}"))?);
    }

    // Fill in children for each session
    for session in &mut sessions {
        // Skills
        let mut skill_stmt = conn.prepare(
            "SELECT skill_name, xp_gained, levels_gained FROM farming_session_skills WHERE session_id = ?1"
        ).map_err(|e| format!("Failed to prepare skill query: {e}"))?;

        let skill_rows = skill_stmt.query_map([session.id], |row| {
            Ok(FarmingSkillRecord {
                skill_name: row.get(0)?,
                xp_gained: row.get(1)?,
                levels_gained: row.get(2)?,
            })
        }).map_err(|e| format!("Skill query failed: {e}"))?;

        for row in skill_rows {
            session.skills.push(row.map_err(|e| format!("Skill row error: {e}"))?);
        }

        // Items
        let mut item_stmt = conn.prepare(
            "SELECT item_name, net_quantity FROM farming_session_items WHERE session_id = ?1 ORDER BY net_quantity DESC"
        ).map_err(|e| format!("Failed to prepare item query: {e}"))?;

        let item_rows = item_stmt.query_map([session.id], |row| {
            Ok(FarmingItemRecord {
                item_name: row.get(0)?,
                net_quantity: row.get(1)?,
            })
        }).map_err(|e| format!("Item query failed: {e}"))?;

        for row in item_rows {
            session.items.push(row.map_err(|e| format!("Item row error: {e}"))?);
        }

        // Favors
        let mut favor_stmt = conn.prepare(
            "SELECT npc_name, npc_id, delta FROM farming_session_favors WHERE session_id = ?1"
        ).map_err(|e| format!("Failed to prepare favor query: {e}"))?;

        let favor_rows = favor_stmt.query_map([session.id], |row| {
            Ok(FarmingFavorRecord {
                npc_name: row.get(0)?,
                npc_id: row.get(1)?,
                delta: row.get(2)?,
            })
        }).map_err(|e| format!("Favor query failed: {e}"))?;

        for row in favor_rows {
            session.favors.push(row.map_err(|e| format!("Favor row error: {e}"))?);
        }
    }

    Ok(sessions)
}

#[tauri::command]
pub fn update_farming_session(
    db: State<'_, DbPool>,
    session_id: i64,
    name: String,
    notes: String,
) -> Result<(), String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    conn.execute(
        "UPDATE farming_sessions SET name = ?1, notes = ?2 WHERE id = ?3",
        rusqlite::params![name, notes, session_id],
    ).map_err(|e| format!("Failed to update farming session: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn delete_farming_session(
    db: State<'_, DbPool>,
    session_id: i64,
) -> Result<(), String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    conn.execute(
        "DELETE FROM farming_sessions WHERE id = ?1",
        [session_id],
    ).map_err(|e| format!("Failed to delete farming session: {e}"))?;

    Ok(())
}
