use super::DbPool;
use serde::{Deserialize, Serialize};
/// Survey event logging commands
use tauri::State;

#[derive(Deserialize)]
pub struct LogSurveyEventInput {
    pub timestamp: String,
    pub session_id: Option<i64>,
    pub event_type: String, // "session_start", "completed"
    pub map_type: Option<String>,
    pub survey_type: Option<String>,
    pub speed_bonus_earned: bool,
}

#[tauri::command]
pub fn log_survey_event(db: State<'_, DbPool>, input: LogSurveyEventInput) -> Result<i64, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

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
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

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
                 LIMIT ?2",
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
                 LIMIT ?1",
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
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

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
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

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
                 LIMIT ?2",
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
                 LIMIT ?1",
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
    include_imports: Option<bool>,
) -> Result<SpeedBonusStats, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let include = include_imports.unwrap_or(false);

    let (total_surveys, speed_bonus_count): (i64, i64) = if let Some(sid) = session_id {
        conn.query_row(
            "SELECT
                COUNT(*) as total,
                SUM(CASE WHEN speed_bonus_earned = 1 THEN 1 ELSE 0 END) as bonus_count
             FROM survey_events
             WHERE event_type IN ('completed', 'motherlode_completed') AND session_id = ?1",
            [sid],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
    } else {
        conn.query_row(
            "SELECT
                COUNT(*) as total,
                SUM(CASE WHEN se.speed_bonus_earned = 1 THEN 1 ELSE 0 END) as bonus_count
             FROM survey_events se
             JOIN survey_session_stats sss ON se.session_id = sss.id
             WHERE se.event_type IN ('completed', 'motherlode_completed')
               AND (?1 = 1 OR sss.import_id IS NULL)",
            [include as i64],
            |row| Ok((row.get(0)?, row.get(1)?)),
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
            |row| Ok((row.get(0).unwrap_or(0), row.get(1).unwrap_or(0))),
        )
    } else {
        conn.query_row(
            "SELECT
                SUM(sli.quantity) as total_items,
                COUNT(DISTINCT sli.item_name) as unique_items
             FROM survey_loot_items sli
             JOIN survey_events se ON sli.event_id = se.id
             JOIN survey_session_stats sss ON se.session_id = sss.id
             WHERE sli.is_speed_bonus = 1
               AND (?1 = 1 OR sss.import_id IS NULL)",
            [include as i64],
            |row| Ok((row.get(0).unwrap_or(0), row.get(1).unwrap_or(0))),
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
    pub vendor_value: f64,
}

#[tauri::command]
pub fn get_loot_breakdown(
    db: State<'_, DbPool>,
    session_id: Option<i64>,
    limit: Option<usize>,
) -> Result<Vec<LootBreakdownEntry>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

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
                    COUNT(*) as times_received,
                    COALESCE(CAST(i.value AS REAL), 0.0) as vendor_value
                 FROM survey_loot_items sli
                 JOIN survey_events se ON sli.event_id = se.id
                 LEFT JOIN items i ON sli.item_name = i.name COLLATE NOCASE
                 WHERE se.session_id = ?1
                 GROUP BY sli.item_name, sli.item_id
                 ORDER BY total_qty DESC
                 LIMIT ?2",
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
                    vendor_value: row.get(6)?,
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
                    COUNT(*) as times_received,
                    COALESCE(CAST(i.value AS REAL), 0.0) as vendor_value
                 FROM survey_loot_items sli
                 LEFT JOIN items i ON sli.item_name = i.name COLLATE NOCASE
                 GROUP BY sli.item_name, sli.item_id
                 ORDER BY total_qty DESC
                 LIMIT ?1",
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
                    vendor_value: row.get(6)?,
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
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let mut results = Vec::new();

    // Use two separate queries to avoid row multiplication from LEFT JOIN:
    // 1. Event-level stats (completed count, speed bonus count) from survey_events
    // 2. Loot-level stats (total items, bonus items) from survey_loot_items via events
    let session_filter = if session_id.is_some() {
        "AND se.session_id = ?1"
    } else {
        ""
    };

    // Event-level stats
    let event_query = format!(
        "SELECT se.survey_type,
                COUNT(*) as total_completed,
                SUM(CASE WHEN se.speed_bonus_earned = 1 THEN 1 ELSE 0 END) as speed_bonus_count
         FROM survey_events se
         WHERE se.event_type IN ('completed', 'motherlode_completed') AND se.survey_type IS NOT NULL {session_filter}
         GROUP BY se.survey_type
         ORDER BY total_completed DESC"
    );
    let mut event_stmt = conn
        .prepare(&event_query)
        .map_err(|e| format!("Failed to prepare survey type metrics query: {e}"))?;

    let mut event_rows: Vec<(String, i64, i64)> = Vec::new();
    {
        let map_row = |row: &rusqlite::Row| -> rusqlite::Result<(String, i64, i64)> {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        };
        let rows = if let Some(sid) = session_id {
            event_stmt.query_map([sid], map_row)
        } else {
            event_stmt.query_map([], map_row)
        }
        .map_err(|e| format!("Survey type metrics query failed: {e}"))?;
        for r in rows {
            if let Ok(row) = r {
                event_rows.push(row);
            }
        }
    }

    // Loot-level stats
    let loot_query = format!(
        "SELECT se.survey_type,
                COALESCE(SUM(sli.quantity), 0) as total_items,
                COALESCE(SUM(CASE WHEN sli.is_speed_bonus = 1 THEN sli.quantity ELSE 0 END), 0) as total_bonus_items
         FROM survey_events se
         JOIN survey_loot_items sli ON sli.event_id = se.id
         WHERE se.event_type IN ('completed', 'motherlode_completed') AND se.survey_type IS NOT NULL {session_filter}
         GROUP BY se.survey_type"
    );
    let mut loot_stmt = conn
        .prepare(&loot_query)
        .map_err(|e| format!("Failed to prepare loot metrics query: {e}"))?;

    let mut loot_map: std::collections::HashMap<String, (i64, i64)> =
        std::collections::HashMap::new();
    {
        let map_row = |row: &rusqlite::Row| -> rusqlite::Result<(String, i64, i64)> {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        };
        let rows = if let Some(sid) = session_id {
            loot_stmt.query_map([sid], map_row)
        } else {
            loot_stmt.query_map([], map_row)
        }
        .map_err(|e| format!("Loot metrics query failed: {e}"))?;
        for r in rows {
            if let Ok((st, total, bonus)) = r {
                loot_map.insert(st, (total, bonus));
            }
        }
    }

    for (survey_type, total_completed, speed_bonus_count) in event_rows {
        let (total_items, total_bonus_items) =
            loot_map.get(&survey_type).copied().unwrap_or((0, 0));

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

        results.push(SurveyTypeMetrics {
            survey_type,
            total_completed,
            speed_bonus_count,
            speed_bonus_rate,
            total_items,
            total_bonus_items,
            avg_items_per_survey,
        });
    }

    Ok(results)
}

// ── Zone-Based Analytics ─────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct SpeedBonusItemStats {
    pub item_name: String,
    pub total_quantity: i64,
    pub times_seen: i64,
    pub total_procs: i64,
    pub min_per_proc: i64,
    pub max_per_proc: i64,
    pub avg_per_proc: f64,
}

#[derive(Serialize)]
pub struct CategorySpeedBonusStats {
    pub category: String,
    pub total_surveys: i64,
    pub speed_bonus_count: i64,
    pub speed_bonus_rate: f64,
    pub avg_bonus_value: f64,
    pub item_stats: Vec<SpeedBonusItemStats>,
}

#[derive(Serialize)]
pub struct SurveyItemStats {
    pub item_name: String,
    pub total_quantity: i64,
    pub times_seen: i64,
    pub min_per_completion: i64,
    pub max_per_completion: i64,
    pub avg_per_completion: f64,
}

#[derive(Serialize)]
pub struct SurveyTypeAnalytics {
    pub survey_type: String,
    pub category: String,
    pub crafting_cost: f64,
    pub total_completed: i64,
    pub item_stats: Vec<SurveyItemStats>,
}

#[derive(Serialize)]
pub struct ZoneAnalytics {
    pub zone: String,
    pub speed_bonus_stats: Vec<CategorySpeedBonusStats>,
    pub survey_type_stats: Vec<SurveyTypeAnalytics>,
}

#[tauri::command]
pub fn get_zone_analytics(
    db: State<'_, DbPool>,
    include_imports: Option<bool>,
) -> Result<Vec<ZoneAnalytics>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let include = include_imports.unwrap_or(false) as i64;

    // ── 1. Per-zone+category event-level stats ───────────────────────────────
    let mut zone_cat_stats: std::collections::HashMap<(String, String), (i64, i64)> =
        std::collections::HashMap::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT st.zone, st.survey_category,
                    COUNT(*) as total_completed,
                    SUM(CASE WHEN se.speed_bonus_earned = 1 THEN 1 ELSE 0 END) as bonus_count
             FROM survey_events se
             JOIN survey_types st ON se.survey_type = st.name COLLATE NOCASE
             JOIN survey_session_stats sss ON se.session_id = sss.id
             WHERE se.event_type IN ('completed', 'motherlode_completed') AND st.zone IS NOT NULL
               AND (?1 = 1 OR sss.import_id IS NULL)
             GROUP BY st.zone, st.survey_category",
            )
            .map_err(|e| format!("Failed to prepare zone stats query: {e}"))?;

        let rows = stmt
            .query_map([include], |row| {
                let zone: String = row.get(0)?;
                let cat: String = row.get(1)?;
                let total: i64 = row.get(2)?;
                let bonus: i64 = row.get(3)?;
                Ok((zone, cat, total, bonus))
            })
            .map_err(|e| format!("Zone stats query failed: {e}"))?;

        for r in rows {
            if let Ok((zone, cat, total, bonus)) = r {
                zone_cat_stats.insert((zone, cat), (total, bonus));
            }
        }
    }

    // ── 2. Speed bonus item stats per zone+category ──────────────────────────
    let mut bonus_item_stats: std::collections::HashMap<
        (String, String),
        Vec<SpeedBonusItemStats>,
    > = std::collections::HashMap::new();
    {
        let mut stmt = conn
            .prepare(
                "WITH per_proc AS (
                SELECT se.id as event_id, st.zone, st.survey_category, sli.item_name,
                       SUM(sli.quantity) as qty
                FROM survey_loot_items sli
                JOIN survey_events se ON sli.event_id = se.id
                JOIN survey_types st ON se.survey_type = st.name COLLATE NOCASE
                JOIN survey_session_stats sss ON se.session_id = sss.id
                WHERE sli.is_speed_bonus = 1
                  AND se.event_type IN ('completed', 'motherlode_completed')
                  AND st.zone IS NOT NULL
                  AND (?1 = 1 OR sss.import_id IS NULL)
                GROUP BY se.id, st.zone, st.survey_category, sli.item_name
            )
            SELECT zone, survey_category, item_name,
                   SUM(qty) as total_qty,
                   COUNT(*) as times_seen,
                   MIN(qty) as min_qty,
                   MAX(qty) as max_qty,
                   AVG(qty) as avg_qty
            FROM per_proc
            GROUP BY zone, survey_category, item_name
            ORDER BY zone, survey_category, total_qty DESC",
            )
            .map_err(|e| format!("Failed to prepare bonus item stats query: {e}"))?;

        let rows = stmt
            .query_map([include], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, f64>(7)?,
                ))
            })
            .map_err(|e| format!("Bonus item stats query failed: {e}"))?;

        for r in rows {
            if let Ok((
                zone,
                cat,
                item_name,
                total_quantity,
                times_seen,
                min_per_proc,
                max_per_proc,
                avg_per_proc,
            )) = r
            {
                bonus_item_stats
                    .entry((zone, cat))
                    .or_default()
                    .push(SpeedBonusItemStats {
                        item_name,
                        total_quantity,
                        times_seen,
                        total_procs: 0, // filled in during assembly
                        min_per_proc,
                        max_per_proc,
                        avg_per_proc,
                    });
            }
        }
    }

    // ── 3. Average bonus value per proc per zone+category ────────────────────
    let mut avg_bonus_values: std::collections::HashMap<(String, String), f64> =
        std::collections::HashMap::new();
    {
        let mut stmt = conn
            .prepare(
                "WITH bonus_event_values AS (
                SELECT se.id as event_id, st.zone, st.survey_category,
                       SUM(sli.quantity * COALESCE(i.value, 0)) as bonus_value
                FROM survey_events se
                JOIN survey_types st ON se.survey_type = st.name COLLATE NOCASE
                JOIN survey_loot_items sli ON sli.event_id = se.id
                LEFT JOIN items i ON sli.item_name = i.name COLLATE NOCASE
                JOIN survey_session_stats sss ON se.session_id = sss.id
                WHERE se.event_type IN ('completed', 'motherlode_completed')
                  AND se.speed_bonus_earned = 1
                  AND sli.is_speed_bonus = 1
                  AND st.zone IS NOT NULL
                  AND (?1 = 1 OR sss.import_id IS NULL)
                GROUP BY se.id, st.zone, st.survey_category
            )
            SELECT zone, survey_category, AVG(bonus_value) as avg_value
            FROM bonus_event_values
            GROUP BY zone, survey_category",
            )
            .map_err(|e| format!("Failed to prepare avg bonus value query: {e}"))?;

        let rows = stmt
            .query_map([include], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, f64>(2)?,
                ))
            })
            .map_err(|e| format!("Avg bonus value query failed: {e}"))?;

        for r in rows {
            if let Ok((zone, cat, avg_value)) = r {
                avg_bonus_values.insert((zone, cat), avg_value);
            }
        }
    }

    // ── 4. Per-survey-type completion stats ───────────────────────────────────
    let mut type_stats: std::collections::HashMap<String, Vec<SurveyTypeAnalytics>> =
        std::collections::HashMap::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT st.zone, st.survey_category, se.survey_type,
                    COALESCE(st.crafting_cost, 0) as crafting_cost,
                    COUNT(*) as total_completed
             FROM survey_events se
             JOIN survey_types st ON se.survey_type = st.name COLLATE NOCASE
             JOIN survey_session_stats sss ON se.session_id = sss.id
             WHERE se.event_type IN ('completed', 'motherlode_completed') AND st.zone IS NOT NULL
               AND (?1 = 1 OR sss.import_id IS NULL)
             GROUP BY st.zone, st.survey_category, se.survey_type
             ORDER BY st.zone, st.survey_category, total_completed DESC",
            )
            .map_err(|e| format!("Failed to prepare type stats query: {e}"))?;

        let rows = stmt
            .query_map([include], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, f64>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })
            .map_err(|e| format!("Type stats query failed: {e}"))?;

        for r in rows {
            if let Ok((zone, cat, survey_type, crafting_cost, total_completed)) = r {
                type_stats
                    .entry(zone)
                    .or_default()
                    .push(SurveyTypeAnalytics {
                        survey_type,
                        category: cat,
                        crafting_cost,
                        total_completed,
                        item_stats: Vec::new(),
                    });
            }
        }
    }

    // ── 5. Per-survey-type item stats (min/max/avg per completion) ────────────
    {
        let mut stmt = conn
            .prepare(
                "WITH per_completion AS (
                SELECT se.id as event_id, se.survey_type, sli.item_name,
                       SUM(sli.quantity) as qty
                FROM survey_loot_items sli
                JOIN survey_events se ON sli.event_id = se.id
                JOIN survey_types st ON se.survey_type = st.name COLLATE NOCASE
                JOIN survey_session_stats sss ON se.session_id = sss.id
                WHERE se.event_type IN ('completed', 'motherlode_completed')
                  AND sli.is_primary = 1
                  AND st.zone IS NOT NULL
                  AND (?1 = 1 OR sss.import_id IS NULL)
                GROUP BY se.id, se.survey_type, sli.item_name
            )
            SELECT survey_type, item_name,
                   SUM(qty) as total_qty,
                   COUNT(*) as times_seen,
                   MIN(qty) as min_qty,
                   MAX(qty) as max_qty,
                   AVG(qty) as avg_qty
            FROM per_completion
            GROUP BY survey_type, item_name
            ORDER BY survey_type, total_qty DESC",
            )
            .map_err(|e| format!("Failed to prepare type item stats query: {e}"))?;

        let mut type_item_map: std::collections::HashMap<String, Vec<SurveyItemStats>> =
            std::collections::HashMap::new();

        let rows = stmt
            .query_map([include], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, f64>(6)?,
                ))
            })
            .map_err(|e| format!("Type item stats query failed: {e}"))?;

        for r in rows {
            if let Ok((
                survey_type,
                item_name,
                total_quantity,
                times_seen,
                min_per_completion,
                max_per_completion,
                avg_per_completion,
            )) = r
            {
                type_item_map
                    .entry(survey_type)
                    .or_default()
                    .push(SurveyItemStats {
                        item_name,
                        total_quantity,
                        times_seen,
                        min_per_completion,
                        max_per_completion,
                        avg_per_completion,
                    });
            }
        }

        // Attach item stats to their survey types
        for types in type_stats.values_mut() {
            for st in types.iter_mut() {
                if let Some(items) = type_item_map.remove(&st.survey_type) {
                    st.item_stats = items;
                }
            }
        }
    }

    // ── 6. Assemble zone-level results ───────────────────────────────────────
    let mut all_zones: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for (zone, _cat) in zone_cat_stats.keys() {
        all_zones.insert(zone.clone());
    }
    for zone in type_stats.keys() {
        all_zones.insert(zone.clone());
    }

    let mut results: Vec<ZoneAnalytics> = Vec::new();

    for zone in all_zones {
        let mut speed_bonus_stats: Vec<CategorySpeedBonusStats> = Vec::new();
        for cat in &["mineral", "mining"] {
            let key = (zone.clone(), cat.to_string());
            if let Some(&(total_surveys, speed_bonus_count)) = zone_cat_stats.get(&key) {
                let speed_bonus_rate = if total_surveys > 0 {
                    (speed_bonus_count as f64 / total_surveys as f64) * 100.0
                } else {
                    0.0
                };
                let avg_bonus_value = avg_bonus_values.get(&key).copied().unwrap_or(0.0);

                let mut items = bonus_item_stats.remove(&key).unwrap_or_default();
                for item in &mut items {
                    item.total_procs = speed_bonus_count;
                }

                speed_bonus_stats.push(CategorySpeedBonusStats {
                    category: cat.to_string(),
                    total_surveys,
                    speed_bonus_count,
                    speed_bonus_rate,
                    avg_bonus_value,
                    item_stats: items,
                });
            }
        }

        let survey_type_stats = type_stats.remove(&zone).unwrap_or_default();

        results.push(ZoneAnalytics {
            zone,
            speed_bonus_stats,
            survey_type_stats,
        });
    }

    Ok(results)
}

// ── Item Cost Analysis (for calculator + efficiency comparison) ──────────────

#[derive(Serialize)]
pub struct ItemSourceAnalysis {
    pub item_name: String,
    pub survey_type: String,
    pub zone: String,
    pub category: String,
    pub crafting_cost: f64,
    pub total_completions: i64,
    pub primary_total_qty: i64,
    pub primary_times_seen: i64,
    pub primary_avg_per_completion: f64,
    pub bonus_total_qty: i64,
    pub bonus_times_seen: i64,
    pub bonus_avg_per_proc: f64,
    pub speed_bonus_rate: f64,
    pub avg_seconds_per_survey: f64,
}

#[tauri::command]
pub fn get_item_cost_analysis(
    db: State<'_, DbPool>,
    include_imports: Option<bool>,
) -> Result<Vec<ItemSourceAnalysis>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let include = include_imports.unwrap_or(false) as i64;

    // ── 1. Per-survey-type completion + speed bonus counts ───────────────────
    let mut type_counts: std::collections::HashMap<String, (i64, i64, String, String, f64)> =
        std::collections::HashMap::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT se.survey_type,
                        COUNT(*) as total_completed,
                        SUM(CASE WHEN se.speed_bonus_earned = 1 THEN 1 ELSE 0 END) as bonus_count,
                        st.zone,
                        st.survey_category,
                        COALESCE(st.crafting_cost, 0) as crafting_cost
                 FROM survey_events se
                 JOIN survey_types st ON se.survey_type = st.name COLLATE NOCASE
                 JOIN survey_session_stats sss ON se.session_id = sss.id
                 WHERE se.event_type IN ('completed', 'motherlode_completed')
                   AND st.zone IS NOT NULL
                   AND (?1 = 1 OR sss.import_id IS NULL)
                 GROUP BY se.survey_type",
            )
            .map_err(|e| format!("Failed to prepare type counts query: {e}"))?;

        let rows = stmt
            .query_map([include], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, f64>(5)?,
                ))
            })
            .map_err(|e| format!("Type counts query failed: {e}"))?;

        for r in rows {
            if let Ok((survey_type, total, bonus, zone, cat, cost)) = r {
                type_counts.insert(survey_type, (total, bonus, zone, cat, cost));
            }
        }
    }

    // ── 2. Per-survey-type, per-item primary loot stats ─────────────────────
    let mut primary_loot: std::collections::HashMap<(String, String), (i64, i64, f64)> =
        std::collections::HashMap::new();
    {
        let mut stmt = conn
            .prepare(
                "WITH per_completion AS (
                    SELECT se.survey_type, sli.item_name,
                           SUM(sli.quantity) as qty
                    FROM survey_loot_items sli
                    JOIN survey_events se ON sli.event_id = se.id
                    JOIN survey_types st ON se.survey_type = st.name COLLATE NOCASE
                    JOIN survey_session_stats sss ON se.session_id = sss.id
                    WHERE se.event_type IN ('completed', 'motherlode_completed')
                      AND sli.is_primary = 1
                      AND st.zone IS NOT NULL
                      AND (?1 = 1 OR sss.import_id IS NULL)
                    GROUP BY se.id, se.survey_type, sli.item_name
                )
                SELECT survey_type, item_name,
                       SUM(qty) as total_qty,
                       COUNT(*) as times_seen,
                       AVG(qty) as avg_qty
                FROM per_completion
                GROUP BY survey_type, item_name",
            )
            .map_err(|e| format!("Failed to prepare primary loot query: {e}"))?;

        let rows = stmt
            .query_map([include], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, f64>(4)?,
                ))
            })
            .map_err(|e| format!("Primary loot query failed: {e}"))?;

        for r in rows {
            if let Ok((survey_type, item_name, total_qty, times_seen, avg_qty)) = r {
                primary_loot.insert((survey_type, item_name), (total_qty, times_seen, avg_qty));
            }
        }
    }

    // ── 3. Per-survey-type, per-item speed bonus loot stats ─────────────────
    let mut bonus_loot: std::collections::HashMap<(String, String), (i64, i64, f64)> =
        std::collections::HashMap::new();
    {
        let mut stmt = conn
            .prepare(
                "WITH per_proc AS (
                    SELECT se.survey_type, sli.item_name,
                           SUM(sli.quantity) as qty
                    FROM survey_loot_items sli
                    JOIN survey_events se ON sli.event_id = se.id
                    JOIN survey_types st ON se.survey_type = st.name COLLATE NOCASE
                    JOIN survey_session_stats sss ON se.session_id = sss.id
                    WHERE se.event_type IN ('completed', 'motherlode_completed')
                      AND sli.is_speed_bonus = 1
                      AND st.zone IS NOT NULL
                      AND (?1 = 1 OR sss.import_id IS NULL)
                    GROUP BY se.id, se.survey_type, sli.item_name
                )
                SELECT survey_type, item_name,
                       SUM(qty) as total_qty,
                       COUNT(*) as times_seen,
                       AVG(qty) as avg_qty
                FROM per_proc
                GROUP BY survey_type, item_name",
            )
            .map_err(|e| format!("Failed to prepare bonus loot query: {e}"))?;

        let rows = stmt
            .query_map([include], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, f64>(4)?,
                ))
            })
            .map_err(|e| format!("Bonus loot query failed: {e}"))?;

        for r in rows {
            if let Ok((survey_type, item_name, total_qty, times_seen, avg_qty)) = r {
                bonus_loot.insert((survey_type, item_name), (total_qty, times_seen, avg_qty));
            }
        }
    }

    // ── 4. Average time per survey from session stats ────────────────────────
    let mut avg_time: std::collections::HashMap<String, f64> =
        std::collections::HashMap::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT survey_type,
                        CAST(SUM(elapsed_seconds) AS REAL) / NULLIF(SUM(surveys_completed), 0) as avg_secs
                 FROM (
                     SELECT DISTINCT se.survey_type, sss.id, sss.elapsed_seconds, sss.surveys_completed
                     FROM survey_session_stats sss
                     JOIN survey_events se ON se.session_id = sss.id
                     WHERE se.event_type IN ('completed', 'motherlode_completed')
                       AND sss.elapsed_seconds > 0 AND sss.surveys_completed > 0
                       AND (?1 = 1 OR sss.import_id IS NULL)
                 )
                 GROUP BY survey_type",
            )
            .map_err(|e| format!("Failed to prepare avg time query: {e}"))?;

        let rows = stmt
            .query_map([include], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, f64>(1).unwrap_or(0.0),
                ))
            })
            .map_err(|e| format!("Avg time query failed: {e}"))?;

        for r in rows {
            if let Ok((survey_type, secs)) = r {
                avg_time.insert(survey_type, secs);
            }
        }
    }

    // ── 5. Collect all unique (survey_type, item_name) pairs ────────────────
    let mut all_pairs: std::collections::BTreeSet<(String, String)> =
        std::collections::BTreeSet::new();
    for (key, _) in &primary_loot {
        all_pairs.insert(key.clone());
    }
    for (key, _) in &bonus_loot {
        all_pairs.insert(key.clone());
    }

    // ── 6. Assemble results ─────────────────────────────────────────────────
    let mut results: Vec<ItemSourceAnalysis> = Vec::new();

    for (survey_type, item_name) in all_pairs {
        let Some(&(total_completions, bonus_count, ref zone, ref category, crafting_cost)) =
            type_counts.get(&survey_type)
        else {
            continue;
        };

        let (primary_total_qty, primary_times_seen, primary_avg) = primary_loot
            .get(&(survey_type.clone(), item_name.clone()))
            .copied()
            .unwrap_or((0, 0, 0.0));

        let (bonus_total_qty, bonus_times_seen, bonus_avg) = bonus_loot
            .get(&(survey_type.clone(), item_name.clone()))
            .copied()
            .unwrap_or((0, 0, 0.0));

        let speed_bonus_rate = if total_completions > 0 {
            (bonus_count as f64 / total_completions as f64) * 100.0
        } else {
            0.0
        };

        let avg_seconds = avg_time.get(&survey_type).copied().unwrap_or(0.0);

        results.push(ItemSourceAnalysis {
            item_name,
            survey_type,
            zone: zone.clone(),
            category: category.clone(),
            crafting_cost,
            total_completions,
            primary_total_qty,
            primary_times_seen,
            primary_avg_per_completion: primary_avg,
            bonus_total_qty,
            bonus_times_seen,
            bonus_avg_per_proc: bonus_avg,
            speed_bonus_rate,
            avg_seconds_per_survey: avg_seconds,
        });
    }

    Ok(results)
}
