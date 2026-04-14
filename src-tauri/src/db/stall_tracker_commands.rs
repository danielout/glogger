use super::DbPool;
use crate::stall_aggregations::{
    aggregate_inventory, aggregate_revenue, Granularity, InventoryEvent, InventoryResult,
    RevenueEvent, RevenueResult,
};
use rusqlite::types::Value;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

/// Serialization lock for stall-tracker DB mutations.
///
/// Both the coordinator's live-ingest path (`insert_stall_events`) and future
/// Clear/Import commands acquire this around their DB writes so a Clear
/// can never interleave with a concurrent PlayerShopLog insert and leave
/// partial or orphaned rows behind. Critical sections are intentionally
/// small (one batch / one DELETE); contention is invisible in practice.
#[derive(Default)]
pub struct StallOpsLock(pub Mutex<()>);

/// Input shape for a single stall event, built by the coordinator from the
/// shop log parser's output.
///
/// `owner` is `Option<String>` to keep the same struct usable by both the
/// live-ingest path (which always stamps `Some(active_character)`) and the
/// future Import command (which may pass the parser's advisory owner hint
/// or fall back to the caller-supplied current owner). In the live path,
/// `None` is never a valid input — the coordinator early-returns if no
/// active character is set.
#[derive(Debug, Clone)]
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

/// Bulk-insert stall events, skipping duplicates via the UNIQUE key
/// `(event_timestamp, raw_message, entry_index)`. Returns the number of
/// rows actually inserted (i.e. not skipped by `INSERT OR IGNORE`).
///
/// Holds `StallOpsLock` for the duration of the transaction so Clear
/// cannot interleave.
pub fn insert_stall_events(
    db: &DbPool,
    ops_lock: &StallOpsLock,
    events: &[StallEventInput],
) -> Result<usize, String> {
    if events.is_empty() {
        return Ok(0);
    }

    let _guard = ops_lock
        .0
        .lock()
        .map_err(|e| format!("StallOpsLock poisoned: {e}"))?;

    let mut conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to begin transaction: {e}"))?;

    let mut inserted = 0usize;
    {
        let mut stmt = tx
            .prepare(
                "INSERT OR IGNORE INTO stall_events (
                    event_timestamp, event_at, log_timestamp, log_title,
                    action, player, owner, item, quantity,
                    price_unit, price_total, raw_message, entry_index
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            )
            .map_err(|e| format!("Failed to prepare insert: {e}"))?;

        for ev in events {
            let changed = stmt
                .execute(rusqlite::params![
                    ev.event_timestamp,
                    ev.event_at,
                    ev.log_timestamp,
                    ev.log_title,
                    ev.action,
                    ev.player,
                    ev.owner,
                    ev.item,
                    ev.quantity,
                    ev.price_unit,
                    ev.price_total,
                    ev.raw_message,
                    ev.entry_index,
                ])
                .map_err(|e| format!("Failed to insert stall event: {e}"))?;
            inserted += changed;
        }
    }

    tx.commit()
        .map_err(|e| format!("Failed to commit transaction: {e}"))?;

    Ok(inserted)
}

// ============================================================
// Phase 3 — Read/mutate commands
// ============================================================

/// A single stall event row as returned to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct StallEvent {
    pub id: i64,
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
    pub ignored: bool,
    pub created_at: String,
}

/// Filter shape shared by list, stats, and aggregation queries. Every field
/// is optional; `owner` is the only one that should always be populated by
/// the frontend (missing/empty owner → empty results, by design).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StallEventsFilters {
    pub owner: Option<String>,
    pub action: Option<String>,
    pub player: Option<String>,
    pub item: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub include_ignored: Option<bool>,
}

/// Parameters for `get_stall_events` (pagination + sort + filters).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StallEventsParams {
    #[serde(flatten)]
    pub filters: StallEventsFilters,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    /// Override the default `action != 'unknown'` filter. Pass `Some("unknown")`
    /// from the Shop Log modal's "show unknowns" toggle.
    pub force_action: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StallEventsPage {
    pub rows: Vec<StallEvent>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StallStats {
    pub total_sales: i64,
    pub total_revenue: i64,
    pub unique_buyers: i64,
    pub unique_items: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StallFilterOptions {
    pub buyers: Vec<String>,
    pub players: Vec<String>,
    pub items: Vec<String>,
    pub dates: Vec<String>,
    pub actions: Vec<String>,
}

fn opt_nonempty(s: &Option<String>) -> Option<&str> {
    s.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

/// Build a SQL WHERE clause + bound params from a filter struct.
///
/// Encodes two non-obvious rules from plan §5.1:
///
/// 1. **Default action filter excludes `'unknown'`** — list views never show
///    unparseable entries. Callers that want unknowns pass
///    `force_action = Some("unknown")` (or the caller sets
///    `filters.action = Some("unknown")` explicitly).
/// 2. **Default `include_ignored = true`** — list views *do* show ignored
///    rows (greyed out). Aggregations pass `false` to exclude them.
///
/// Returns `None` if `owner` is missing/empty — callers should return empty
/// results rather than running an unscoped query.
fn build_filter_where(
    filters: &StallEventsFilters,
    force_action: Option<&str>,
) -> Option<(String, Vec<Value>)> {
    let owner = opt_nonempty(&filters.owner)?;

    let mut sql = String::from(" WHERE 1=1");
    let mut params: Vec<Value> = Vec::new();

    sql.push_str(" AND owner = ?");
    params.push(Value::Text(owner.to_string()));

    if let Some(force) = force_action {
        sql.push_str(" AND action = ?");
        params.push(Value::Text(force.to_string()));
    } else if let Some(action) = opt_nonempty(&filters.action) {
        sql.push_str(" AND action = ?");
        params.push(Value::Text(action.to_string()));
    } else {
        sql.push_str(" AND action != 'unknown'");
    }

    if let Some(player) = opt_nonempty(&filters.player) {
        sql.push_str(" AND player = ?");
        params.push(Value::Text(player.to_string()));
    }

    if let Some(item) = opt_nonempty(&filters.item) {
        sql.push_str(" AND item = ?");
        params.push(Value::Text(item.to_string()));
    }

    // Date inputs MUST be `YYYY-MM-DD` (10 chars). Reject anything else
    // loudly via empty result rather than producing the malformed bound
    // `"2026-04-13 14:29:00 00:00:00"` if the frontend ever ships a full
    // ISO timestamp by mistake.
    if let Some(from) = opt_nonempty(&filters.date_from) {
        if from.len() != 10 {
            return None;
        }
        sql.push_str(" AND event_at >= ?");
        params.push(Value::Text(format!("{from} 00:00:00")));
    }

    if let Some(to) = opt_nonempty(&filters.date_to) {
        if to.len() != 10 {
            return None;
        }
        sql.push_str(" AND event_at <= ?");
        params.push(Value::Text(format!("{to} 23:59:59")));
    }

    let include_ignored = filters.include_ignored.unwrap_or(true);
    if !include_ignored {
        sql.push_str(" AND ignored = 0");
    }

    Some((sql, params))
}

/// Whitelisted sort column + direction + stable tiebreakers.
fn resolve_sort(sort_by: Option<&str>, sort_dir: Option<&str>) -> String {
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
    let dir = match sort_dir.unwrap_or("desc").to_ascii_lowercase().as_str() {
        "asc" => "ASC",
        _ => "DESC",
    };
    // Stable tiebreakers ensure deterministic pagination across refetches.
    format!("{col} {dir}, event_at DESC, entry_index DESC, id DESC")
}

fn row_to_event(row: &rusqlite::Row) -> rusqlite::Result<StallEvent> {
    Ok(StallEvent {
        id: row.get(0)?,
        event_timestamp: row.get(1)?,
        event_at: row.get(2)?,
        log_timestamp: row.get(3)?,
        log_title: row.get(4)?,
        action: row.get(5)?,
        player: row.get(6)?,
        owner: row.get(7)?,
        item: row.get(8)?,
        quantity: row.get(9)?,
        price_unit: row.get(10)?,
        price_total: row.get(11)?,
        raw_message: row.get(12)?,
        entry_index: row.get(13)?,
        ignored: row.get::<_, i64>(14)? != 0,
        created_at: row.get(15)?,
    })
}

const STALL_EVENT_COLUMNS: &str = "id, event_timestamp, event_at, log_timestamp, log_title, \
    action, player, owner, item, quantity, price_unit, price_total, raw_message, entry_index, \
    ignored, created_at";

/// Paginated list of stall events with filters + sort.
///
/// Empty `owner` returns an empty page rather than leaking cross-character
/// data. `limit` is clamped to `[1, 10_000]`, `offset` floored at 0.
#[tauri::command]
pub fn get_stall_events(
    db: State<'_, DbPool>,
    params: StallEventsParams,
) -> Result<StallEventsPage, String> {
    let Some((where_sql, bound)) =
        build_filter_where(&params.filters, params.force_action.as_deref())
    else {
        return Ok(StallEventsPage {
            rows: Vec::new(),
            total_count: 0,
        });
    };

    let limit = params.limit.unwrap_or(500).clamp(1, 10_000);
    let offset = params.offset.unwrap_or(0).max(0);
    let order_by = resolve_sort(params.sort_by.as_deref(), params.sort_dir.as_deref());

    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    // Count query uses the same WHERE bound params.
    let count_sql = format!("SELECT COUNT(*) FROM stall_events{where_sql}");
    let total_count: i64 = conn
        .query_row(
            &count_sql,
            rusqlite::params_from_iter(bound.iter()),
            |row| row.get(0),
        )
        .map_err(|e| format!("Count query failed: {e}"))?;

    let list_sql = format!(
        "SELECT {STALL_EVENT_COLUMNS} FROM stall_events{where_sql} \
         ORDER BY {order_by} LIMIT ? OFFSET ?"
    );
    let mut list_params = bound;
    list_params.push(Value::Integer(limit));
    list_params.push(Value::Integer(offset));

    let mut stmt = conn
        .prepare(&list_sql)
        .map_err(|e| format!("Failed to prepare list query: {e}"))?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(list_params.iter()), row_to_event)
        .map_err(|e| format!("List query failed: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read list results: {e}"))?;

    Ok(StallEventsPage { rows, total_count })
}

/// Aggregate stats for the Sales header. Always forces `action = 'bought'`
/// and `include_ignored = false`, regardless of caller input, so the numbers
/// match "revenue from sales I haven't muted".
#[tauri::command]
pub fn get_stall_stats(
    db: State<'_, DbPool>,
    filters: StallEventsFilters,
) -> Result<StallStats, String> {
    // Stats are by definition over `bought + non-ignored`. Caller-supplied
    // `action` / `include_ignored` are intentionally overridden — the date
    // range, owner, player, and item filters still flow through.
    let mut filters = filters;
    filters.include_ignored = Some(false);

    let Some((where_sql, bound)) = build_filter_where(&filters, Some("bought")) else {
        return Ok(StallStats {
            total_sales: 0,
            total_revenue: 0,
            unique_buyers: 0,
            unique_items: 0,
        });
    };

    let sql = format!(
        "SELECT COUNT(*) AS total_sales, \
                COALESCE(SUM(price_total), 0) AS total_revenue, \
                COUNT(DISTINCT player) AS unique_buyers, \
                COUNT(DISTINCT item) AS unique_items \
         FROM stall_events{where_sql}"
    );

    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    conn.query_row(&sql, rusqlite::params_from_iter(bound.iter()), |row| {
        Ok(StallStats {
            total_sales: row.get(0)?,
            total_revenue: row.get(1)?,
            unique_buyers: row.get(2)?,
            unique_items: row.get(3)?,
        })
    })
    .map_err(|e| format!("Stats query failed: {e}"))
}

/// Distinct value lists for filter dropdowns, scoped to a single owner.
/// `buyers` = `DISTINCT player WHERE action='bought'`; `players` =
/// `DISTINCT player WHERE action != 'unknown'` (includes the stall owner).
#[tauri::command]
pub fn get_stall_filter_options(
    db: State<'_, DbPool>,
    owner: Option<String>,
) -> Result<StallFilterOptions, String> {
    let Some(owner) = opt_nonempty(&owner).map(str::to_string) else {
        return Ok(StallFilterOptions {
            buyers: Vec::new(),
            players: Vec::new(),
            items: Vec::new(),
            dates: Vec::new(),
            actions: Vec::new(),
        });
    };

    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let collect_strings = |sql: &str| -> Result<Vec<String>, String> {
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| format!("Failed to prepare filter query: {e}"))?;
        let rows = stmt
            .query_map(rusqlite::params![owner], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Filter query failed: {e}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to read filter results: {e}"))
    };

    let buyers = collect_strings(
        "SELECT DISTINCT player FROM stall_events \
         WHERE owner = ?1 AND action = 'bought' \
         ORDER BY player",
    )?;
    let players = collect_strings(
        "SELECT DISTINCT player FROM stall_events \
         WHERE owner = ?1 AND action != 'unknown' \
         ORDER BY player",
    )?;
    let items = collect_strings(
        "SELECT DISTINCT item FROM stall_events \
         WHERE owner = ?1 AND item IS NOT NULL \
         ORDER BY item",
    )?;
    let dates = collect_strings(
        "SELECT DISTINCT substr(event_at, 1, 10) AS d FROM stall_events \
         WHERE owner = ?1 AND event_at IS NOT NULL \
         ORDER BY d DESC",
    )?;
    let actions = collect_strings(
        "SELECT DISTINCT action FROM stall_events \
         WHERE owner = ?1 \
         ORDER BY action",
    )?;

    Ok(StallFilterOptions {
        buyers,
        players,
        items,
        dates,
        actions,
    })
}

/// Toggle the soft-mute flag on a single event. `ignored = true` excludes the
/// row from all aggregations (but not list views).
///
/// The `owner` parameter is defense-in-depth — the frontend already passes
/// IDs only from the currently-loaded (owner-scoped) list, but enforcing
/// `WHERE id = ?1 AND owner = ?2` at the SQL level closes the door on any
/// future caller that might forget the scope.
#[tauri::command]
pub fn toggle_stall_event_ignored(
    db: State<'_, DbPool>,
    id: i64,
    ignored: bool,
    owner: Option<String>,
) -> Result<(), String> {
    let owner = opt_nonempty(&owner)
        .ok_or_else(|| "toggle_stall_event_ignored requires an owner".to_string())?
        .to_string();
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;
    conn.execute(
        "UPDATE stall_events SET ignored = ?1 WHERE id = ?2 AND owner = ?3",
        rusqlite::params![ignored as i64, id, owner],
    )
    .map_err(|e| format!("Toggle query failed: {e}"))?;
    Ok(())
}

// ============================================================
// Phase 4 — Aggregation commands
// ============================================================

/// Parameters for `get_stall_revenue`. Granularity is a string on the wire
/// for natural JS interop (`"daily"` / `"weekly"` / `"monthly"`).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StallRevenueParams {
    pub owner: Option<String>,
    pub granularity: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub player: Option<String>,
    pub item: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StallInventoryParams {
    pub owner: Option<String>,
    pub period_days: Option<i64>,
}

fn parse_granularity(s: &str) -> Granularity {
    match s.to_ascii_lowercase().as_str() {
        "weekly" => Granularity::Weekly,
        "monthly" => Granularity::Monthly,
        _ => Granularity::Daily,
    }
}

/// Revenue pivot for the Revenue tab. Always over `bought + non-ignored`,
/// scoped to a single owner. Date range, buyer, and item filters flow
/// through. Returns an empty result on missing owner.
#[tauri::command]
pub fn get_stall_revenue(
    db: State<'_, DbPool>,
    params: StallRevenueParams,
) -> Result<RevenueResult, String> {
    let filters = StallEventsFilters {
        owner: params.owner,
        action: None,
        player: params.player,
        item: params.item,
        date_from: params.date_from,
        date_to: params.date_to,
        include_ignored: Some(false),
    };
    let Some((where_sql, bound)) = build_filter_where(&filters, Some("bought")) else {
        return Ok(RevenueResult {
            periods: Vec::new(),
            items: Vec::new(),
            cells: Vec::new(),
            row_totals: Vec::new(),
            col_totals: Vec::new(),
            grand_total: 0,
        });
    };

    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    // Only rows with item AND event_at can contribute to the pivot.
    let sql = format!(
        "SELECT item, event_at, price_total FROM stall_events{where_sql} \
         AND item IS NOT NULL AND event_at IS NOT NULL AND price_total IS NOT NULL"
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare revenue query: {e}"))?;
    let events = stmt
        .query_map(rusqlite::params_from_iter(bound.iter()), |row| {
            Ok(RevenueEvent {
                item: row.get::<_, String>(0)?,
                event_at: row.get::<_, String>(1)?,
                price_total: row.get::<_, i64>(2)?,
            })
        })
        .map_err(|e| format!("Revenue query failed: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read revenue results: {e}"))?;

    let granularity = params
        .granularity
        .as_deref()
        .map(parse_granularity)
        .unwrap_or(Granularity::Daily);

    Ok(aggregate_revenue(events, granularity))
}

/// Inventory tier-stack snapshot for the Inventory tab. Replays every
/// non-ignored event for the owner, in `(event_at ASC, id ASC)` order so
/// same-minute `visible` events arrive after their `added` counterpart.
#[tauri::command]
pub fn get_stall_inventory(
    db: State<'_, DbPool>,
    params: StallInventoryParams,
) -> Result<InventoryResult, String> {
    let Some(owner) = opt_nonempty(&params.owner).map(str::to_string) else {
        return Ok(InventoryResult {
            items: Vec::new(),
            active_dates: Vec::new(),
            estimated_value: 0,
            total_sold: 0,
            avg_daily_revenue: 0.0,
        });
    };

    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    // CRITICAL: ORDER BY event_at ASC, id ASC. Without the id tiebreaker,
    // `visible` events could be processed before `added` events within the
    // same minute, leaving the tier unpriced.
    let sql = "SELECT item, event_at, action, quantity, price_unit, price_total \
               FROM stall_events \
               WHERE owner = ?1 AND ignored = 0 AND item IS NOT NULL AND event_at IS NOT NULL \
                 AND action != 'unknown' AND action != 'collected' \
               ORDER BY event_at ASC, id ASC";
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("Failed to prepare inventory query: {e}"))?;
    let events = stmt
        .query_map(rusqlite::params![owner], |row| {
            Ok(InventoryEvent {
                item: row.get::<_, String>(0)?,
                event_at: row.get::<_, String>(1)?,
                action: row.get::<_, String>(2)?,
                quantity: row.get::<_, i64>(3)?,
                price_unit: row.get::<_, Option<f64>>(4)?,
                price_total: row.get::<_, Option<i64>>(5)?,
            })
        })
        .map_err(|e| format!("Inventory query failed: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read inventory results: {e}"))?;

    let period_days = params.period_days.unwrap_or(7);
    Ok(aggregate_inventory(events, period_days))
}

/// Delete every stall event for a single character. Requires non-empty owner —
/// we never want a typo to wipe all data across every alt. Holds `StallOpsLock`
/// so a concurrent live-ingest batch can't write after the DELETE.
#[tauri::command]
pub fn clear_stall_events(
    db: State<'_, DbPool>,
    ops_lock: State<'_, StallOpsLock>,
    owner: Option<String>,
) -> Result<usize, String> {
    let owner = opt_nonempty(&owner)
        .ok_or_else(|| "clear_stall_events requires an owner".to_string())?
        .to_string();

    let _guard = ops_lock
        .0
        .lock()
        .map_err(|e| format!("StallOpsLock poisoned: {e}"))?;

    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;
    let deleted = conn
        .execute(
            "DELETE FROM stall_events WHERE owner = ?1",
            rusqlite::params![owner],
        )
        .map_err(|e| format!("Clear query failed: {e}"))?;
    Ok(deleted)
}
