use super::DbPool;
use std::sync::Mutex;

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
