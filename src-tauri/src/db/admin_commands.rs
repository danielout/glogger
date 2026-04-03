use super::{DbConnection, DbPool};
use rusqlite::params;
use serde::{Deserialize, Serialize};
/// Advanced database administration commands
use tauri::State;

// ── Database Statistics ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct DatabaseStats {
    pub total_size_bytes: i64,
    pub cdn_size_bytes: i64,
    pub player_data_size_bytes: i64,
    pub market_prices_count: i64,
    pub sales_history_count: i64,
    pub survey_sessions_count: i64,
    pub event_log_count: i64,
}

#[tauri::command]
pub fn get_database_stats(db: State<'_, DbPool>) -> Result<DatabaseStats, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    // Get page count and page size to calculate total DB size
    let page_count: i64 = conn
        .query_row("PRAGMA page_count", [], |row| row.get(0))
        .map_err(|e| format!("Failed to get page count: {e}"))?;

    let page_size: i64 = conn
        .query_row("PRAGMA page_size", [], |row| row.get(0))
        .map_err(|e| format!("Failed to get page size: {e}"))?;

    let total_size_bytes = page_count * page_size;

    // Estimate CDN data size (items, skills, abilities, recipes, npcs, quests)
    let cdn_tables = vec![
        "items",
        "skills",
        "abilities",
        "recipes",
        "recipe_ingredients",
        "npcs",
        "npc_skills",
        "quests",
    ];
    let mut cdn_size_bytes = 0i64;

    for table in cdn_tables {
        let query = format!("SELECT SUM(pgsize) FROM dbstat WHERE name = '{}'", table);
        let size: Option<i64> = conn.query_row(&query, [], |row| row.get(0)).ok();
        cdn_size_bytes += size.unwrap_or(0);
    }

    // Get player data counts
    let market_prices_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM market_prices", [], |row| row.get(0))
        .unwrap_or(0);

    let sales_history_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM sales_history", [], |row| row.get(0))
        .unwrap_or(0);

    let survey_sessions_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM survey_sessions", [], |row| row.get(0))
        .unwrap_or(0);

    let event_log_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM event_log", [], |row| row.get(0))
        .unwrap_or(0);

    // Estimate player data size
    let player_data_size_bytes = total_size_bytes - cdn_size_bytes;

    Ok(DatabaseStats {
        total_size_bytes,
        cdn_size_bytes,
        player_data_size_bytes,
        market_prices_count,
        sales_history_count,
        survey_sessions_count,
        event_log_count,
    })
}

// ── Force Rebuild CDN Tables ──────────────────────────────────────────────────

#[tauri::command]
pub async fn force_rebuild_cdn_tables(
    db: State<'_, DbPool>,
    cdn_state: State<'_, crate::cdn_commands::GameDataState>,
) -> Result<String, String> {
    let mut conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    // Get the current game data from memory
    let data = cdn_state.read().await;

    if data.version == 0 {
        return Err(
            "No CDN data loaded. Please wait for initial data load or force refresh CDN."
                .to_string(),
        );
    }

    // Persist to database (this clears and rebuilds CDN tables)
    crate::db::cdn_persistence::persist_cdn_data(&mut conn, &data)
        .map_err(|e| format!("Failed to rebuild CDN tables: {e}"))?;

    Ok(format!(
        "CDN tables rebuilt successfully from version {}",
        data.version
    ))
}

// ── Purge Player Data ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct PurgeOptions {
    pub older_than_days: Option<u32>,
    pub purge_all: bool,
}

#[derive(Serialize)]
pub struct PurgeResult {
    pub market_prices_deleted: usize,
    pub sales_deleted: usize,
    pub survey_sessions_deleted: usize,
    pub events_deleted: usize,
}

#[tauri::command]
pub fn purge_player_data(
    db: State<'_, DbPool>,
    options: PurgeOptions,
) -> Result<PurgeResult, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let (market_prices_deleted, sales_deleted, survey_sessions_deleted, events_deleted) =
        if options.purge_all {
            // Purge everything
            let market_prices = conn
                .execute("DELETE FROM market_prices", [])
                .map_err(|e| format!("Failed to purge market prices: {e}"))?;

            let sales = conn
                .execute("DELETE FROM sales_history", [])
                .map_err(|e| format!("Failed to purge sales history: {e}"))?;

            let surveys = conn
                .execute("DELETE FROM survey_sessions", [])
                .map_err(|e| format!("Failed to purge survey sessions: {e}"))?;

            let events = conn
                .execute("DELETE FROM event_log", [])
                .map_err(|e| format!("Failed to purge event log: {e}"))?;

            (market_prices, sales, surveys, events)
        } else if let Some(days) = options.older_than_days {
            // Purge entries older than X days
            let cutoff = format!("-{} days", days);

            let market_prices = conn
                .execute(
                    "DELETE FROM market_prices WHERE observed_at < datetime('now', ?1)",
                    params![cutoff],
                )
                .map_err(|e| format!("Failed to purge old market prices: {e}"))?;

            let sales = conn
                .execute(
                    "DELETE FROM sales_history WHERE sold_at < datetime('now', ?1)",
                    params![cutoff],
                )
                .map_err(|e| format!("Failed to purge old sales: {e}"))?;

            let surveys = conn
                .execute(
                    "DELETE FROM survey_sessions WHERE start_time < datetime('now', ?1)",
                    params![cutoff],
                )
                .map_err(|e| format!("Failed to purge old surveys: {e}"))?;

            let events = conn
                .execute(
                    "DELETE FROM event_log WHERE created_at < datetime('now', ?1)",
                    params![cutoff],
                )
                .map_err(|e| format!("Failed to purge old events: {e}"))?;

            (market_prices, sales, surveys, events)
        } else {
            return Err("Must specify either purge_all or older_than_days".to_string());
        };

    // Run VACUUM to reclaim space
    conn.execute_batch("VACUUM")
        .map_err(|e| format!("Failed to vacuum database: {e}"))?;

    Ok(PurgeResult {
        market_prices_deleted,
        sales_deleted,
        survey_sessions_deleted,
        events_deleted,
    })
}

// ── Auto-Purge Settings ───────────────────────────────────────────────────────

/// Check and perform auto-purge if needed based on settings
/// This should be called periodically (e.g., on app startup)
#[allow(dead_code)]
pub fn check_auto_purge(
    conn: &DbConnection,
    auto_purge_days: Option<u32>,
) -> Result<PurgeResult, String> {
    if let Some(days) = auto_purge_days {
        let cutoff = format!("-{} days", days);

        let market_prices = conn
            .execute(
                "DELETE FROM market_prices WHERE observed_at < datetime('now', ?1)",
                params![cutoff],
            )
            .map_err(|e| format!("Auto-purge market prices failed: {e}"))?;

        let sales = conn
            .execute(
                "DELETE FROM sales_history WHERE sold_at < datetime('now', ?1)",
                params![cutoff],
            )
            .map_err(|e| format!("Auto-purge sales failed: {e}"))?;

        let surveys = conn
            .execute(
                "DELETE FROM survey_sessions WHERE start_time < datetime('now', ?1)",
                params![cutoff],
            )
            .map_err(|e| format!("Auto-purge surveys failed: {e}"))?;

        let events = conn
            .execute(
                "DELETE FROM event_log WHERE created_at < datetime('now', ?1)",
                params![cutoff],
            )
            .map_err(|e| format!("Auto-purge events failed: {e}"))?;

        if market_prices > 0 || sales > 0 || surveys > 0 || events > 0 {
            eprintln!(
                "Auto-purge: Deleted {} market prices, {} sales, {} surveys, {} events (older than {} days)",
                market_prices, sales, surveys, events, days
            );
        }

        Ok(PurgeResult {
            market_prices_deleted: market_prices,
            sales_deleted: sales,
            survey_sessions_deleted: surveys,
            events_deleted: events,
        })
    } else {
        Ok(PurgeResult {
            market_prices_deleted: 0,
            sales_deleted: 0,
            survey_sessions_deleted: 0,
            events_deleted: 0,
        })
    }
}
