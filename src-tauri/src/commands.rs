use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

use crate::db::DbPool;
use crate::parsers::parse_skill_update;
use crate::player_event_parser::PlayerEventParser;
use crate::survey_parser::{KnownSurveyType, SurveyParser};
use crate::survey_persistence::SurveySessionTracker;

/// Parse an entire Player.log file at once (used by "Upload Player.log" in Advanced Settings).
/// Runs all parsers: skill updates, survey events, and player events.
#[tauri::command]
pub async fn parse_log(path: String, app: AppHandle) -> Result<(), String> {
    let path = PathBuf::from(&path);
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }

    // Load known survey types from DB before spawning the parse task
    let known_surveys = if let Some(db) = app.try_state::<DbPool>() {
        let conn = db.get().map_err(|e| format!("Database error: {}", e))?;
        load_known_surveys_for_parse(&conn)
    } else {
        HashMap::new()
    };

    tokio::spawn(async move {
        let file = File::open(&path).expect("could not open log file");
        let reader = std::io::BufReader::new(file);
        let mut survey_parser = SurveyParser::new(known_surveys);
        let mut survey_tracker = SurveySessionTracker::new();
        let mut player_parser = PlayerEventParser::new();

        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let l = l.trim_end().to_string();
                    emit_events(
                        &app,
                        &l,
                        &mut survey_parser,
                        &mut survey_tracker,
                        &mut player_parser,
                    );
                }
                Err(e) => eprintln!("Read error: {}", e),
            }
        }

        // Flush any pending player events at end of file
        let flush_events = player_parser.flush_all_pending();
        for event in flush_events {
            app.emit("player-event", &event).ok();
        }
    });

    Ok(())
}

/// Central dispatch: parse one line through all parsers and emit events
fn emit_events(
    app: &AppHandle,
    line: &str,
    survey_parser: &mut SurveyParser,
    survey_tracker: &mut SurveySessionTracker,
    player_parser: &mut PlayerEventParser,
) {
    // Skill updates (ProcessUpdateSkill)
    if let Some(update) = parse_skill_update(line) {
        app.emit("skill-update", update).ok();
    }

    // Player events first (items, skills, NPC, vendor, storage, screen text, books, delay loops)
    let player_events = player_parser.process_line(line);

    // Survey events consume PlayerEvents + raw line (for ProcessMapFx)
    let survey_events = survey_parser.process_events(&player_events, line);
    for event in &survey_events {
        if let Some(db) = app.try_state::<DbPool>() {
            let result = survey_tracker.process_event(event, db.inner());
            if result.session_ended {
                if let Some(sid) = result.session_id {
                    app.emit("survey-session-ended", sid).ok();
                }
            }
        }
        app.emit("survey-event", event).ok();
    }

    for event in player_events {
        app.emit("player-event", &event).ok();
    }
}

/// Load known survey types from DB for the parse_log command
fn load_known_surveys_for_parse(conn: &rusqlite::Connection) -> HashMap<String, KnownSurveyType> {
    let mut map = HashMap::new();
    let mut stmt = match conn.prepare("SELECT internal_name, name, is_motherlode FROM survey_types")
    {
        Ok(s) => s,
        Err(_) => return map,
    };

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, bool>(2)?,
        ))
    });

    if let Ok(rows) = rows {
        for row in rows.flatten() {
            let (internal_name, display_name, is_motherlode) = row;
            map.insert(
                internal_name,
                KnownSurveyType {
                    display_name,
                    is_motherlode,
                },
            );
        }
    }

    map
}
