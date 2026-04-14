use super::DbPool;
use crate::stall_aggregations::{
    aggregate_inventory, aggregate_revenue, Granularity, InventoryEvent, InventoryResult,
    RevenueEvent, RevenueResult,
};
use chrono::{Datelike, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Mutex;
use tauri::State;

/// Short-held mutex that serializes stall_events writes across the
/// coordinator's live-ingest path and user-initiated writes (Clear).
/// Acquire only around the actual DB mutation — keep critical sections small.
#[derive(Default)]
pub struct StallOpsLock(pub Mutex<()>);

// ── Input types ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct StallEventInput {
    pub event_timestamp: String,
    pub event_at: Option<String>,
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
///
/// Acquires `StallOpsLock` around the batch so a concurrent Clear can't
/// interleave with partial inserts and leave orphaned rows behind.
pub fn insert_stall_events(
    pool: &DbPool,
    ops_lock: &StallOpsLock,
    events: &[StallEventInput],
) -> Result<usize, String> {
    let _guard = ops_lock.0.lock().map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let mut count = 0usize;
    for event in events {
        let result = conn.execute(
            "INSERT OR IGNORE INTO stall_events
                (event_timestamp, event_at, log_timestamp, log_title, action, player, owner, item, quantity, price_unit, price_total, raw_message, entry_index)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            rusqlite::params![
                event.event_timestamp,
                event.event_at,
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

/// Common filter shape for paginated/aggregated queries.
/// All fields optional; empty strings behave as None.
///
/// `owner` scopes the query to a single character's stall data — the
/// frontend always passes the active character. A missing/empty value
/// returns all characters' data (used by diagnostic tooling only).
#[derive(Deserialize, Default, Debug)]
pub struct StallEventsFilters {
    pub owner: Option<String>,
    pub action: Option<String>,
    pub player: Option<String>,
    pub item: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub include_ignored: Option<bool>,
}

fn opt_nonempty(s: &Option<String>) -> Option<&str> {
    s.as_deref().filter(|v| !v.is_empty())
}

/// Build a WHERE clause + bound parameters from a filter struct.
/// Returns (sql_fragment, params). Starts with " WHERE 1=1" so callers can
/// concatenate more conditions without worrying about the leading keyword.
fn build_filter_where(
    filters: &StallEventsFilters,
    force_action: Option<&str>,
) -> (String, Vec<rusqlite::types::Value>) {
    use rusqlite::types::Value;
    let mut sql = String::from(" WHERE 1=1");
    let mut params: Vec<Value> = Vec::new();

    if let Some(o) = opt_nonempty(&filters.owner) {
        sql.push_str(" AND owner = ?");
        params.push(Value::Text(o.into()));
    }
    if let Some(a) = force_action {
        sql.push_str(" AND action = ?");
        params.push(Value::Text(a.into()));
    } else if let Some(a) = opt_nonempty(&filters.action) {
        sql.push_str(" AND action = ?");
        params.push(Value::Text(a.into()));
    } else {
        // Exclude unknown events from list views.
        sql.push_str(" AND action != 'unknown'");
    }
    if let Some(p) = opt_nonempty(&filters.player) {
        sql.push_str(" AND player = ?");
        params.push(Value::Text(p.into()));
    }
    if let Some(i) = opt_nonempty(&filters.item) {
        sql.push_str(" AND item = ?");
        params.push(Value::Text(i.into()));
    }
    if let Some(from) = opt_nonempty(&filters.date_from) {
        sql.push_str(" AND event_at >= ?");
        params.push(Value::Text(format!("{from} 00:00:00")));
    }
    if let Some(to) = opt_nonempty(&filters.date_to) {
        sql.push_str(" AND event_at <= ?");
        params.push(Value::Text(format!("{to} 23:59:59")));
    }
    if !filters.include_ignored.unwrap_or(true) {
        sql.push_str(" AND NOT ignored");
    }
    (sql, params)
}

#[derive(Deserialize, Default)]
pub struct StallEventsParams {
    #[serde(flatten)]
    pub filters: StallEventsFilters,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize)]
pub struct StallEventsPage {
    pub rows: Vec<StallEvent>,
    pub total_count: i64,
}

fn resolve_sort(sort_by: Option<&str>, sort_dir: Option<&str>) -> String {
    // Whitelist sortable columns. `event_at` is the real timestamp column
    // (Phase 1); entry_index + id give stable tiebreakers for same-minute rows.
    let col = match sort_by.unwrap_or("event_at") {
        "event_at" | "event_timestamp" => "event_at",
        "player" => "player",
        "item" => "item",
        "action" => "action",
        "quantity" => "quantity",
        "price_unit" => "price_unit",
        "price_total" => "price_total",
        _ => "event_at",
    };
    let dir = match sort_dir.unwrap_or("desc") {
        "asc" => "ASC",
        _ => "DESC",
    };
    // Primary + stable tiebreakers. COALESCE event_timestamp for pre-migration rows.
    format!("{col} {dir}, event_at {dir}, entry_index {dir}, id {dir}")
}

#[tauri::command]
pub fn get_stall_events(
    db: State<'_, DbPool>,
    params: StallEventsParams,
) -> Result<StallEventsPage, String> {
    // No character loaded yet → nothing to show. Avoid leaking data from
    // other characters if the frontend forgets to pass owner.
    if opt_nonempty(&params.filters.owner).is_none() {
        return Ok(StallEventsPage { rows: Vec::new(), total_count: 0 });
    }
    let conn = db.get().map_err(|e| e.to_string())?;

    let (where_sql, where_params) = build_filter_where(&params.filters, None);
    let order_sql = resolve_sort(params.sort_by.as_deref(), params.sort_dir.as_deref());
    let limit = params.limit.unwrap_or(500).clamp(1, 10_000);
    let offset = params.offset.unwrap_or(0).max(0);

    let count_sql = format!("SELECT COUNT(*) FROM stall_events{where_sql}");
    let total_count: i64 = conn
        .query_row(
            &count_sql,
            rusqlite::params_from_iter(where_params.iter()),
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let rows_sql = format!(
        "SELECT id, event_timestamp, log_timestamp, log_title, action, player, owner, item, quantity, price_unit, price_total, raw_message, created_at, ignored
         FROM stall_events{where_sql}
         ORDER BY {order_sql}
         LIMIT ? OFFSET ?"
    );

    let mut stmt = conn.prepare(&rows_sql).map_err(|e| e.to_string())?;
    let mut full_params = where_params.clone();
    full_params.push(rusqlite::types::Value::Integer(limit));
    full_params.push(rusqlite::types::Value::Integer(offset));

    let rows_iter = stmt
        .query_map(rusqlite::params_from_iter(full_params.iter()), row_to_stall_event)
        .map_err(|e| e.to_string())?;
    let rows: Vec<StallEvent> = rows_iter.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(StallEventsPage { rows, total_count })
}

#[tauri::command]
pub fn get_stall_stats(
    db: State<'_, DbPool>,
    filters: Option<StallEventsFilters>,
) -> Result<StallStats, String> {
    let mut f = filters.unwrap_or_default();
    if opt_nonempty(&f.owner).is_none() {
        return Ok(StallStats {
            total_sales: 0,
            total_revenue: 0,
            unique_buyers: 0,
            unique_items: 0,
        });
    }
    let conn = db.get().map_err(|e| e.to_string())?;

    // Stats are always scoped to non-ignored 'bought' events, with optional
    // additional filters from the caller.
    f.include_ignored = Some(false);
    let (where_sql, where_params) = build_filter_where(&f, Some("bought"));
    let params_ref = rusqlite::params_from_iter(where_params.iter());

    let sql = format!(
        "SELECT
            COUNT(*),
            COALESCE(SUM(price_total), 0),
            COUNT(DISTINCT player),
            COUNT(DISTINCT CASE WHEN item IS NOT NULL THEN item END)
         FROM stall_events{where_sql}"
    );

    let (total_sales, total_revenue, unique_buyers, unique_items): (i64, i64, i64, i64) =
        conn.query_row(&sql, params_ref, |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| e.to_string())?;

    let result = StallStats {
        total_sales,
        total_revenue,
        unique_buyers,
        unique_items,
    };
    Ok(result)
}

#[tauri::command]
pub fn clear_stall_events(
    db: State<'_, DbPool>,
    ops_lock: State<'_, StallOpsLock>,
    owner: Option<String>,
) -> Result<usize, String> {
    // Always scope Clear to a specific character. The UI flow can only reach
    // this command once a character is loaded, so a missing owner here means
    // the client misbehaved — fail loudly rather than wipe every character.
    let owner = owner
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "clear_stall_events requires an owner".to_string())?;

    let _guard = ops_lock.0.lock().map_err(|e| e.to_string())?;
    let conn = db.get().map_err(|e| e.to_string())?;
    let deleted = conn
        .execute("DELETE FROM stall_events WHERE owner = ?1", rusqlite::params![owner])
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
    /// The stall owner parsed from the book's owner actions (added, visible,
    /// configured, etc.). None if the book had no owner-type events, which
    /// happens when the file is a plain list of "bought" entries with no
    /// context. Frontend uses this to warn when the parsed owner doesn't
    /// match the active character.
    pub parsed_owner: Option<String>,
}

/// Extract a 4-digit year from a filename like `Deradon-shop-log-2026-04-13.txt`
/// or `shop-log-2026-03-23.txt`. Falls back to None if no year pattern matches.
fn year_from_filename(path: &str) -> Option<i32> {
    let name = std::path::Path::new(path)
        .file_name()?
        .to_str()?;
    // Find a 4-digit sequence between 2000 and 2099 (generous bounds).
    let bytes = name.as_bytes();
    for i in 0..bytes.len().saturating_sub(3) {
        let slice = &bytes[i..i + 4];
        if slice.iter().all(|b| b.is_ascii_digit()) {
            let y: i32 = std::str::from_utf8(slice).ok()?.parse().ok()?;
            if (2000..=2099).contains(&y) {
                return Some(y);
            }
        }
    }
    None
}

#[tauri::command]
pub fn import_shop_log_file(
    db: State<'_, DbPool>,
    ops_lock: State<'_, StallOpsLock>,
    path: String,
) -> Result<ImportResult, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {e}"))?;

    let base_year = year_from_filename(&path).unwrap_or_else(|| {
        use chrono::Datelike;
        chrono::Local::now().year()
    });
    let shop_log = crate::shop_log_parser::parse_shop_log("Imported", &content, "imported", base_year);
    let total_entries = shop_log.entries.len();

    let inputs: Vec<StallEventInput> = shop_log.entries.iter().map(|e| {
        StallEventInput {
            event_timestamp: e.timestamp.clone(),
            event_at: e.event_at.clone(),
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

    let new_entries = insert_stall_events(&db, &ops_lock, &inputs)?;

    Ok(ImportResult {
        total_entries,
        new_entries,
        parsed_owner: shop_log.owner.clone(),
    })
}

/// Dev-only: bulk-insert synthetic stall events for benchmarking.
///
/// Uses a single transaction for speed. ~90% bought, ~10% owner actions.
/// Spreads events across 30 days in Mar/Apr with 20 unique buyer names.
/// `owner` must be the active character so queries scoped by owner
/// actually surface the seeded data.
#[tauri::command]
pub fn seed_stall_events_dev(
    db: State<'_, DbPool>,
    count: usize,
    owner: Option<String>,
) -> Result<usize, String> {
    let owner_name = owner
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "seed_stall_events_dev requires an owner".to_string())?;
    let mut conn = db.get().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let buyers = [
        "Alduin", "Brynn", "Cyra", "Dren", "Elyra", "Fendrel", "Grimjaw", "Halek",
        "Ilia", "Jorek", "Kessa", "Loric", "Mavren", "Nyra", "Orin", "Pella",
        "Quillen", "Rask", "Sylva", "Tavish",
    ];
    let items = [
        ("Quality Reins", 4500),
        ("Great Saddle", 5000),
        ("Astounding Mystic Saddlebag", 40000),
        ("Amazing Horseshoes", 6000),
        ("Nice Reins", 4000),
    ];
    let days = [
        ("Mon", "Mar", 23, 3u32), ("Tue", "Mar", 24, 3), ("Wed", "Mar", 25, 3), ("Thu", "Mar", 26, 3),
        ("Fri", "Mar", 27, 3), ("Sat", "Mar", 28, 3), ("Sun", "Mar", 29, 3), ("Mon", "Mar", 30, 3),
        ("Tue", "Mar", 31, 3), ("Wed", "Apr", 1, 4), ("Thu", "Apr", 2, 4), ("Fri", "Apr", 3, 4),
        ("Sat", "Apr", 4, 4), ("Sun", "Apr", 5, 4), ("Mon", "Apr", 6, 4), ("Tue", "Apr", 7, 4),
        ("Wed", "Apr", 8, 4), ("Thu", "Apr", 9, 4), ("Fri", "Apr", 10, 4), ("Sat", "Apr", 11, 4),
        ("Sun", "Apr", 12, 4), ("Mon", "Apr", 13, 4),
    ];
    let owner = owner_name;
    use chrono::Datelike;
    let seed_year = chrono::Local::now().year();

    {
        let mut stmt = tx.prepare(
            "INSERT OR IGNORE INTO stall_events
                (event_timestamp, event_at, log_timestamp, log_title, action, player, owner, item, quantity, price_unit, price_total, raw_message, entry_index)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        ).map_err(|e| e.to_string())?;

        for i in 0..count {
            let (dow, mon, day, mon_num) = days[i % days.len()];
            let hour = (i / 60) % 24;
            let minute = i % 60;
            let ts = format!("{dow} {mon} {day} {hour:02}:{minute:02}");
            let event_at = format!("{seed_year:04}-{mon_num:02}-{day:02} {hour:02}:{minute:02}:00");
            let entry_index = (i / (days.len() * 24 * 60)) as i64;

            // Every 10th event is an owner action (mix of added/visible/configured/collected).
            if i % 10 == 0 {
                let (item, price) = items[i % items.len()];
                let (action, raw) = match (i / 10) % 4 {
                    0 => ("added", format!("{owner} added {item} to shop")),
                    1 => ("visible", format!("{owner} made {item} visible in shop at a cost of {price} per 1")),
                    2 => ("configured", format!("{owner} configured {item} to cost {price} per 1")),
                    _ => ("collected", format!("{owner} collected {} Councils from customer purchases", price * 3)),
                };
                stmt.execute(rusqlite::params![
                    ts, event_at, "seeded", "Seeded", action, owner, Some(owner),
                    if action == "collected" { None } else { Some(item) },
                    1i64,
                    if action == "added" || action == "collected" { None } else { Some(price as f64) },
                    if action == "collected" { Some((price * 3) as i64) } else { None },
                    raw, entry_index,
                ]).map_err(|e| e.to_string())?;
            } else {
                let buyer = buyers[i % buyers.len()];
                let (item, price) = items[i % items.len()];
                let raw = format!("{buyer} bought {item} at a cost of {price} per 1 = {price}");
                stmt.execute(rusqlite::params![
                    ts, event_at, "seeded", "Seeded", "bought", buyer, Some(owner),
                    Some(item), 1i64, Some(price as f64), Some(price as i64),
                    raw, entry_index,
                ]).map_err(|e| e.to_string())?;
            }
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(count)
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
    owner: Option<String>,
) -> Result<ExportResult, String> {
    // Export is scoped to a specific character to match the rest of the
    // Stall Tracker. Users who want to back up multiple characters switch
    // character and export each. The UI flow can only reach this command
    // once a character is loaded.
    let owner = owner
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "export_shop_log_files requires an owner".to_string())?;

    let conn = db.get().map_err(|e| e.to_string())?;

    // Fetch non-unknown events for this owner. We read `raw_message` to
    // preserve exact in-game phrasing and `event_at` to group by real date.
    // Pre-V22 rows that somehow lack event_at fall back to event_timestamp +
    // current year, which is only wrong across year boundaries — unlikely
    // for data that survived the V22 backfill.
    let mut stmt = conn
        .prepare(
            "SELECT event_timestamp, event_at, owner, raw_message, entry_index
             FROM stall_events
             WHERE action != 'unknown' AND owner = ?1",
        )
        .map_err(|e| e.to_string())?;

    struct Row {
        event_timestamp: String,
        event_at: Option<String>,
        owner: Option<String>,
        raw_message: String,
        entry_index: i64,
    }

    let rows = stmt
        .query_map(rusqlite::params![owner], |row| {
            Ok(Row {
                event_timestamp: row.get(0)?,
                event_at: row.get(1)?,
                owner: row.get(2)?,
                raw_message: row.get(3)?,
                entry_index: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let current_year = chrono::Local::now().year();

    // Group by (owner, year, month, day). Year comes from `event_at` when
    // available, falling back to current year for legacy rows.
    let mut groups: BTreeMap<(String, i32, u32, u32), Vec<Row>> = BTreeMap::new();
    for row in rows {
        let (year, mon, day) = if let Some(ea) = row.event_at.as_deref() {
            match chrono::NaiveDateTime::parse_from_str(ea, "%Y-%m-%d %H:%M:%S") {
                Ok(dt) => (dt.year(), dt.month(), dt.day()),
                Err(_) => continue,
            }
        } else {
            match parse_month_day(&row.event_timestamp) {
                Some((m, d)) => (current_year, m, d),
                None => continue,
            }
        };
        let owner = row
            .owner
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "unknown".to_string());
        groups.entry((owner, year, mon, day)).or_default().push(row);
    }

    let dir_path = std::path::PathBuf::from(&directory);
    if !dir_path.is_dir() {
        return Err(format!("Not a directory: {directory}"));
    }

    let mut files_written = 0usize;
    let mut entries_written = 0usize;

    for ((owner, year, mon, day), mut entries) in groups {
        // Newest first: sort by (event_at desc, entry_index desc), falling
        // back to event_timestamp for legacy rows without event_at.
        entries.sort_by(|a, b| {
            let a_key = a.event_at.as_deref().unwrap_or(&a.event_timestamp);
            let b_key = b.event_at.as_deref().unwrap_or(&b.event_timestamp);
            b_key.cmp(a_key).then(b.entry_index.cmp(&a.entry_index))
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

// ── Phase 2 aggregation commands ────────────────────────────────────────────

fn parse_event_at(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()
}

#[derive(Deserialize, Default)]
pub struct RevenueParams {
    pub owner: Option<String>,
    pub granularity: String,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub buyer: Option<String>,
    pub item: Option<String>,
}

#[tauri::command]
pub fn get_stall_revenue(
    db: State<'_, DbPool>,
    params: RevenueParams,
) -> Result<RevenueResult, String> {
    let granularity = Granularity::parse(&params.granularity)
        .ok_or_else(|| format!("Invalid granularity: {}", params.granularity))?;

    if opt_nonempty(&params.owner).is_none() {
        return Ok(aggregate_revenue(std::iter::empty(), granularity));
    }

    let conn = db.get().map_err(|e| e.to_string())?;
    let mut sql = String::from(
        "SELECT event_at, item, price_total, player
         FROM stall_events
         WHERE action = 'bought'
           AND NOT ignored
           AND item IS NOT NULL
           AND event_at IS NOT NULL",
    );
    let mut args: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(o) = params.owner.as_deref().filter(|s| !s.is_empty()) {
        sql.push_str(" AND owner = ?");
        args.push(rusqlite::types::Value::Text(o.into()));
    }
    if let Some(from) = &params.date_from {
        sql.push_str(" AND event_at >= ?");
        args.push(rusqlite::types::Value::Text(format!("{from} 00:00:00")));
    }
    if let Some(to) = &params.date_to {
        sql.push_str(" AND event_at <= ?");
        args.push(rusqlite::types::Value::Text(format!("{to} 23:59:59")));
    }
    if let Some(b) = &params.buyer {
        if !b.is_empty() {
            sql.push_str(" AND player = ?");
            args.push(rusqlite::types::Value::Text(b.clone()));
        }
    }
    if let Some(i) = &params.item {
        if !i.is_empty() {
            sql.push_str(" AND item = ?");
            args.push(rusqlite::types::Value::Text(i.clone()));
        }
    }

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(args.iter()), |row| {
            let event_at: String = row.get(0)?;
            let item: String = row.get(1)?;
            let revenue: Option<i64> = row.get(2)?;
            Ok((event_at, item, revenue.unwrap_or(0)))
        })
        .map_err(|e| e.to_string())?;

    let events: Vec<RevenueEvent> = rows
        .filter_map(|r| {
            let (event_at, item, revenue) = r.ok()?;
            let dt = parse_event_at(&event_at)?;
            Some(RevenueEvent {
                event_at: dt,
                item,
                revenue,
            })
        })
        .collect();

    Ok(aggregate_revenue(events, granularity))
}

#[derive(Deserialize)]
pub struct InventoryParams {
    pub owner: Option<String>,
    /// Sales-period window in days. Use a very large number (e.g., 100000) for "all time".
    pub period_days: f64,
}

#[tauri::command]
pub fn get_stall_inventory(
    db: State<'_, DbPool>,
    params: InventoryParams,
) -> Result<InventoryResult, String> {
    if opt_nonempty(&params.owner).is_none() {
        return Ok(aggregate_inventory(std::iter::empty(), params.period_days));
    }
    let conn = db.get().map_err(|e| e.to_string())?;

    let mut sql = String::from(
        "SELECT event_at, action, item, quantity, price_unit, price_total
         FROM stall_events
         WHERE action IN ('added', 'removed', 'bought', 'configured', 'visible')
           AND NOT ignored
           AND item IS NOT NULL
           AND event_at IS NOT NULL",
    );
    let mut args: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(o) = params.owner.as_deref().filter(|s| !s.is_empty()) {
        sql.push_str(" AND owner = ?");
        args.push(rusqlite::types::Value::Text(o.into()));
    }
    sql.push_str(" ORDER BY event_at ASC, id ASC");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params_from_iter(args.iter()), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, Option<f64>>(4)?,
                row.get::<_, Option<i64>>(5)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let events: Vec<InventoryEvent> = rows
        .filter_map(|r| {
            let (event_at, action, item, quantity, price_unit, price_total) = r.ok()?;
            let dt = parse_event_at(&event_at)?;
            Some(InventoryEvent {
                event_at: dt,
                action,
                item,
                quantity,
                price_unit,
                price_total,
            })
        })
        .collect();

    Ok(aggregate_inventory(events, params.period_days))
}

#[derive(Serialize)]
pub struct StallFilterOptions {
    /// Distinct players from 'bought' events only (for Sales/Revenue tabs).
    pub buyers: Vec<String>,
    /// Distinct players across all non-unknown events (for Shop Log tab).
    /// Includes both buyers and stall owners.
    pub players: Vec<String>,
    pub items: Vec<String>,
    pub dates: Vec<String>,   // ISO dates, newest first
    pub actions: Vec<String>,
}

#[tauri::command]
pub fn get_stall_filter_options(
    db: State<'_, DbPool>,
    owner: Option<String>,
) -> Result<StallFilterOptions, String> {
    if opt_nonempty(&owner).is_none() {
        return Ok(StallFilterOptions {
            buyers: Vec::new(),
            players: Vec::new(),
            items: Vec::new(),
            dates: Vec::new(),
            actions: Vec::new(),
        });
    }
    let conn = db.get().map_err(|e| e.to_string())?;

    // When an owner is supplied, every dropdown list is scoped to that
    // character's data. Callers pass the active character; only dev/debug
    // tooling would call this without one.
    let owner_clause = if owner.as_deref().map(|s| !s.is_empty()).unwrap_or(false) {
        " AND owner = ?1"
    } else {
        ""
    };
    let params: Vec<&dyn rusqlite::ToSql> =
        if owner.as_deref().map(|s| !s.is_empty()).unwrap_or(false) {
            vec![owner.as_ref().unwrap() as &dyn rusqlite::ToSql]
        } else {
            vec![]
        };

    let buyers: Vec<String> = conn
        .prepare(&format!(
            "SELECT DISTINCT player FROM stall_events
             WHERE action = 'bought'{owner_clause}
             ORDER BY player"
        ))
        .map_err(|e| e.to_string())?
        .query_map(rusqlite::params_from_iter(params.iter()), |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let players: Vec<String> = conn
        .prepare(&format!(
            "SELECT DISTINCT player FROM stall_events
             WHERE action != 'unknown' AND player != ''{owner_clause}
             ORDER BY player"
        ))
        .map_err(|e| e.to_string())?
        .query_map(rusqlite::params_from_iter(params.iter()), |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let items: Vec<String> = conn
        .prepare(&format!(
            "SELECT DISTINCT item FROM stall_events
             WHERE item IS NOT NULL AND action != 'unknown'{owner_clause}
             ORDER BY item"
        ))
        .map_err(|e| e.to_string())?
        .query_map(rusqlite::params_from_iter(params.iter()), |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let dates: Vec<String> = conn
        .prepare(&format!(
            "SELECT DISTINCT substr(event_at, 1, 10) FROM stall_events
             WHERE event_at IS NOT NULL{owner_clause}
             ORDER BY 1 DESC"
        ))
        .map_err(|e| e.to_string())?
        .query_map(rusqlite::params_from_iter(params.iter()), |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let actions: Vec<String> = conn
        .prepare(&format!(
            "SELECT DISTINCT action FROM stall_events
             WHERE action != 'unknown'{owner_clause} ORDER BY action"
        ))
        .map_err(|e| e.to_string())?
        .query_map(rusqlite::params_from_iter(params.iter()), |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(StallFilterOptions {
        buyers,
        players,
        items,
        dates,
        actions,
    })
}
