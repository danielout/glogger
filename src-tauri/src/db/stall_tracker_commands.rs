use super::DbPool;
use serde::{Deserialize, Serialize};
use tauri::State;

// ── Input types ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct StallEventInput {
    pub event_timestamp: String,
    pub log_timestamp: String,
    pub log_title: String,
    pub action: String,
    pub player: String,
    pub owner: Option<String>,
    pub item: Option<String>,
    pub quantity: i64,
    pub price_unit: Option<f64>,
    pub price_total: Option<i64>,
    pub raw_message: String,
}

// ── Output types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct StallEvent {
    pub id: i64,
    pub event_timestamp: String,
    pub log_timestamp: String,
    pub log_title: String,
    pub action: String,
    pub player: String,
    pub owner: Option<String>,
    pub item: Option<String>,
    pub quantity: i64,
    pub price_unit: Option<f64>,
    pub price_total: Option<i64>,
    pub raw_message: String,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct StallStats {
    pub total_sales: i64,
    pub total_revenue: i64,
    pub unique_buyers: i64,
    pub unique_items: i64,
}

// ── Non-command helper (called from coordinator) ────────────────────────────

/// Insert stall events into the database, ignoring duplicates.
/// Returns the number of newly inserted rows.
pub fn insert_stall_events(pool: &DbPool, events: &[StallEventInput]) -> Result<usize, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    let mut count = 0usize;
    for event in events {
        let result = conn.execute(
            "INSERT OR IGNORE INTO stall_events
                (event_timestamp, log_timestamp, log_title, action, player, owner, item, quantity, price_unit, price_total, raw_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                event.event_timestamp,
                event.log_timestamp,
                event.log_title,
                event.action,
                event.player,
                event.owner,
                event.item,
                event.quantity,
                event.price_unit,
                event.price_total,
                event.raw_message,
            ],
        ).map_err(|e| e.to_string())?;
        count += result;
    }
    Ok(count)
}

// ── Tauri commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_stall_sales(
    db: State<'_, DbPool>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<StallEvent>, String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(500);
    let offset = offset.unwrap_or(0);

    let mut stmt = conn
        .prepare(
            "SELECT id, event_timestamp, log_timestamp, log_title, action, player, owner, item, quantity, price_unit, price_total, raw_message, created_at
             FROM stall_events
             WHERE action = 'bought'
             ORDER BY event_timestamp DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![limit, offset], |row| {
            Ok(StallEvent {
                id: row.get(0)?,
                event_timestamp: row.get(1)?,
                log_timestamp: row.get(2)?,
                log_title: row.get(3)?,
                action: row.get(4)?,
                player: row.get(5)?,
                owner: row.get(6)?,
                item: row.get(7)?,
                quantity: row.get(8)?,
                price_unit: row.get(9)?,
                price_total: row.get(10)?,
                raw_message: row.get(11)?,
                created_at: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_stall_log(
    db: State<'_, DbPool>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<StallEvent>, String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(500);
    let offset = offset.unwrap_or(0);

    let mut stmt = conn
        .prepare(
            "SELECT id, event_timestamp, log_timestamp, log_title, action, player, owner, item, quantity, price_unit, price_total, raw_message, created_at
             FROM stall_events
             ORDER BY event_timestamp DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![limit, offset], |row| {
            Ok(StallEvent {
                id: row.get(0)?,
                event_timestamp: row.get(1)?,
                log_timestamp: row.get(2)?,
                log_title: row.get(3)?,
                action: row.get(4)?,
                player: row.get(5)?,
                owner: row.get(6)?,
                item: row.get(7)?,
                quantity: row.get(8)?,
                price_unit: row.get(9)?,
                price_total: row.get(10)?,
                raw_message: row.get(11)?,
                created_at: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_stall_stats(db: State<'_, DbPool>) -> Result<StallStats, String> {
    let conn = db.get().map_err(|e| e.to_string())?;

    let total_sales: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM stall_events WHERE action = 'bought'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let total_revenue: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(price_total), 0) FROM stall_events WHERE action = 'bought'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let unique_buyers: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT player) FROM stall_events WHERE action = 'bought'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let unique_items: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT item) FROM stall_events WHERE action = 'bought' AND item IS NOT NULL",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(StallStats {
        total_sales,
        total_revenue,
        unique_buyers,
        unique_items,
    })
}
