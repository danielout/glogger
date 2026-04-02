use super::DbPool;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct CharacterDeath {
    pub id: i64,
    pub character_name: String,
    pub server_name: String,
    pub died_at: String,
    pub killer_name: String,
    pub killer_entity_id: Option<String>,
    pub killing_ability: String,
    pub health_damage: i64,
    pub armor_damage: i64,
    pub area: Option<String>,
    pub damage_type: Option<String>,
}

#[tauri::command]
pub fn get_character_deaths(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<Vec<CharacterDeath>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, character_name, server_name, died_at, killer_name,
                    killer_entity_id, killing_ability, health_damage, armor_damage,
                    area, damage_type
             FROM character_deaths
             WHERE character_name = ?1 AND server_name = ?2
             ORDER BY died_at DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![character_name, server_name], |row| {
            Ok(CharacterDeath {
                id: row.get(0)?,
                character_name: row.get(1)?,
                server_name: row.get(2)?,
                died_at: row.get(3)?,
                killer_name: row.get(4)?,
                killer_entity_id: row.get(5)?,
                killing_ability: row.get(6)?,
                health_damage: row.get(7)?,
                armor_damage: row.get(8)?,
                area: row.get(9)?,
                damage_type: row.get(10)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read results: {e}"))
}

#[derive(Debug, Clone, Serialize)]
pub struct DeathDamageSource {
    pub id: i64,
    pub death_id: i64,
    pub event_order: i64,
    pub timestamp: String,
    pub attacker_name: String,
    pub attacker_entity_id: Option<String>,
    pub ability_name: String,
    pub health_damage: i64,
    pub armor_damage: i64,
    pub is_crit: bool,
}

#[tauri::command]
pub fn get_death_damage_sources(
    db: State<'_, DbPool>,
    death_id: i64,
) -> Result<Vec<DeathDamageSource>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, death_id, event_order, timestamp, attacker_name,
                    attacker_entity_id, ability_name, health_damage, armor_damage, is_crit
             FROM death_damage_sources
             WHERE death_id = ?1
             ORDER BY event_order ASC",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![death_id], |row| {
            Ok(DeathDamageSource {
                id: row.get(0)?,
                death_id: row.get(1)?,
                event_order: row.get(2)?,
                timestamp: row.get(3)?,
                attacker_name: row.get(4)?,
                attacker_entity_id: row.get(5)?,
                ability_name: row.get(6)?,
                health_damage: row.get(7)?,
                armor_damage: row.get(8)?,
                is_crit: row.get(9)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read results: {e}"))
}
