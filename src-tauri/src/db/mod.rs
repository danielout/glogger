use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub mod admin_commands;
pub mod aggregate_commands;
pub mod build_planner_commands;
pub mod cdn_persistence;
pub mod character_commands;
pub mod chat_commands;
pub mod crafting_commands;
pub mod death_commands;
pub mod farming_commands;
pub mod resuscitate_commands;
pub mod game_state_commands;
pub mod gourmand_commands;
pub mod inventory_commands;
pub mod market_commands;
pub mod migrations;
pub mod player_commands;
pub mod price_helper_commands;
pub mod player_commands_survey_events;
pub mod queries;
pub mod survey_commands;
pub mod survey_sharing_commands;

pub type DbPool = r2d2::Pool<SqliteConnectionManager>;
pub type DbConnection = r2d2::PooledConnection<SqliteConnectionManager>;

/// Initialize the database pool with the given path.
/// `tz_offset_seconds` is needed for one-time migration to fix historical timestamps.
pub fn init_pool(db_path: PathBuf, tz_offset_seconds: Option<i32>) -> Result<DbPool, Box<dyn std::error::Error>> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let manager = SqliteConnectionManager::file(&db_path).with_init(|conn| {
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
                 PRAGMA busy_timeout=5000;
                 PRAGMA synchronous=NORMAL;
                 PRAGMA foreign_keys=ON;",
        )
    });
    let pool = r2d2::Pool::builder()
        .max_size(15) // Allow multiple concurrent connections
        .build(manager)?;

    // Run migrations on a connection
    let conn = pool.get()?;
    migrations::run_migrations(&conn, tz_offset_seconds)?;
    drop(conn); // Release connection back to pool

    Ok(pool)
}

/// Get current schema version
pub fn get_schema_version(conn: &Connection) -> Result<i32> {
    let version: i32 = conn
        .query_row(
            "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(version)
}

/// Record that a migration was applied
fn record_migration(conn: &Connection, version: i32) -> Result<()> {
    conn.execute(
        "INSERT INTO schema_migrations (version) VALUES (?1)",
        [version],
    )?;
    Ok(())
}
