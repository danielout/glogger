use super::DbPool;
use serde::{Deserialize, Serialize};
use tauri::State;

// ── Export/Import Format ─────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct SurveyExportFile {
    format: String,
    version: u32,
    metadata: SurveyExportMetadata,
    sessions: Vec<ExportSession>,
}

#[derive(Serialize, Deserialize)]
struct SurveyExportMetadata {
    exported_at: String,
    exporter_name: String,
    server_name: Option<String>,
    session_count: usize,
    event_count: usize,
}

#[derive(Serialize, Deserialize)]
struct ExportSession {
    start_time: String,
    end_time: Option<String>,
    maps_started: i64,
    surveys_completed: i64,
    total_revenue: f64,
    total_cost: f64,
    total_profit: f64,
    profit_per_hour: f64,
    elapsed_seconds: i64,
    speed_bonus_count: i64,
    survey_types_used: Option<String>,
    maps_used_summary: Option<String>,
    events: Vec<ExportEvent>,
}

#[derive(Serialize, Deserialize)]
struct ExportEvent {
    timestamp: String,
    event_type: String,
    map_type: Option<String>,
    survey_type: Option<String>,
    speed_bonus_earned: bool,
    loot_items: Vec<ExportLootItem>,
}

#[derive(Serialize, Deserialize)]
struct ExportLootItem {
    item_name: String,
    quantity: i64,
    is_speed_bonus: bool,
    is_primary: bool,
}

// ── Export Command ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn export_survey_data(
    db: State<'_, DbPool>,
    exporter_name: Option<String>,
    server_name: Option<String>,
) -> Result<String, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let exporter = exporter_name.unwrap_or_else(|| "Anonymous".to_string());

    // Fetch all local sessions (import_id IS NULL)
    let mut session_stmt = conn
        .prepare(
            "SELECT id, datetime(start_time) as start_time, datetime(end_time) as end_time,
                    maps_started, surveys_completed,
                    total_revenue, total_cost, total_profit, profit_per_hour,
                    elapsed_seconds, speed_bonus_count, survey_types_used, maps_used_summary
             FROM survey_session_stats
             WHERE import_id IS NULL
             ORDER BY start_time ASC",
        )
        .map_err(|e| format!("Failed to prepare session query: {e}"))?;

    let mut sessions: Vec<ExportSession> = Vec::new();
    let mut total_events = 0usize;

    let session_rows: Vec<(i64, String, Option<String>, i64, i64, f64, f64, f64, f64, i64, i64, Option<String>, Option<String>)> = {
        let rows = session_stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, f64>(5)?,
                    row.get::<_, f64>(6)?,
                    row.get::<_, f64>(7)?,
                    row.get::<_, f64>(8)?,
                    row.get::<_, i64>(9)?,
                    row.get::<_, i64>(10)?,
                    row.get::<_, Option<String>>(11)?,
                    row.get::<_, Option<String>>(12)?,
                ))
            })
            .map_err(|e| format!("Session query failed: {e}"))?;
        let mut collected = Vec::new();
        for r in rows {
            collected.push(r.map_err(|e| format!("Row error: {e}"))?);
        }
        collected
    };

    for (session_id, start_time, end_time, maps_started, surveys_completed,
         total_revenue, total_cost, total_profit, profit_per_hour,
         elapsed_seconds, speed_bonus_count, survey_types_used, maps_used_summary) in session_rows
    {
        // Fetch events for this session
        let mut event_stmt = conn
            .prepare(
                "SELECT id, datetime(timestamp) as timestamp, event_type, map_type, survey_type, speed_bonus_earned
                 FROM survey_events
                 WHERE session_id = ?1
                 ORDER BY timestamp ASC",
            )
            .map_err(|e| format!("Failed to prepare event query: {e}"))?;

        let event_rows: Vec<(i64, String, String, Option<String>, Option<String>, bool)> = {
            let rows = event_stmt
                .query_map([session_id], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, bool>(5)?,
                    ))
                })
                .map_err(|e| format!("Event query failed: {e}"))?;
            let mut collected = Vec::new();
            for r in rows {
                collected.push(r.map_err(|e| format!("Row error: {e}"))?);
            }
            collected
        };

        let mut events: Vec<ExportEvent> = Vec::new();

        for (event_id, timestamp, event_type, map_type, survey_type, speed_bonus_earned) in event_rows {
            // Fetch loot items for this event
            let mut loot_stmt = conn
                .prepare(
                    "SELECT item_name, quantity, is_speed_bonus, is_primary
                     FROM survey_loot_items
                     WHERE event_id = ?1
                     ORDER BY id ASC",
                )
                .map_err(|e| format!("Failed to prepare loot query: {e}"))?;

            let loot_items: Vec<ExportLootItem> = {
                let rows = loot_stmt
                    .query_map([event_id], |row| {
                        Ok(ExportLootItem {
                            item_name: row.get(0)?,
                            quantity: row.get(1)?,
                            is_speed_bonus: row.get(2)?,
                            is_primary: row.get(3)?,
                        })
                    })
                    .map_err(|e| format!("Loot query failed: {e}"))?;
                let mut collected = Vec::new();
                for r in rows {
                    collected.push(r.map_err(|e| format!("Row error: {e}"))?);
                }
                collected
            };

            events.push(ExportEvent {
                timestamp,
                event_type,
                map_type,
                survey_type,
                speed_bonus_earned,
                loot_items,
            });
        }

        total_events += events.len();

        sessions.push(ExportSession {
            start_time,
            end_time,
            maps_started,
            surveys_completed,
            total_revenue,
            total_cost,
            total_profit,
            profit_per_hour,
            elapsed_seconds,
            speed_bonus_count,
            survey_types_used,
            maps_used_summary,
            events,
        });
    }

    let export = SurveyExportFile {
        format: "glogger-survey-export".to_string(),
        version: 1,
        metadata: SurveyExportMetadata {
            exported_at: chrono::Utc::now().to_rfc3339(),
            exporter_name: exporter,
            server_name,
            session_count: sessions.len(),
            event_count: total_events,
        },
        sessions,
    };

    serde_json::to_string_pretty(&export).map_err(|e| format!("Serialization error: {e}"))
}

// ── Import Command ───────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct SurveyImportResult {
    pub import_id: i64,
    pub label: String,
    pub sessions_imported: usize,
    pub events_imported: usize,
    pub loot_items_imported: usize,
}

#[tauri::command]
pub fn import_survey_data_from_file(
    db: State<'_, DbPool>,
    file_path: String,
    label: Option<String>,
) -> Result<SurveyImportResult, String> {
    let json_data =
        std::fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file: {e}"))?;

    let export: SurveyExportFile =
        serde_json::from_str(&json_data).map_err(|e| format!("Invalid survey export file: {e}"))?;

    if export.format != "glogger-survey-export" {
        return Err("Not a valid glogger survey export file".to_string());
    }
    if export.version != 1 {
        return Err(format!(
            "Unsupported export version: {} (this app supports version 1)",
            export.version
        ));
    }

    let import_label = label.unwrap_or_else(|| {
        if let Some(ref server) = export.metadata.server_name {
            format!("{} @ {}", export.metadata.exporter_name, server)
        } else {
            format!("{}'s data", export.metadata.exporter_name)
        }
    });

    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    conn.execute("BEGIN", []).ok();

    // Create import record
    let import_id = match conn.execute(
        "INSERT INTO survey_data_imports (label, source_player, session_count, event_count)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            import_label,
            export.metadata.exporter_name,
            export.sessions.len(),
            export.metadata.event_count,
        ],
    ) {
        Ok(_) => conn.last_insert_rowid(),
        Err(e) => {
            conn.execute("ROLLBACK", []).ok();
            return Err(format!("Failed to create import record: {e}"));
        }
    };

    let mut sessions_imported = 0usize;
    let mut events_imported = 0usize;
    let mut loot_items_imported = 0usize;

    for session in &export.sessions {
        // Insert session
        let session_id = match conn.execute(
            "INSERT INTO survey_session_stats (
                start_time, end_time, maps_started, surveys_completed,
                total_revenue, total_cost, total_profit, profit_per_hour,
                elapsed_seconds, speed_bonus_count, survey_types_used, maps_used_summary,
                import_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            rusqlite::params![
                session.start_time,
                session.end_time,
                session.maps_started,
                session.surveys_completed,
                session.total_revenue,
                session.total_cost,
                session.total_profit,
                session.profit_per_hour,
                session.elapsed_seconds,
                session.speed_bonus_count,
                session.survey_types_used,
                session.maps_used_summary,
                import_id,
            ],
        ) {
            Ok(_) => conn.last_insert_rowid(),
            Err(e) => {
                conn.execute("ROLLBACK", []).ok();
                return Err(format!("Failed to insert session: {e}"));
            }
        };
        sessions_imported += 1;

        for event in &session.events {
            let event_id = match conn.execute(
                "INSERT INTO survey_events (timestamp, session_id, event_type, map_type, survey_type, speed_bonus_earned)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    event.timestamp,
                    session_id,
                    event.event_type,
                    event.map_type,
                    event.survey_type,
                    event.speed_bonus_earned,
                ],
            ) {
                Ok(_) => conn.last_insert_rowid(),
                Err(e) => {
                    conn.execute("ROLLBACK", []).ok();
                    return Err(format!("Failed to insert event: {e}"));
                }
            };
            events_imported += 1;

            for loot in &event.loot_items {
                if let Err(e) = conn.execute(
                    "INSERT INTO survey_loot_items (event_id, item_name, quantity, is_speed_bonus, is_primary)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        event_id,
                        loot.item_name,
                        loot.quantity,
                        loot.is_speed_bonus,
                        loot.is_primary,
                    ],
                ) {
                    conn.execute("ROLLBACK", []).ok();
                    return Err(format!("Failed to insert loot item: {e}"));
                }
                loot_items_imported += 1;
            }
        }
    }

    // Update counts on import record
    conn.execute(
        "UPDATE survey_data_imports SET session_count = ?1, event_count = ?2 WHERE id = ?3",
        rusqlite::params![sessions_imported, events_imported, import_id],
    )
    .ok();

    conn.execute("COMMIT", []).ok();

    Ok(SurveyImportResult {
        import_id,
        label: import_label,
        sessions_imported,
        events_imported,
        loot_items_imported,
    })
}

// ── Import Management Commands ───────────────────────────────────────────────

#[derive(Serialize)]
pub struct SurveyImportInfo {
    pub id: i64,
    pub label: String,
    pub source_player: Option<String>,
    pub session_count: i64,
    pub event_count: i64,
    pub imported_at: String,
}

#[tauri::command]
pub fn get_survey_imports(db: State<'_, DbPool>) -> Result<Vec<SurveyImportInfo>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, label, source_player, session_count, event_count,
                    datetime(imported_at) as imported_at
             FROM survey_data_imports
             ORDER BY imported_at DESC",
        )
        .map_err(|e| format!("Failed to prepare imports query: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(SurveyImportInfo {
                id: row.get(0)?,
                label: row.get(1)?,
                source_player: row.get(2)?,
                session_count: row.get(3)?,
                event_count: row.get(4)?,
                imported_at: row.get(5)?,
            })
        })
        .map_err(|e| format!("Imports query failed: {e}"))?;

    let mut results = Vec::new();
    for r in rows {
        results.push(r.map_err(|e| format!("Row error: {e}"))?);
    }

    Ok(results)
}

#[tauri::command]
pub fn delete_survey_import(db: State<'_, DbPool>, import_id: i64) -> Result<(), String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    conn.execute(
        "DELETE FROM survey_data_imports WHERE id = ?1",
        [import_id],
    )
    .map_err(|e| format!("Failed to delete import: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn rename_survey_import(
    db: State<'_, DbPool>,
    import_id: i64,
    label: String,
) -> Result<(), String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    conn.execute(
        "UPDATE survey_data_imports SET label = ?1 WHERE id = ?2",
        rusqlite::params![label, import_id],
    )
    .map_err(|e| format!("Failed to rename import: {e}"))?;

    Ok(())
}
