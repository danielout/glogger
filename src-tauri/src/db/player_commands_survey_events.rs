/// Survey event logging commands

use tauri::State;
use serde::{Deserialize, Serialize};
use super::DbPool;

#[derive(Deserialize)]
pub struct LogSurveyEventInput {
    pub timestamp: String,
    pub session_id: Option<i64>,
    pub event_type: String,  // "session_start", "completed"
    pub map_type: Option<String>,
    pub survey_type: Option<String>,
    pub speed_bonus_earned: bool,
}

#[tauri::command]
pub fn log_survey_event(
    db: State<'_, DbPool>,
    input: LogSurveyEventInput,
) -> Result<i64, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    conn.execute(
        "INSERT INTO survey_events (
            timestamp, session_id, event_type, map_type, survey_type,
            speed_bonus_earned
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            input.timestamp,
            input.session_id,
            input.event_type,
            input.map_type,
            input.survey_type,
            input.speed_bonus_earned,
        ],
    )
    .map_err(|e| format!("Failed to log survey event: {e}"))?;

    Ok(conn.last_insert_rowid())
}

#[derive(Serialize)]
pub struct SurveyEventRecord {
    pub id: i64,
    pub timestamp: String,
    pub session_id: Option<i64>,
    pub event_type: String,
    pub map_type: Option<String>,
    pub survey_type: Option<String>,
    pub speed_bonus_earned: bool,
    pub created_at: String,
}

#[tauri::command]
pub fn get_survey_events(
    db: State<'_, DbPool>,
    session_id: Option<i64>,
    limit: Option<usize>,
) -> Result<Vec<SurveyEventRecord>, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    let limit = limit.unwrap_or(100);
    let mut results = Vec::new();

    if let Some(sid) = session_id {
        let mut stmt = conn
            .prepare(
                "SELECT id, datetime(timestamp) as timestamp, session_id, event_type,
                        map_type, survey_type, speed_bonus_earned,
                        datetime(created_at) as created_at
                 FROM survey_events
                 WHERE session_id = ?1
                 ORDER BY timestamp DESC
                 LIMIT ?2"
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let rows = stmt
            .query_map([sid, limit as i64], |row| {
                Ok(SurveyEventRecord {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    session_id: row.get(2)?,
                    event_type: row.get(3)?,
                    map_type: row.get(4)?,
                    survey_type: row.get(5)?,
                    speed_bonus_earned: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Query failed: {e}"))?;

        for row in rows {
            results.push(row.map_err(|e| format!("Row parse error: {e}"))?);
        }
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT id, datetime(timestamp) as timestamp, session_id, event_type,
                        map_type, survey_type, speed_bonus_earned,
                        datetime(created_at) as created_at
                 FROM survey_events
                 ORDER BY timestamp DESC
                 LIMIT ?1"
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let rows = stmt
            .query_map([limit], |row| {
                Ok(SurveyEventRecord {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    session_id: row.get(2)?,
                    event_type: row.get(3)?,
                    map_type: row.get(4)?,
                    survey_type: row.get(5)?,
                    speed_bonus_earned: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Query failed: {e}"))?;

        for row in rows {
            results.push(row.map_err(|e| format!("Row parse error: {e}"))?);
        }
    }

    Ok(results)
}

// ── Survey Loot Items Commands ────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LogSurveyLootItemInput {
    pub event_id: i64,
    pub item_id: Option<u32>,
    pub item_name: String,
    pub quantity: u32,
    pub is_speed_bonus: bool,
    pub is_primary: bool,
}

#[tauri::command]
pub fn log_survey_loot_item(
    db: State<'_, DbPool>,
    input: LogSurveyLootItemInput,
) -> Result<i64, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    conn.execute(
        "INSERT INTO survey_loot_items (
            event_id, item_id, item_name, quantity, is_speed_bonus, is_primary
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            input.event_id,
            input.item_id,
            input.item_name,
            input.quantity,
            input.is_speed_bonus,
            input.is_primary,
        ],
    )
    .map_err(|e| format!("Failed to log loot item: {e}"))?;

    Ok(conn.last_insert_rowid())
}

#[derive(Serialize)]
pub struct SurveyLootItemRecord {
    pub id: i64,
    pub event_id: i64,
    pub item_id: Option<u32>,
    pub item_name: String,
    pub quantity: u32,
    pub is_speed_bonus: bool,
    pub is_primary: bool,
    pub obtained_at: String,
}

#[tauri::command]
pub fn get_survey_loot_items(
    db: State<'_, DbPool>,
    event_id: Option<i64>,
    limit: Option<usize>,
) -> Result<Vec<SurveyLootItemRecord>, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    let limit = limit.unwrap_or(100);
    let mut results = Vec::new();

    if let Some(eid) = event_id {
        let mut stmt = conn
            .prepare(
                "SELECT id, event_id, item_id, item_name, quantity, is_speed_bonus,
                        is_primary, datetime(obtained_at) as obtained_at
                 FROM survey_loot_items
                 WHERE event_id = ?1
                 ORDER BY is_primary DESC, is_speed_bonus DESC, id ASC
                 LIMIT ?2"
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let rows = stmt
            .query_map([eid, limit as i64], |row| {
                Ok(SurveyLootItemRecord {
                    id: row.get(0)?,
                    event_id: row.get(1)?,
                    item_id: row.get(2)?,
                    item_name: row.get(3)?,
                    quantity: row.get(4)?,
                    is_speed_bonus: row.get(5)?,
                    is_primary: row.get(6)?,
                    obtained_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Query failed: {e}"))?;

        for row in rows {
            results.push(row.map_err(|e| format!("Row parse error: {e}"))?);
        }
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT id, event_id, item_id, item_name, quantity, is_speed_bonus,
                        is_primary, datetime(obtained_at) as obtained_at
                 FROM survey_loot_items
                 ORDER BY obtained_at DESC
                 LIMIT ?1"
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let rows = stmt
            .query_map([limit], |row| {
                Ok(SurveyLootItemRecord {
                    id: row.get(0)?,
                    event_id: row.get(1)?,
                    item_id: row.get(2)?,
                    item_name: row.get(3)?,
                    quantity: row.get(4)?,
                    is_speed_bonus: row.get(5)?,
                    is_primary: row.get(6)?,
                    obtained_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Query failed: {e}"))?;

        for row in rows {
            results.push(row.map_err(|e| format!("Row parse error: {e}"))?);
        }
    }

    Ok(results)
}

// ── Analytics Query Commands ──────────────────────────────────────────────────

#[derive(Serialize)]
pub struct SpeedBonusStats {
    pub total_surveys: i64,
    pub speed_bonus_count: i64,
    pub speed_bonus_rate: f64,
    pub total_bonus_items: i64,
    pub unique_bonus_items: i64,
}

#[tauri::command]
pub fn get_speed_bonus_stats(
    db: State<'_, DbPool>,
    session_id: Option<i64>,
) -> Result<SpeedBonusStats, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    let (total_surveys, speed_bonus_count): (i64, i64) = if let Some(sid) = session_id {
        conn.query_row(
            "SELECT
                COUNT(*) as total,
                SUM(CASE WHEN speed_bonus_earned = 1 THEN 1 ELSE 0 END) as bonus_count
             FROM survey_events
             WHERE event_type = 'completed' AND session_id = ?1",
            [sid],
            |row| Ok((row.get(0)?, row.get(1)?))
        )
    } else {
        conn.query_row(
            "SELECT
                COUNT(*) as total,
                SUM(CASE WHEN speed_bonus_earned = 1 THEN 1 ELSE 0 END) as bonus_count
             FROM survey_events
             WHERE event_type = 'completed'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?))
        )
    }
    .map_err(|e| format!("Failed to query speed bonus stats: {e}"))?;

    let (total_bonus_items, unique_bonus_items): (i64, i64) = if let Some(sid) = session_id {
        conn.query_row(
            "SELECT
                SUM(sli.quantity) as total_items,
                COUNT(DISTINCT sli.item_name) as unique_items
             FROM survey_loot_items sli
             JOIN survey_events se ON sli.event_id = se.id
             WHERE sli.is_speed_bonus = 1 AND se.session_id = ?1",
            [sid],
            |row| Ok((row.get(0).unwrap_or(0), row.get(1).unwrap_or(0)))
        )
    } else {
        conn.query_row(
            "SELECT
                SUM(sli.quantity) as total_items,
                COUNT(DISTINCT sli.item_name) as unique_items
             FROM survey_loot_items sli
             WHERE sli.is_speed_bonus = 1",
            [],
            |row| Ok((row.get(0).unwrap_or(0), row.get(1).unwrap_or(0)))
        )
    }
    .map_err(|e| format!("Failed to query bonus items: {e}"))?;

    let speed_bonus_rate = if total_surveys > 0 {
        (speed_bonus_count as f64 / total_surveys as f64) * 100.0
    } else {
        0.0
    };

    Ok(SpeedBonusStats {
        total_surveys,
        speed_bonus_count,
        speed_bonus_rate,
        total_bonus_items,
        unique_bonus_items,
    })
}

#[derive(Serialize)]
pub struct LootBreakdownEntry {
    pub item_name: String,
    pub item_id: Option<u32>,
    pub total_quantity: i64,
    pub primary_quantity: i64,
    pub bonus_quantity: i64,
    pub times_received: i64,
}

#[tauri::command]
pub fn get_loot_breakdown(
    db: State<'_, DbPool>,
    session_id: Option<i64>,
    limit: Option<usize>,
) -> Result<Vec<LootBreakdownEntry>, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    let limit = limit.unwrap_or(100);
    let mut results = Vec::new();

    if let Some(sid) = session_id {
        let mut stmt = conn
            .prepare(
                "SELECT
                    sli.item_name,
                    sli.item_id,
                    SUM(sli.quantity) as total_qty,
                    SUM(CASE WHEN sli.is_primary = 1 THEN sli.quantity ELSE 0 END) as primary_qty,
                    SUM(CASE WHEN sli.is_speed_bonus = 1 THEN sli.quantity ELSE 0 END) as bonus_qty,
                    COUNT(*) as times_received
                 FROM survey_loot_items sli
                 JOIN survey_events se ON sli.event_id = se.id
                 WHERE se.session_id = ?1
                 GROUP BY sli.item_name, sli.item_id
                 ORDER BY total_qty DESC
                 LIMIT ?2"
            )
            .map_err(|e| format!("Failed to prepare loot breakdown query: {e}"))?;

        let rows = stmt
            .query_map([sid, limit as i64], |row| {
                Ok(LootBreakdownEntry {
                    item_name: row.get(0)?,
                    item_id: row.get(1)?,
                    total_quantity: row.get(2)?,
                    primary_quantity: row.get(3)?,
                    bonus_quantity: row.get(4)?,
                    times_received: row.get(5)?,
                })
            })
            .map_err(|e| format!("Loot breakdown query failed: {e}"))?;

        for row in rows {
            results.push(row.map_err(|e| format!("Row parse error: {e}"))?);
        }
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT
                    sli.item_name,
                    sli.item_id,
                    SUM(sli.quantity) as total_qty,
                    SUM(CASE WHEN sli.is_primary = 1 THEN sli.quantity ELSE 0 END) as primary_qty,
                    SUM(CASE WHEN sli.is_speed_bonus = 1 THEN sli.quantity ELSE 0 END) as bonus_qty,
                    COUNT(*) as times_received
                 FROM survey_loot_items sli
                 GROUP BY sli.item_name, sli.item_id
                 ORDER BY total_qty DESC
                 LIMIT ?1"
            )
            .map_err(|e| format!("Failed to prepare loot breakdown query: {e}"))?;

        let rows = stmt
            .query_map([limit], |row| {
                Ok(LootBreakdownEntry {
                    item_name: row.get(0)?,
                    item_id: row.get(1)?,
                    total_quantity: row.get(2)?,
                    primary_quantity: row.get(3)?,
                    bonus_quantity: row.get(4)?,
                    times_received: row.get(5)?,
                })
            })
            .map_err(|e| format!("Loot breakdown query failed: {e}"))?;

        for row in rows {
            results.push(row.map_err(|e| format!("Row parse error: {e}"))?);
        }
    }

    Ok(results)
}

#[derive(Serialize)]
pub struct SurveyTypeMetrics {
    pub survey_type: String,
    pub total_completed: i64,
    pub speed_bonus_count: i64,
    pub speed_bonus_rate: f64,
    pub total_items: i64,
    pub total_bonus_items: i64,
    pub avg_items_per_survey: f64,
}

#[tauri::command]
pub fn get_survey_type_metrics(
    db: State<'_, DbPool>,
    session_id: Option<i64>,
) -> Result<Vec<SurveyTypeMetrics>, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    let mut results = Vec::new();

    if let Some(sid) = session_id {
        let mut stmt = conn
            .prepare(
                "SELECT
                    se.survey_type,
                    COUNT(*) as total_completed,
                    SUM(CASE WHEN se.speed_bonus_earned = 1 THEN 1 ELSE 0 END) as speed_bonus_count,
                    COUNT(DISTINCT sli.id) as total_items,
                    SUM(CASE WHEN sli.is_speed_bonus = 1 THEN sli.quantity ELSE 0 END) as total_bonus_items
                 FROM survey_events se
                 LEFT JOIN survey_loot_items sli ON sli.event_id = se.id
                 WHERE se.event_type = 'completed' AND se.survey_type IS NOT NULL AND se.session_id = ?1
                 GROUP BY se.survey_type
                 ORDER BY total_completed DESC"
            )
            .map_err(|e| format!("Failed to prepare survey type metrics query: {e}"))?;

        let rows = stmt
            .query_map([sid], |row| {
                let survey_type: String = row.get(0)?;
                let total_completed: i64 = row.get(1)?;
                let speed_bonus_count: i64 = row.get(2)?;
                let total_items: i64 = row.get(3)?;
                let total_bonus_items: i64 = row.get(4)?;

                let speed_bonus_rate = if total_completed > 0 {
                    (speed_bonus_count as f64 / total_completed as f64) * 100.0
                } else {
                    0.0
                };

                let avg_items_per_survey = if total_completed > 0 {
                    total_items as f64 / total_completed as f64
                } else {
                    0.0
                };

                Ok(SurveyTypeMetrics {
                    survey_type,
                    total_completed,
                    speed_bonus_count,
                    speed_bonus_rate,
                    total_items,
                    total_bonus_items,
                    avg_items_per_survey,
                })
            })
            .map_err(|e| format!("Survey type metrics query failed: {e}"))?;

        for row in rows {
            results.push(row.map_err(|e| format!("Row parse error: {e}"))?);
        }
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT
                    se.survey_type,
                    COUNT(*) as total_completed,
                    SUM(CASE WHEN se.speed_bonus_earned = 1 THEN 1 ELSE 0 END) as speed_bonus_count,
                    COUNT(DISTINCT sli.id) as total_items,
                    SUM(CASE WHEN sli.is_speed_bonus = 1 THEN sli.quantity ELSE 0 END) as total_bonus_items
                 FROM survey_events se
                 LEFT JOIN survey_loot_items sli ON sli.event_id = se.id
                 WHERE se.event_type = 'completed' AND se.survey_type IS NOT NULL
                 GROUP BY se.survey_type
                 ORDER BY total_completed DESC"
            )
            .map_err(|e| format!("Failed to prepare survey type metrics query: {e}"))?;

        let rows = stmt
            .query_map([], |row| {
                let survey_type: String = row.get(0)?;
                let total_completed: i64 = row.get(1)?;
                let speed_bonus_count: i64 = row.get(2)?;
                let total_items: i64 = row.get(3)?;
                let total_bonus_items: i64 = row.get(4)?;

                let speed_bonus_rate = if total_completed > 0 {
                    (speed_bonus_count as f64 / total_completed as f64) * 100.0
                } else {
                    0.0
                };

                let avg_items_per_survey = if total_completed > 0 {
                    total_items as f64 / total_completed as f64
                } else {
                    0.0
                };

                Ok(SurveyTypeMetrics {
                    survey_type,
                    total_completed,
                    speed_bonus_count,
                    speed_bonus_rate,
                    total_items,
                    total_bonus_items,
                    avg_items_per_survey,
                })
            })
            .map_err(|e| format!("Survey type metrics query failed: {e}"))?;

        for row in rows {
            results.push(row.map_err(|e| format!("Row parse error: {e}"))?);
        }
    }

    Ok(results)
}
