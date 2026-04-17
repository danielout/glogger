use std::fs::{self, File};
use std::io::BufRead;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use crate::cdn_commands::GameDataState;
use crate::db::DbPool;
use crate::game_state::GameStateManager;
use crate::parsers::parse_skill_update;
use crate::player_event_parser::PlayerEventParser;
use crate::settings::SettingsManager;
use crate::survey::aggregator::SurveySessionAggregator;

/// Parse an entire Player.log file at once (used by "Upload Player.log" in Advanced Settings).
///
/// Runs the player event parser, skill-update parser, game-state persistence,
/// and the survey aggregator against the file as if it were a replay. The
/// file's modification time (converted to UTC) is used as the base date for
/// all `HH:MM:SS` timestamps, so events land in the database with the date
/// they were captured rather than today's date.
///
/// Solo Player.log has no embedded date and no chat-log correlation; the
/// mtime is the best signal available. If the file has been copied between
/// machines or otherwise had its mtime clobbered, results will be off by
/// that amount.
#[tauri::command]
pub async fn parse_log(path: String, app: AppHandle) -> Result<(), String> {
    let path = PathBuf::from(&path);
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }

    let db = app.state::<DbPool>().inner().clone();
    let game_data = app.state::<GameDataState>().inner().clone();
    let settings = app.state::<Arc<SettingsManager>>().inner().clone();

    let base_date = file_mtime_utc_date(&path).unwrap_or_else(|| chrono::Utc::now().date_naive());
    eprintln!("[parse_log] Base date (from file mtime): {}", base_date);

    tokio::task::spawn_blocking(move || {
        let mut player_parser = PlayerEventParser::new();
        let mut game_state = GameStateManager::new(game_data.clone());
        game_state.set_base_date(base_date);
        let mut survey_aggregator = SurveySessionAggregator::new(game_data);
        survey_aggregator.set_base_date(base_date);

        // Seed character/server from persisted settings — the solo log path has
        // no login line, so we assume the user is reparsing their own log.
        let current_settings = settings.get();
        if let (Some(c), Some(s)) = (
            current_settings.active_character_name.as_deref(),
            current_settings.active_server_name.as_deref(),
        ) {
            game_state.set_active_character_name(c);
            game_state.set_active_server_name(s);
        }
        let active_char = current_settings.active_character_name.clone();
        let active_server = current_settings.active_server_name.clone();

        let file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[parse_log] open error: {e}");
                return;
            }
        };
        let reader = std::io::BufReader::new(file);

        for line in reader.lines() {
            let l = match line {
                Ok(l) => l.trim_end().to_string(),
                Err(e) => {
                    eprintln!("[parse_log] read error: {e}");
                    continue;
                }
            };

            if let Some(update) = parse_skill_update(&l) {
                app.emit("skill-update", update).ok();
            }

            let mut events = player_parser.process_line(&l);
            dispatch_events(
                &app,
                &db,
                &mut game_state,
                &mut survey_aggregator,
                active_char.as_deref(),
                active_server.as_deref(),
                &mut events,
            );
        }

        // Flush any pending events buffered in the parser.
        let mut flush_events = player_parser.flush_all_pending();
        dispatch_events(
            &app,
            &db,
            &mut game_state,
            &mut survey_aggregator,
            active_char.as_deref(),
            active_server.as_deref(),
            &mut flush_events,
        );
    });

    Ok(())
}

/// Read the file's last-modified time and return it as a UTC `NaiveDate`.
/// `SystemTime` is platform-local in wall-clock semantics but `DateTime::<Utc>::from`
/// handles the conversion directly (no manual offset needed).
fn file_mtime_utc_date(path: &PathBuf) -> Option<chrono::NaiveDate> {
    let meta = fs::metadata(path).ok()?;
    let mtime = meta.modified().ok()?;
    let dt: chrono::DateTime<chrono::Utc> = mtime.into();
    Some(dt.date_naive())
}

fn dispatch_events(
    app: &AppHandle,
    db: &DbPool,
    game_state: &mut GameStateManager,
    survey_aggregator: &mut SurveySessionAggregator,
    active_char: Option<&str>,
    active_server: Option<&str>,
    events: &mut [crate::player_event_parser::PlayerEvent],
) {
    for pe in events.iter_mut() {
        if let (Some(character), Some(server), Ok(conn)) = (active_char, active_server, db.get()) {
            let _ = survey_aggregator.process_event(pe, &conn, character, server, None);
        }

        let gs_result = game_state.process_event(pe, db);
        if !gs_result.domains_updated.is_empty() {
            app.emit("game-state-updated", &gs_result.domains_updated).ok();
        }
        app.emit("player-event", &*pe).ok();
    }
}
