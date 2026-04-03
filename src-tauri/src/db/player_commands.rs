use super::{queries, DbPool};
use serde::{Deserialize, Serialize};
/// Tauri commands for player data operations (market prices, sales, surveys, etc.)
use tauri::State;

// ── Market Price Commands ─────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct MarketPriceInput {
    pub item_id: u32,
    pub price: f64,
    pub quantity: u32,
    pub vendor_type: String, // "bazaar", "player_vendor", "work_order"
    pub vendor_name: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn add_market_price(db: State<'_, DbPool>, input: MarketPriceInput) -> Result<i64, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    queries::player_data::insert_market_price(
        &conn,
        input.item_id,
        input.price,
        input.quantity,
        &input.vendor_type,
        input.vendor_name.as_deref(),
        input.notes.as_deref(),
    )
    .map_err(|e| format!("Failed to insert market price: {e}"))
}

#[derive(Serialize)]
pub struct MarketPriceRecord {
    pub id: i64,
    pub item_id: u32,
    pub price: f64,
    pub quantity: u32,
    pub vendor_type: String,
    pub vendor_name: Option<String>,
    pub observed_at: String,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn get_market_prices_for_item(
    db: State<'_, DbPool>,
    item_id: u32,
    limit: Option<usize>,
) -> Result<Vec<MarketPriceRecord>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, item_id, price, quantity, vendor_type, vendor_name,
                    datetime(observed_at) as observed_at, notes
             FROM market_prices
             WHERE item_id = ?1
             ORDER BY observed_at DESC
             LIMIT ?2",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let limit = limit.unwrap_or(20);
    let rows = stmt
        .query_map([item_id, limit as u32], |row| {
            Ok(MarketPriceRecord {
                id: row.get(0)?,
                item_id: row.get(1)?,
                price: row.get(2)?,
                quantity: row.get(3)?,
                vendor_type: row.get(4)?,
                vendor_name: row.get(5)?,
                observed_at: row.get(6)?,
                notes: row.get(7)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Row parse error: {e}"))?);
    }

    Ok(results)
}

// ── Sales History Commands ────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SaleInput {
    pub item_id: u32,
    pub quantity: u32,
    pub sale_price: f64,
    pub sale_method: String, // "vendor", "bazaar", "trade", "consignment"
    pub buyer_name: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn add_sale(db: State<'_, DbPool>, input: SaleInput) -> Result<i64, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    queries::player_data::insert_sale(
        &conn,
        input.item_id,
        input.quantity,
        input.sale_price,
        &input.sale_method,
        input.buyer_name.as_deref(),
        input.notes.as_deref(),
    )
    .map_err(|e| format!("Failed to insert sale: {e}"))
}

#[derive(Serialize)]
pub struct SaleRecord {
    pub id: i64,
    pub item_id: u32,
    pub quantity: u32,
    pub sale_price: f64,
    pub sale_method: String,
    pub buyer_name: Option<String>,
    pub sold_at: String,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn get_recent_sales(
    db: State<'_, DbPool>,
    days: Option<u32>,
    limit: Option<usize>,
) -> Result<Vec<SaleRecord>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let days = days.unwrap_or(30);
    let limit = limit.unwrap_or(50);

    let mut stmt = conn
        .prepare(
            "SELECT id, item_id, quantity, sale_price, sale_method, buyer_name,
                    datetime(sold_at) as sold_at, notes
             FROM sales_history
             WHERE sold_at > datetime('now', ?1)
             ORDER BY sold_at DESC
             LIMIT ?2",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt
        .query_map([format!("-{} days", days), limit.to_string()], |row| {
            Ok(SaleRecord {
                id: row.get(0)?,
                item_id: row.get(1)?,
                quantity: row.get(2)?,
                sale_price: row.get(3)?,
                sale_method: row.get(4)?,
                buyer_name: row.get(5)?,
                sold_at: row.get(6)?,
                notes: row.get(7)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Row parse error: {e}"))?);
    }

    Ok(results)
}

// ── Survey Commands ───────────────────────────────────────────────────────────

// ── Event Log Commands ────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LogEventInput {
    pub event_type: String,
    pub event_data: serde_json::Value,
}

#[tauri::command]
pub fn log_event(db: State<'_, DbPool>, input: LogEventInput) -> Result<i64, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let event_data_json = serde_json::to_string(&input.event_data)
        .map_err(|e| format!("Failed to serialize event data: {e}"))?;

    queries::player_data::log_event(&conn, &input.event_type, &event_data_json)
        .map_err(|e| format!("Failed to log event: {e}"))
}

#[derive(Serialize)]
pub struct EventLogRecord {
    pub id: i64,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub created_at: String,
}

#[tauri::command]
pub fn get_recent_events(
    db: State<'_, DbPool>,
    event_type: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<EventLogRecord>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let limit = limit.unwrap_or(50);

    let mut results = Vec::new();

    if let Some(evt) = event_type {
        let mut stmt = conn
            .prepare(
                "SELECT id, event_type, event_data, datetime(created_at) as created_at
                 FROM event_log
                 WHERE event_type = ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let rows = stmt
            .query_map([evt, limit.to_string()], |row| {
                let event_data_str: String = row.get(2)?;
                let event_data: serde_json::Value =
                    serde_json::from_str(&event_data_str).unwrap_or(serde_json::Value::Null);

                Ok(EventLogRecord {
                    id: row.get(0)?,
                    event_type: row.get(1)?,
                    event_data,
                    created_at: row.get(3)?,
                })
            })
            .map_err(|e| format!("Query failed: {e}"))?;

        for row in rows {
            results.push(row.map_err(|e| format!("Row parse error: {e}"))?);
        }
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT id, event_type, event_data, datetime(created_at) as created_at
                 FROM event_log
                 ORDER BY created_at DESC
                 LIMIT ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let rows = stmt
            .query_map([limit], |row| {
                let event_data_str: String = row.get(2)?;
                let event_data: serde_json::Value =
                    serde_json::from_str(&event_data_str).unwrap_or(serde_json::Value::Null);

                Ok(EventLogRecord {
                    id: row.get(0)?,
                    event_type: row.get(1)?,
                    event_data,
                    created_at: row.get(3)?,
                })
            })
            .map_err(|e| format!("Query failed: {e}"))?;

        for row in rows {
            results.push(row.map_err(|e| format!("Row parse error: {e}"))?);
        }
    }

    Ok(results)
}

// ── Survey Session Stats Commands ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SaveSessionStatsInput {
    pub start_time: String,
    pub end_time: Option<String>,
    pub maps_started: u32,
    pub surveys_located: u32,
    pub surveys_completed: u32,
    pub surveying_xp_gained: u32,
    pub mining_xp_gained: u32,
    pub geology_xp_gained: u32,
    pub total_revenue: u32,
    pub total_cost: u32,
    pub total_profit: i32,
    pub profit_per_hour: i32,
    pub elapsed_seconds: u32,
    pub is_manual: bool,
}

#[tauri::command]
pub fn save_survey_session_stats(
    db: State<'_, DbPool>,
    input: SaveSessionStatsInput,
) -> Result<i64, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    conn.execute(
        "INSERT INTO survey_session_stats (
            start_time, end_time, maps_started, surveys_located, surveys_completed,
            surveying_xp_gained, mining_xp_gained, geology_xp_gained,
            total_revenue, total_cost, total_profit, profit_per_hour,
            elapsed_seconds, is_manual
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        rusqlite::params![
            input.start_time,
            input.end_time,
            input.maps_started,
            input.surveys_located,
            input.surveys_completed,
            input.surveying_xp_gained,
            input.mining_xp_gained,
            input.geology_xp_gained,
            input.total_revenue,
            input.total_cost,
            input.total_profit,
            input.profit_per_hour,
            input.elapsed_seconds,
            input.is_manual,
        ],
    )
    .map_err(|e| format!("Failed to save session stats: {e}"))?;

    Ok(conn.last_insert_rowid())
}

/// Patch frontend-known fields onto a session. Also (re-)runs finalization to compute
/// revenue/cost/profit from DB data, then patches in elapsed time (with pause accounting),
/// XP gains, and manual flag. This ensures both auto-ended and manually-ended sessions
/// get correct economics.
#[derive(Deserialize)]
pub struct PatchSessionInput {
    pub elapsed_seconds: i64,
    pub surveying_xp_gained: u32,
    pub mining_xp_gained: u32,
    pub geology_xp_gained: u32,
    pub is_manual: bool,
}

#[tauri::command]
pub fn patch_survey_session(
    db: State<'_, DbPool>,
    session_id: i64,
    input: PatchSessionInput,
) -> Result<(), String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    // (Re-)run finalization to compute revenue/cost/profit/bonus counts/summaries
    crate::survey_persistence::finalize_session(&conn, session_id);

    // Now read the freshly computed profit and recompute profit_per_hour with
    // the frontend's accurate elapsed time (which accounts for pauses)
    let total_profit: f64 = conn
        .query_row(
            "SELECT total_profit FROM survey_session_stats WHERE id = ?1",
            [session_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to read session: {e}"))?;

    let hours = input.elapsed_seconds as f64 / 3600.0;
    let profit_per_hour = if hours > 0.0 {
        total_profit / hours
    } else {
        0.0
    };

    conn.execute(
        "UPDATE survey_session_stats SET
            elapsed_seconds = ?1,
            surveying_xp_gained = ?2,
            mining_xp_gained = ?3,
            geology_xp_gained = ?4,
            is_manual = ?5,
            profit_per_hour = ?6
         WHERE id = ?7",
        rusqlite::params![
            input.elapsed_seconds,
            input.surveying_xp_gained,
            input.mining_xp_gained,
            input.geology_xp_gained,
            input.is_manual,
            profit_per_hour,
            session_id,
        ],
    )
    .map_err(|e| format!("Failed to patch session stats: {e}"))?;

    Ok(())
}

#[derive(Serialize)]
pub struct HistoricalSession {
    pub id: i64,
    pub start_time: String,
    pub end_time: Option<String>,
    pub maps_started: u32,
    pub surveys_completed: u32,
    /// Total completions including motherlodes (computed from survey_events)
    pub total_completions: u32,
    pub total_revenue: f64,
    pub total_cost: f64,
    pub total_profit: f64,
    pub profit_per_hour: f64,
    pub elapsed_seconds: i64,
    pub speed_bonus_count: i64,
    pub survey_types_used: Option<String>,
    pub maps_used_summary: Option<String>,
    pub name: String,
    pub notes: String,
    pub surveying_xp_gained: u32,
    pub mining_xp_gained: u32,
    pub geology_xp_gained: u32,
}

#[tauri::command]
pub fn get_historical_sessions(
    db: State<'_, DbPool>,
    limit: Option<usize>,
) -> Result<Vec<HistoricalSession>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let limit = limit.unwrap_or(50);

    // All summary data is pre-computed in survey_session_stats by finalize_session()
    let mut stmt = conn
        .prepare(
            "SELECT
                s.id,
                datetime(s.start_time) as start_time,
                datetime(s.end_time) as end_time,
                s.maps_started,
                s.surveys_completed,
                COALESCE((SELECT COUNT(*) FROM survey_events se
                          WHERE se.session_id = s.id
                            AND se.event_type IN ('completed', 'motherlode_completed')), 0) as total_completions,
                s.total_revenue,
                s.total_cost,
                s.total_profit,
                s.profit_per_hour,
                s.elapsed_seconds,
                s.speed_bonus_count,
                s.survey_types_used,
                s.maps_used_summary,
                s.name,
                s.notes,
                s.surveying_xp_gained,
                s.mining_xp_gained,
                s.geology_xp_gained
             FROM survey_session_stats s
             WHERE s.maps_started > 0
             ORDER BY s.start_time DESC
             LIMIT ?1"
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt
        .query_map([limit], |row| {
            Ok(HistoricalSession {
                id: row.get(0)?,
                start_time: row.get(1)?,
                end_time: row.get(2)?,
                maps_started: row.get(3)?,
                surveys_completed: row.get(4)?,
                total_completions: row.get(5)?,
                total_revenue: row.get(6)?,
                total_cost: row.get(7)?,
                total_profit: row.get(8)?,
                profit_per_hour: row.get(9)?,
                elapsed_seconds: row.get(10)?,
                speed_bonus_count: row.get(11)?,
                survey_types_used: row.get(12)?,
                maps_used_summary: row.get(13)?,
                name: row.get(14)?,
                notes: row.get(15)?,
                surveying_xp_gained: row.get(16)?,
                mining_xp_gained: row.get(17)?,
                geology_xp_gained: row.get(18)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Row parse error: {e}"))?);
    }

    Ok(results)
}

#[tauri::command]
pub fn update_survey_session(
    db: State<'_, DbPool>,
    session_id: i64,
    name: String,
    notes: String,
) -> Result<(), String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;
    conn.execute(
        "UPDATE survey_session_stats SET name = ?1, notes = ?2 WHERE id = ?3",
        rusqlite::params![name, notes, session_id],
    )
    .map_err(|e| format!("Failed to update session: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn delete_survey_session(db: State<'_, DbPool>, session_id: i64) -> Result<(), String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    conn.execute(
        "DELETE FROM survey_session_stats WHERE id = ?1",
        [session_id],
    )
    .map_err(|e| format!("Failed to delete survey session: {e}"))?;

    Ok(())
}
