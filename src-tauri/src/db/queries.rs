use rusqlite::{params, Result, OptionalExtension};
use super::DbConnection;

/// Player data query functions
pub mod player_data {
    use super::*;

    /// Insert a market price observation
    pub fn insert_market_price(
        conn: &DbConnection,
        item_id: u32,
        price: f64,
        quantity: u32,
        vendor_type: &str,
        vendor_name: Option<&str>,
        notes: Option<&str>,
    ) -> Result<i64> {
        conn.execute(
            "INSERT INTO market_prices (item_id, price, quantity, vendor_type, vendor_name, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![item_id, price, quantity, vendor_type, vendor_name, notes],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Record a sale in sales history
    pub fn insert_sale(
        conn: &DbConnection,
        item_id: u32,
        quantity: u32,
        sale_price: f64,
        sale_method: &str,
        buyer_name: Option<&str>,
        notes: Option<&str>,
    ) -> Result<i64> {
        conn.execute(
            "INSERT INTO sales_history (item_id, quantity, sale_price, sale_method, buyer_name, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![item_id, quantity, sale_price, sale_method, buyer_name, notes],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Log a generic event
    pub fn log_event(
        conn: &DbConnection,
        event_type: &str,
        event_data: &str, // JSON string
    ) -> Result<i64> {
        conn.execute(
            "INSERT INTO event_log (event_type, event_data) VALUES (?1, ?2)",
            params![event_type, event_data],
        )?;
        Ok(conn.last_insert_rowid())
    }
}

/// Survey-related queries
/// CDN data persistence queries
pub mod cdn_data {
    use super::*;

    /// Check if CDN data is loaded and get version
    pub fn get_cdn_version(conn: &DbConnection) -> Result<Option<u32>> {
        let version: Option<u32> = conn
            .query_row("SELECT version FROM cdn_version WHERE id = 1", [], |row| {
                row.get(0)
            })
            .optional()?;
        Ok(version)
    }

    /// Update CDN version (upsert)
    pub fn set_cdn_version(conn: &DbConnection, version: u32) -> Result<()> {
        conn.execute(
            "INSERT OR REPLACE INTO cdn_version (id, version) VALUES (1, ?1)",
            params![version],
        )?;
        Ok(())
    }

    /// Clear all CDN data (for refresh)
    pub fn clear_cdn_data(conn: &DbConnection) -> Result<()> {
        conn.execute_batch(
            "DELETE FROM recipe_ingredients;
             DELETE FROM recipes;
             DELETE FROM abilities;
             DELETE FROM skills;
             DELETE FROM items;
             DELETE FROM npc_skills;
             DELETE FROM npcs;
             DELETE FROM quests;"
        )?;
        Ok(())
    }
}

/// Log file position tracking (unified for all log types)
pub mod log_positions {
    use super::*;

    /// Get the last processed position for a log file
    pub fn get_position(conn: &DbConnection, file_path: &str) -> Result<u64> {
        let position: i64 = conn
            .query_row(
                "SELECT last_position FROM log_file_positions WHERE file_path = ?1",
                params![file_path],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok(position as u64)
    }

    /// Update the last processed position for a log file
    pub fn update_position(
        conn: &DbConnection,
        file_path: &str,
        file_type: &str,
        position: u64,
        player_name: Option<&str>,
        metadata: Option<&str>,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO log_file_positions (file_path, file_type, last_position, player_name, metadata, last_processed)
             VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP)
             ON CONFLICT(file_path) DO UPDATE SET
                last_position = ?3,
                player_name = COALESCE(?4, player_name),
                metadata = COALESCE(?5, metadata),
                last_processed = CURRENT_TIMESTAMP",
            params![file_path, file_type, position as i64, player_name, metadata],
        )?;

        Ok(())
    }

    /// Get all positions for a specific file type
    pub fn get_positions_by_type(conn: &DbConnection, file_type: &str) -> Result<Vec<(String, u64)>> {
        let mut stmt = conn.prepare(
            "SELECT file_path, last_position FROM log_file_positions WHERE file_type = ?1 ORDER BY last_processed DESC"
        )?;

        let rows = stmt.query_map([file_type], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u64))
        })?;

        let mut positions = Vec::new();
        for row in rows {
            positions.push(row?);
        }

        Ok(positions)
    }
}
