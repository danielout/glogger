use super::DbPool;
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
    pub entry_index: i64,
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
    pub ignored: bool,
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
                (event_timestamp, log_timestamp, log_title, action, player, owner, item, quantity, price_unit, price_total, raw_message, entry_index)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
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
                event.entry_index,
            ],
        ).map_err(|e| e.to_string())?;
        count += result;
    }
    Ok(count)
}

// ── Row mapper ─────────────────────────────────────────────────────────────

fn row_to_stall_event(row: &rusqlite::Row) -> rusqlite::Result<StallEvent> {
    let ignored_int: i64 = row.get(13)?;
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
        ignored: ignored_int != 0,
    })
}

// ── Tauri commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_stall_sales(
    db: State<'_, DbPool>,
) -> Result<Vec<StallEvent>, String> {
    let conn = db.get().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, event_timestamp, log_timestamp, log_title, action, player, owner, item, quantity, price_unit, price_total, raw_message, created_at, ignored
             FROM stall_events
             WHERE action = 'bought'
             ORDER BY event_timestamp DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], row_to_stall_event)
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_stall_log(
    db: State<'_, DbPool>,
) -> Result<Vec<StallEvent>, String> {
    let conn = db.get().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, event_timestamp, log_timestamp, log_title, action, player, owner, item, quantity, price_unit, price_total, raw_message, created_at, ignored
             FROM stall_events
             ORDER BY event_timestamp DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], row_to_stall_event)
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

#[tauri::command]
pub fn clear_stall_events(db: State<'_, DbPool>) -> Result<usize, String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    let deleted = conn
        .execute("DELETE FROM stall_events", [])
        .map_err(|e| e.to_string())?;
    Ok(deleted)
}

#[tauri::command]
pub fn toggle_stall_event_ignored(
    db: State<'_, DbPool>,
    id: i64,
    ignored: bool,
) -> Result<(), String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE stall_events SET ignored = ?1 WHERE id = ?2",
        rusqlite::params![ignored as i32, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Serialize)]
pub struct ImportResult {
    pub total_entries: usize,
    pub new_entries: usize,
}

#[tauri::command]
pub fn import_shop_log_file(
    db: State<'_, DbPool>,
    path: String,
) -> Result<ImportResult, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {e}"))?;

    let shop_log = crate::shop_log_parser::parse_shop_log("Imported", &content, "imported");
    let total_entries = shop_log.entries.len();

    let inputs: Vec<StallEventInput> = shop_log.entries.iter().map(|e| {
        StallEventInput {
            event_timestamp: e.timestamp.clone(),
            log_timestamp: "imported".to_string(),
            log_title: "Imported".to_string(),
            action: e.action.clone(),
            player: e.player.clone(),
            owner: shop_log.owner.clone(),
            item: e.item.clone(),
            quantity: e.quantity,
            price_unit: e.price_unit,
            price_total: e.price_total,
            raw_message: e.raw_message.clone(),
            entry_index: e.entry_index,
        }
    }).collect();

    let new_entries = insert_stall_events(&db, &inputs)?;

    Ok(ImportResult {
        total_entries,
        new_entries,
    })
}

#[derive(Serialize)]
pub struct ExportResult {
    pub files_written: usize,
    pub entries_written: usize,
    pub directory: String,
}

static MONTHS: &[(&str, u32)] = &[
    ("Jan", 1), ("Feb", 2), ("Mar", 3), ("Apr", 4), ("May", 5), ("Jun", 6),
    ("Jul", 7), ("Aug", 8), ("Sep", 9), ("Oct", 10), ("Nov", 11), ("Dec", 12),
];

/// Parse "Mon Apr 13 14:29" → (month, day). Returns None if unparseable.
fn parse_month_day(ts: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = ts.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    let mon = MONTHS.iter().find(|(n, _)| *n == parts[1]).map(|(_, m)| *m)?;
    let day: u32 = parts[2].parse().ok()?;
    Some((mon, day))
}

fn sanitize_filename_component(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

#[tauri::command]
pub fn export_shop_log_files(
    db: State<'_, DbPool>,
    directory: String,
) -> Result<ExportResult, String> {
    let conn = db.get().map_err(|e| e.to_string())?;

    // Fetch all non-unknown events. We read raw_message to preserve exact
    // in-game phrasing and event_timestamp to group by (owner, date).
    let mut stmt = conn
        .prepare(
            "SELECT event_timestamp, owner, raw_message, entry_index
             FROM stall_events
             WHERE action != 'unknown'",
        )
        .map_err(|e| e.to_string())?;

    struct Row {
        event_timestamp: String,
        owner: Option<String>,
        raw_message: String,
        entry_index: i64,
    }

    let rows = stmt
        .query_map([], |row| {
            Ok(Row {
                event_timestamp: row.get(0)?,
                owner: row.get(1)?,
                raw_message: row.get(2)?,
                entry_index: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // event_timestamp has no year; use the current year for filenames.
    let year = chrono::Local::now().year();

    // Group by (owner, month, day). Each value is a list of rows we'll sort
    // within the day and emit newest-first, blank-line separated.
    let mut groups: BTreeMap<(String, u32, u32), Vec<Row>> = BTreeMap::new();
    for row in rows {
        let Some((mon, day)) = parse_month_day(&row.event_timestamp) else {
            continue;
        };
        let owner = row
            .owner
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "unknown".to_string());
        groups.entry((owner, mon, day)).or_default().push(row);
    }

    let dir_path = std::path::PathBuf::from(&directory);
    if !dir_path.is_dir() {
        return Err(format!("Not a directory: {directory}"));
    }

    let mut files_written = 0usize;
    let mut entries_written = 0usize;

    for ((owner, mon, day), mut entries) in groups {
        // Newest first: sort by (timestamp string desc, entry_index desc).
        // The HH:MM portion sorts lexicographically within the same day.
        entries.sort_by(|a, b| {
            b.event_timestamp
                .cmp(&a.event_timestamp)
                .then(b.entry_index.cmp(&a.entry_index))
        });

        let mut body = String::new();
        for (i, row) in entries.iter().enumerate() {
            if i > 0 {
                body.push_str("\n\n");
            }
            body.push_str(&row.event_timestamp);
            body.push_str(" - ");
            body.push_str(&row.raw_message);
        }
        body.push('\n');

        let filename = format!(
            "{}-shop-log-{:04}-{:02}-{:02}.txt",
            sanitize_filename_component(&owner),
            year,
            mon,
            day
        );
        let path = dir_path.join(&filename);
        std::fs::write(&path, body)
            .map_err(|e| format!("Failed to write {}: {e}", path.display()))?;

        files_written += 1;
        entries_written += entries.len();
    }

    Ok(ExportResult {
        files_written,
        entries_written,
        directory,
    })
}
