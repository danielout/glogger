use super::DbPool;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct CharacterResuscitation {
    pub id: i64,
    pub character_name: String,
    pub server_name: String,
    pub occurred_at: String,
    pub caster_name: String,
    pub target_name: String,
    pub success: bool,
    pub area: Option<String>,
}

#[tauri::command]
pub fn get_character_resuscitations(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<CharacterResuscitation>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, character_name, server_name, occurred_at, caster_name,
                    target_name, success, area
             FROM character_resuscitations
             WHERE character_name = ?1 AND server_name = ?2
             ORDER BY occurred_at DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![character_name, server_name], |row| {
            Ok(CharacterResuscitation {
                id: row.get(0)?,
                character_name: row.get(1)?,
                server_name: row.get(2)?,
                occurred_at: row.get(3)?,
                caster_name: row.get(4)?,
                target_name: row.get(5)?,
                success: row.get(6)?,
                area: row.get(7)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read results: {e}"))
}
