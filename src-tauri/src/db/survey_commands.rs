use serde::Serialize;
use tauri::State;

use super::DbPool;

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct SurveyTypeInfo {
    pub item_id: u32,
    pub internal_name: String,
    pub name: String,
    pub zone: Option<String>,
    pub icon_id: Option<u32>,
    pub survey_category: String,
    pub is_motherlode: bool,
    pub skill_req_name: Option<String>,
    pub skill_req_level: Option<i64>,
    pub survey_skill_req: Option<i64>,
    pub recipe_id: Option<u32>,
    pub survey_xp: Option<f32>,
    pub survey_xp_first_time: Option<f32>,
    pub crafting_cost: Option<f64>,
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Get all survey types from the pre-parsed survey_types table
#[tauri::command]
pub fn get_all_survey_types(db: State<'_, DbPool>) -> Result<Vec<SurveyTypeInfo>, String> {
    let conn = db.get().map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn.prepare(
        "SELECT item_id, internal_name, name, zone, icon_id,
                survey_category, is_motherlode, skill_req_name,
                skill_req_level, survey_skill_req, recipe_id,
                survey_xp, survey_xp_first_time, crafting_cost
         FROM survey_types
         ORDER BY survey_category ASC, zone ASC, skill_req_level ASC"
    ).map_err(|e| format!("Query prepare error: {e}"))?;

    let surveys = stmt.query_map([], |row| {
        Ok(SurveyTypeInfo {
            item_id: row.get(0)?,
            internal_name: row.get(1)?,
            name: row.get(2)?,
            zone: row.get(3)?,
            icon_id: row.get(4)?,
            survey_category: row.get(5)?,
            is_motherlode: row.get(6)?,
            skill_req_name: row.get(7)?,
            skill_req_level: row.get(8)?,
            survey_skill_req: row.get(9)?,
            recipe_id: row.get(10)?,
            survey_xp: row.get(11)?,
            survey_xp_first_time: row.get(12)?,
            crafting_cost: row.get(13)?,
        })
    }).map_err(|e| format!("Query error: {e}"))?;

    let result: Vec<SurveyTypeInfo> = surveys.filter_map(|r| r.ok()).collect();
    Ok(result)
}
