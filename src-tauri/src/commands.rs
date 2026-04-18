use std::fs::{self, File};
use std::io::BufRead;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

use crate::cdn_commands::GameDataState;
use crate::db::DbPool;
use crate::game_state::GameStateManager;
use crate::parsers::parse_skill_update;
use crate::player_event_parser::{PlayerEvent, PlayerEventParser};
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

        // Batch player events and domain updates to reduce PostMessage calls,
        // matching the coordinator's batching strategy.
        const BATCH_MAX_SIZE: usize = 50;
        const BATCH_MAX_AGE: Duration = Duration::from_millis(100);

        let mut player_event_batch: Vec<PlayerEvent> = Vec::new();
        let mut domains_batch: Vec<&'static str> = Vec::new();
        let mut batch_start = Instant::now();
        let mut emits_since_yield: u32 = 0;
        let mut last_yield = Instant::now();

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
                emits_since_yield += 1;
            }

            let mut events = player_parser.process_line(&l);

            // Run survey aggregator before batching
            for pe in events.iter_mut() {
                if let (Some(character), Some(server), Ok(conn)) =
                    (active_char.as_deref(), active_server.as_deref(), db.get())
                {
                    let _ = survey_aggregator.process_event(pe, &conn, character, server, None);
                }
            }
            player_event_batch.extend(events);

            // Flush when batch is full or old enough
            if player_event_batch.len() >= BATCH_MAX_SIZE
                || (!player_event_batch.is_empty() && batch_start.elapsed() >= BATCH_MAX_AGE)
            {
                let batch_result = game_state.process_events_batch(&player_event_batch, &db);
                domains_batch.extend(batch_result.domains_updated);

                app.emit("player-events-batch", &player_event_batch).ok();
                player_event_batch.clear();
                emits_since_yield += 1;

                if !domains_batch.is_empty() {
                    domains_batch.sort_unstable();
                    domains_batch.dedup();
                    app.emit("game-state-updated", &domains_batch).ok();
                    domains_batch.clear();
                    emits_since_yield += 1;
                }
                batch_start = Instant::now();

                // Yield so the webview JS event loop can drain
                if emits_since_yield >= 4 {
                    std::thread::sleep(Duration::from_millis(15));
                    last_yield = Instant::now();
                    emits_since_yield = 0;
                }
            }

            // Extra yield for non-batched events (skill-update)
            if emits_since_yield >= 20 && last_yield.elapsed() < Duration::from_millis(50) {
                std::thread::sleep(Duration::from_millis(15));
                last_yield = Instant::now();
                emits_since_yield = 0;
            } else if last_yield.elapsed() >= Duration::from_millis(50) {
                last_yield = Instant::now();
                emits_since_yield = 0;
            }
        }

        // Flush remaining batch
        if !player_event_batch.is_empty() {
            let batch_result = game_state.process_events_batch(&player_event_batch, &db);
            domains_batch.extend(batch_result.domains_updated);
            app.emit("player-events-batch", &player_event_batch).ok();
            player_event_batch.clear();
        }
        if !domains_batch.is_empty() {
            domains_batch.sort_unstable();
            domains_batch.dedup();
            app.emit("game-state-updated", &domains_batch).ok();
        }

        // Flush pending events from the parser itself
        let mut flush_events = player_parser.flush_all_pending();
        if !flush_events.is_empty() {
            for pe in flush_events.iter_mut() {
                if let (Some(character), Some(server), Ok(conn)) =
                    (active_char.as_deref(), active_server.as_deref(), db.get())
                {
                    let _ = survey_aggregator.process_event(pe, &conn, character, server, None);
                }
            }
            let batch_result = game_state.process_events_batch(&flush_events, &db);
            app.emit("player-events-batch", &flush_events).ok();
            if !batch_result.domains_updated.is_empty() {
                let mut domains = batch_result.domains_updated;
                domains.sort_unstable();
                domains.dedup();
                app.emit("game-state-updated", &domains).ok();
            }
        }
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

