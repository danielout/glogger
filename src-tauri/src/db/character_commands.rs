/// Tauri commands for character report import and querying

use std::collections::HashMap;
use tauri::State;
use serde::{Deserialize, Serialize};
use super::DbPool;

// ── JSON Deserialization Structs (match game's /outputcharacter format) ───────

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CharacterReport {
    pub character: String,
    pub server_name: String,
    pub timestamp: String,
    pub report: String,
    pub report_version: u32,
    pub race: String,
    pub skills: HashMap<String, SkillData>,
    pub recipe_completions: HashMap<String, i64>,
    pub current_stats: HashMap<String, f64>,
    pub currencies: HashMap<String, i64>,
    #[serde(rename = "NPCs")]
    pub npcs: HashMap<String, NpcFavorData>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SkillData {
    pub level: i32,
    pub bonus_levels: i32,
    pub xp_toward_next_level: i64,
    pub xp_needed_for_next_level: i64,
    pub abilities: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NpcFavorData {
    pub favor_level: String,
}

// ── Command Response Types ───────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ImportResult {
    pub character_name: String,
    pub server_name: String,
    pub snapshot_timestamp: String,
    pub skills_imported: usize,
    pub npcs_imported: usize,
    pub recipes_imported: usize,
    pub stats_imported: usize,
    pub currencies_imported: usize,
    pub was_duplicate: bool,
}

#[derive(Serialize)]
pub struct CharacterSnapshotSummary {
    pub id: i64,
    pub character_name: String,
    pub server_name: String,
    pub snapshot_timestamp: String,
    pub race: String,
    pub import_date: String,
    pub skill_count: usize,
}

#[derive(Serialize)]
pub struct SnapshotSkillLevel {
    pub skill_name: String,
    pub level: i32,
    pub bonus_levels: i32,
    pub xp_toward_next: i64,
    pub xp_needed_for_next: i64,
}

#[derive(Serialize)]
pub struct SnapshotNpcFavor {
    pub npc_key: String,
    pub favor_level: String,
}

#[derive(Serialize)]
pub struct SnapshotRecipeCompletion {
    pub recipe_key: String,
    pub completions: i64,
}

#[derive(Serialize)]
pub struct SnapshotStat {
    pub stat_key: String,
    pub value: f64,
}

#[derive(Serialize)]
pub struct SnapshotCurrency {
    pub currency_key: String,
    pub amount: i64,
}

#[derive(Serialize)]
pub struct CharacterInfo {
    pub character_name: String,
    pub server_name: String,
    pub latest_snapshot: String,
    pub snapshot_count: usize,
}

#[derive(Serialize)]
pub struct SkillDiff {
    pub skill_name: String,
    pub old_level: i32,
    pub new_level: i32,
    pub level_change: i32,
    pub old_xp: i64,
    pub new_xp: i64,
}

// ── Commands ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn import_character_report(
    db: State<'_, DbPool>,
    file_path: String,
) -> Result<ImportResult, String> {
    import_character_report_internal(&db, &file_path)
}

/// Internal import logic, callable without Tauri State wrapper
pub fn import_character_report_internal(
    db: &DbPool,
    file_path: &str,
) -> Result<ImportResult, String> {
    // 1. Read file
    let raw_json = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {e}"))?;

    // 2. Deserialize
    let report: CharacterReport = serde_json::from_str(&raw_json)
        .map_err(|e| format!("Failed to parse character report: {e}"))?;

    // 3. Validate report type
    if report.report != "CharacterSheet" {
        return Err(format!(
            "Wrong report type: expected \"CharacterSheet\", got \"{}\"",
            report.report
        ));
    }

    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    // 4. Begin transaction
    conn.execute("BEGIN", [])
        .map_err(|e| format!("Failed to begin transaction: {e}"))?;

    let result = (|| -> Result<ImportResult, String> {
        // 5. Insert snapshot (skip duplicates)
        conn.execute(
            "INSERT INTO character_snapshots (character_name, server_name, snapshot_timestamp, race, raw_json)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(character_name, server_name, snapshot_timestamp) DO NOTHING",
            rusqlite::params![
                report.character,
                report.server_name,
                report.timestamp,
                report.race,
                raw_json,
            ],
        ).map_err(|e| format!("Failed to insert snapshot: {e}"))?;

        // 6. Check if insert created a new row
        let changes = conn.changes();
        if changes == 0 {
            return Ok(ImportResult {
                character_name: report.character.clone(),
                server_name: report.server_name.clone(),
                snapshot_timestamp: report.timestamp.clone(),
                skills_imported: 0,
                npcs_imported: 0,
                recipes_imported: 0,
                stats_imported: 0,
                currencies_imported: 0,
                was_duplicate: true,
            });
        }

        let snapshot_id = conn.last_insert_rowid();

        // 7. Batch insert skill levels
        let mut skill_stmt = conn.prepare(
            "INSERT INTO character_skill_levels (snapshot_id, skill_name, level, bonus_levels, xp_toward_next, xp_needed_for_next)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        ).map_err(|e| format!("Failed to prepare skill insert: {e}"))?;

        for (skill_name, skill_data) in &report.skills {
            skill_stmt.execute(rusqlite::params![
                snapshot_id,
                skill_name,
                skill_data.level,
                skill_data.bonus_levels,
                skill_data.xp_toward_next_level,
                skill_data.xp_needed_for_next_level,
            ]).map_err(|e| format!("Failed to insert skill {skill_name}: {e}"))?;
        }

        // 8. Batch insert NPC favor
        let mut npc_stmt = conn.prepare(
            "INSERT INTO character_npc_favor (snapshot_id, npc_key, favor_level)
             VALUES (?1, ?2, ?3)"
        ).map_err(|e| format!("Failed to prepare NPC insert: {e}"))?;

        for (npc_key, npc_data) in &report.npcs {
            npc_stmt.execute(rusqlite::params![
                snapshot_id,
                npc_key,
                npc_data.favor_level,
            ]).map_err(|e| format!("Failed to insert NPC {npc_key}: {e}"))?;
        }

        // 9. Batch insert recipe completions
        let mut recipe_stmt = conn.prepare(
            "INSERT INTO character_recipe_completions (snapshot_id, recipe_key, completions)
             VALUES (?1, ?2, ?3)"
        ).map_err(|e| format!("Failed to prepare recipe insert: {e}"))?;

        for (recipe_key, completions) in &report.recipe_completions {
            recipe_stmt.execute(rusqlite::params![
                snapshot_id,
                recipe_key,
                completions,
            ]).map_err(|e| format!("Failed to insert recipe {recipe_key}: {e}"))?;
        }

        // 10. Batch insert stats
        let mut stat_stmt = conn.prepare(
            "INSERT INTO character_stats (snapshot_id, stat_key, value)
             VALUES (?1, ?2, ?3)"
        ).map_err(|e| format!("Failed to prepare stat insert: {e}"))?;

        for (stat_key, value) in &report.current_stats {
            stat_stmt.execute(rusqlite::params![
                snapshot_id,
                stat_key,
                value,
            ]).map_err(|e| format!("Failed to insert stat {stat_key}: {e}"))?;
        }

        // 11. Batch insert currencies
        let mut currency_stmt = conn.prepare(
            "INSERT INTO character_currencies (snapshot_id, currency_key, amount)
             VALUES (?1, ?2, ?3)"
        ).map_err(|e| format!("Failed to prepare currency insert: {e}"))?;

        for (currency_key, amount) in &report.currencies {
            currency_stmt.execute(rusqlite::params![
                snapshot_id,
                currency_key,
                amount,
            ]).map_err(|e| format!("Failed to insert currency {currency_key}: {e}"))?;
        }

        Ok(ImportResult {
            character_name: report.character.clone(),
            server_name: report.server_name.clone(),
            snapshot_timestamp: report.timestamp.clone(),
            skills_imported: report.skills.len(),
            npcs_imported: report.npcs.len(),
            recipes_imported: report.recipe_completions.len(),
            stats_imported: report.current_stats.len(),
            currencies_imported: report.currencies.len(),
            was_duplicate: false,
        })
    })();

    // 12. Commit or rollback
    match &result {
        Ok(_) => {
            conn.execute("COMMIT", [])
                .map_err(|e| format!("Failed to commit transaction: {e}"))?;
        }
        Err(_) => {
            conn.execute("ROLLBACK", []).ok();
        }
    }

    result
}

#[tauri::command]
pub fn get_characters(
    db: State<'_, DbPool>,
) -> Result<Vec<CharacterInfo>, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn.prepare(
        "SELECT character_name, server_name,
                MAX(snapshot_timestamp) as latest_snapshot,
                COUNT(*) as snapshot_count
         FROM character_snapshots
         GROUP BY character_name, server_name
         ORDER BY latest_snapshot DESC"
    ).map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt.query_map([], |row| {
        Ok(CharacterInfo {
            character_name: row.get(0)?,
            server_name: row.get(1)?,
            latest_snapshot: row.get(2)?,
            snapshot_count: row.get::<_, i64>(3)? as usize,
        })
    }).map_err(|e| format!("Query failed: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read results: {e}"))
}

#[tauri::command]
pub fn get_character_snapshots(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: Option<String>,
) -> Result<Vec<CharacterSnapshotSummary>, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(ref server) = server_name {
        (
            "SELECT cs.id, cs.character_name, cs.server_name, cs.snapshot_timestamp,
                    cs.race, datetime(cs.import_date) as import_date,
                    (SELECT COUNT(*) FROM character_skill_levels WHERE snapshot_id = cs.id) as skill_count
             FROM character_snapshots cs
             WHERE cs.character_name = ?1 AND cs.server_name = ?2
             ORDER BY cs.snapshot_timestamp DESC",
            vec![
                Box::new(character_name.clone()) as Box<dyn rusqlite::types::ToSql>,
                Box::new(server.clone()),
            ],
        )
    } else {
        (
            "SELECT cs.id, cs.character_name, cs.server_name, cs.snapshot_timestamp,
                    cs.race, datetime(cs.import_date) as import_date,
                    (SELECT COUNT(*) FROM character_skill_levels WHERE snapshot_id = cs.id) as skill_count
             FROM character_snapshots cs
             WHERE cs.character_name = ?1
             ORDER BY cs.snapshot_timestamp DESC",
            vec![Box::new(character_name.clone()) as Box<dyn rusqlite::types::ToSql>],
        )
    };

    let mut stmt = conn.prepare(sql)
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(CharacterSnapshotSummary {
            id: row.get(0)?,
            character_name: row.get(1)?,
            server_name: row.get(2)?,
            snapshot_timestamp: row.get(3)?,
            race: row.get(4)?,
            import_date: row.get(5)?,
            skill_count: row.get::<_, i64>(6)? as usize,
        })
    }).map_err(|e| format!("Query failed: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read results: {e}"))
}

#[tauri::command]
pub fn get_snapshot_skills(
    db: State<'_, DbPool>,
    snapshot_id: i64,
) -> Result<Vec<SnapshotSkillLevel>, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn.prepare(
        "SELECT skill_name, level, bonus_levels, xp_toward_next, xp_needed_for_next
         FROM character_skill_levels
         WHERE snapshot_id = ?1
         ORDER BY skill_name"
    ).map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt.query_map([snapshot_id], |row| {
        Ok(SnapshotSkillLevel {
            skill_name: row.get(0)?,
            level: row.get(1)?,
            bonus_levels: row.get(2)?,
            xp_toward_next: row.get(3)?,
            xp_needed_for_next: row.get(4)?,
        })
    }).map_err(|e| format!("Query failed: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read results: {e}"))
}

#[tauri::command]
pub fn get_snapshot_npc_favor(
    db: State<'_, DbPool>,
    snapshot_id: i64,
) -> Result<Vec<SnapshotNpcFavor>, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn.prepare(
        "SELECT npc_key, favor_level
         FROM character_npc_favor
         WHERE snapshot_id = ?1
         ORDER BY npc_key"
    ).map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt.query_map([snapshot_id], |row| {
        Ok(SnapshotNpcFavor {
            npc_key: row.get(0)?,
            favor_level: row.get(1)?,
        })
    }).map_err(|e| format!("Query failed: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read results: {e}"))
}

#[tauri::command]
pub fn get_snapshot_recipes(
    db: State<'_, DbPool>,
    snapshot_id: i64,
) -> Result<Vec<SnapshotRecipeCompletion>, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn.prepare(
        "SELECT recipe_key, completions
         FROM character_recipe_completions
         WHERE snapshot_id = ?1
         ORDER BY recipe_key"
    ).map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt.query_map([snapshot_id], |row| {
        Ok(SnapshotRecipeCompletion {
            recipe_key: row.get(0)?,
            completions: row.get(1)?,
        })
    }).map_err(|e| format!("Query failed: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read results: {e}"))
}

#[tauri::command]
pub fn get_snapshot_stats(
    db: State<'_, DbPool>,
    snapshot_id: i64,
) -> Result<Vec<SnapshotStat>, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn.prepare(
        "SELECT stat_key, value
         FROM character_stats
         WHERE snapshot_id = ?1
         ORDER BY stat_key"
    ).map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt.query_map([snapshot_id], |row| {
        Ok(SnapshotStat {
            stat_key: row.get(0)?,
            value: row.get(1)?,
        })
    }).map_err(|e| format!("Query failed: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read results: {e}"))
}

#[tauri::command]
pub fn get_snapshot_currencies(
    db: State<'_, DbPool>,
    snapshot_id: i64,
) -> Result<Vec<SnapshotCurrency>, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn.prepare(
        "SELECT currency_key, amount
         FROM character_currencies
         WHERE snapshot_id = ?1
         ORDER BY currency_key"
    ).map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt.query_map([snapshot_id], |row| {
        Ok(SnapshotCurrency {
            currency_key: row.get(0)?,
            amount: row.get(1)?,
        })
    }).map_err(|e| format!("Query failed: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read results: {e}"))
}

#[tauri::command]
pub fn compare_snapshots(
    db: State<'_, DbPool>,
    snapshot_id_old: i64,
    snapshot_id_new: i64,
) -> Result<Vec<SkillDiff>, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    // Get skills from both snapshots and compute diffs using UNION to emulate FULL OUTER JOIN
    let mut stmt = conn.prepare(
        "SELECT
            skill_name,
            MAX(CASE WHEN snapshot_id = ?1 THEN level ELSE 0 END) as old_level,
            MAX(CASE WHEN snapshot_id = ?2 THEN level ELSE 0 END) as new_level,
            MAX(CASE WHEN snapshot_id = ?2 THEN level ELSE 0 END) -
                MAX(CASE WHEN snapshot_id = ?1 THEN level ELSE 0 END) as level_change,
            MAX(CASE WHEN snapshot_id = ?1 THEN xp_toward_next ELSE 0 END) as old_xp,
            MAX(CASE WHEN snapshot_id = ?2 THEN xp_toward_next ELSE 0 END) as new_xp
         FROM character_skill_levels
         WHERE snapshot_id IN (?1, ?2)
         GROUP BY skill_name
         ORDER BY skill_name"
    ).map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt.query_map(rusqlite::params![snapshot_id_old, snapshot_id_new], |row| {
        Ok(SkillDiff {
            skill_name: row.get(0)?,
            old_level: row.get(1)?,
            new_level: row.get(2)?,
            level_change: row.get(3)?,
            old_xp: row.get(4)?,
            new_xp: row.get(5)?,
        })
    }).map_err(|e| format!("Query failed: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read results: {e}"))
}
