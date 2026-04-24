use super::DbPool;
use serde::Serialize;
use tauri::State;

// ── Output types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct HoplologyStudy {
    pub id: i64,
    pub character_name: String,
    pub server_name: String,
    pub item_name: String,
    pub studied_at: String,
    pub source: String,
}

#[derive(Serialize)]
pub struct HoplologyStats {
    pub total_studied: u32,
    pub last_studied_at: Option<String>,
    pub last_studied_item: Option<String>,
}

// ── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_hoplology_studies(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<HoplologyStudy>, String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, character_name, server_name, item_name, studied_at, source
             FROM hoplology_studies
             WHERE character_name = ?1 AND server_name = ?2
             ORDER BY studied_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([&character_name, &server_name], |row| {
            Ok(HoplologyStudy {
                id: row.get(0)?,
                character_name: row.get(1)?,
                server_name: row.get(2)?,
                item_name: row.get(3)?,
                studied_at: row.get(4)?,
                source: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut studies = Vec::new();
    for row in rows {
        studies.push(row.map_err(|e| e.to_string())?);
    }
    Ok(studies)
}

#[tauri::command]
pub fn get_hoplology_stats(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<HoplologyStats, String> {
    let conn = db.get().map_err(|e| e.to_string())?;

    let total_studied: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM hoplology_studies WHERE character_name = ?1 AND server_name = ?2",
            [&character_name, &server_name],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let last = conn
        .query_row(
            "SELECT studied_at, item_name FROM hoplology_studies
             WHERE character_name = ?1 AND server_name = ?2
             ORDER BY studied_at DESC LIMIT 1",
            [&character_name, &server_name],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .ok();

    Ok(HoplologyStats {
        total_studied,
        last_studied_at: last.as_ref().map(|(ts, _)| ts.clone()),
        last_studied_item: last.map(|(_, name)| name),
    })
}

// ── Internal helper (called from coordinator, not a Tauri command) ──────────

/// Insert or ignore a hoplology study record. Returns the row id if newly inserted.
pub fn insert_hoplology_study(
    conn: &rusqlite::Connection,
    character_name: &str,
    server_name: &str,
    item_name: &str,
    studied_at: &str,
) -> Result<Option<i64>, rusqlite::Error> {
    let changed = conn.execute(
        "INSERT OR IGNORE INTO hoplology_studies (character_name, server_name, item_name, studied_at, source)
         VALUES (?1, ?2, ?3, ?4, 'auto')",
        rusqlite::params![character_name, server_name, item_name, studied_at],
    )?;

    if changed > 0 {
        Ok(Some(conn.last_insert_rowid()))
    } else {
        Ok(None) // Already studied
    }
}
