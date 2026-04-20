use super::DbPool;
use serde::{Deserialize, Serialize};
use tauri::State;

// ── Output types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct WordOfPower {
    pub id: i64,
    pub character_name: String,
    pub server_name: String,
    pub word: String,
    pub power_name: String,
    pub description: Option<String>,
    pub discovered_at: String,
    pub source: String,
}

// ── Input types ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AddWordInput {
    pub word: String,
    pub power_name: String,
    pub description: Option<String>,
}

// ── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_words_of_power(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<WordOfPower>, String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, character_name, server_name, word, power_name, description, discovered_at, source
             FROM words_of_power
             WHERE character_name = ?1 AND server_name = ?2
             ORDER BY discovered_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([&character_name, &server_name], |row| {
            Ok(WordOfPower {
                id: row.get(0)?,
                character_name: row.get(1)?,
                server_name: row.get(2)?,
                word: row.get(3)?,
                power_name: row.get(4)?,
                description: row.get(5)?,
                discovered_at: row.get(6)?,
                source: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut words = Vec::new();
    for row in rows {
        words.push(row.map_err(|e| e.to_string())?);
    }
    Ok(words)
}

#[tauri::command]
pub fn add_word_of_power(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
    input: AddWordInput,
) -> Result<WordOfPower, String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    conn.execute(
        "INSERT INTO words_of_power (character_name, server_name, word, power_name, description, discovered_at, source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'manual')",
        rusqlite::params![
            &character_name,
            &server_name,
            &input.word,
            &input.power_name,
            &input.description,
            &now,
        ],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    Ok(WordOfPower {
        id,
        character_name,
        server_name,
        word: input.word,
        power_name: input.power_name,
        description: input.description,
        discovered_at: now,
        source: "manual".to_string(),
    })
}

#[tauri::command]
pub fn delete_word_of_power(db: State<'_, DbPool>, id: i64) -> Result<(), String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM words_of_power WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ── Internal helper (called from coordinator, not a Tauri command) ──────────

pub fn insert_word_of_power(
    conn: &rusqlite::Connection,
    character_name: &str,
    server_name: &str,
    word: &str,
    power_name: &str,
    description: Option<&str>,
    discovered_at: &str,
) -> Result<i64, rusqlite::Error> {
    conn.execute(
        "INSERT INTO words_of_power (character_name, server_name, word, power_name, description, discovered_at, source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'auto')",
        rusqlite::params![character_name, server_name, word, power_name, description, discovered_at],
    )?;
    Ok(conn.last_insert_rowid())
}
