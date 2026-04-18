/// Dual-log replay — simulates live tailing by interleaving Player.log and Chat.log
/// events by timestamp, processing them through the same coordinator pipelines.
///
/// This enables cross-referencing between the two log streams (e.g., correcting
/// motherlode loot quantities from Chat.log [Status] messages) using archived logs.
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

use crate::cdn_commands::GameDataState;
use crate::chat_parser::{
    is_timestamped_line, parse_chat_line, parse_chat_login_line, ChatMessage,
};
use crate::chat_status_parser::parse_status_message;
use crate::db::DbPool;
use crate::game_state::GameStateManager;
use crate::parsers::{chat_local_to_utc, parse_skill_update, parse_timestamp};
use crate::player_event_parser::{PlayerEvent, PlayerEventParser};
use crate::survey::aggregator::SurveySessionAggregator;

/// A timestamped event from either log source, used for interleaving.
#[derive(Debug)]
enum TimedEvent {
    /// Events from Player.log (processed first within a second)
    PlayerLine {
        /// UTC second (for ordering)
        utc_second: i64,
        /// The raw log line
        line: String,
    },
    /// A chat message from Chat.log
    ChatMessage {
        /// UTC second (for ordering)
        utc_second: i64,
        msg: ChatMessage,
    },
    /// Login detected from Chat.log — carries timezone offset
    ChatLogin {
        /// UTC second (for ordering)
        utc_second: i64,
        server_name: String,
        character_name: String,
        timezone_offset_seconds: Option<i32>,
    },
}

impl TimedEvent {
    fn utc_second(&self) -> i64 {
        match self {
            TimedEvent::PlayerLine { utc_second, .. } => *utc_second,
            TimedEvent::ChatMessage { utc_second, .. } => *utc_second,
            TimedEvent::ChatLogin { utc_second, .. } => *utc_second,
        }
    }

    /// Sort key: (utc_second, source_order)
    /// source_order: 0 = ChatLogin (timezone must come first), 1 = PlayerLine, 2 = ChatMessage
    fn sort_key(&self) -> (i64, u8) {
        match self {
            TimedEvent::ChatLogin { utc_second, .. } => (*utc_second, 0),
            TimedEvent::PlayerLine { utc_second, .. } => (*utc_second, 1),
            TimedEvent::ChatMessage { utc_second, .. } => (*utc_second, 2),
        }
    }
}

/// Progress event emitted to the frontend during replay.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReplayProgress {
    pub phase: String,
    pub current: usize,
    pub total: usize,
    pub detail: String,
}

/// Replay result summary.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReplayResult {
    pub player_lines_processed: usize,
    pub chat_messages_processed: usize,
    pub player_events_emitted: usize,
    pub chat_status_events_emitted: usize,
}

/// Parse Player.log into timestamped lines.
///
/// Player.log timestamps are local time `[HH:MM:SS]` with no date.
/// Player.log timestamps are already UTC with no date. We derive the date from
/// the chat log filename or fall back to today's UTC date.
fn parse_player_log_lines(
    path: &PathBuf,
    base_date: chrono::NaiveDate,
) -> Result<Vec<TimedEvent>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open Player.log: {}", e))?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Read error: {}", e))?;
        let line = line.trim_end().to_string();
        if line.is_empty() {
            continue;
        }

        // Extract [HH:MM:SS] timestamp — already UTC
        if let Some(ts_str) = parse_timestamp(&line) {
            if let Ok(utc_time) = chrono::NaiveTime::parse_from_str(&ts_str, "%H:%M:%S") {
                let utc_dt = base_date.and_time(utc_time);
                let utc_second = utc_dt.and_utc().timestamp();

                events.push(TimedEvent::PlayerLine { utc_second, line });
            }
        }
        // Lines without timestamps (login announcements, etc.) get appended
        // with the same second as the previous event
        else if !events.is_empty() {
            let prev_second = events.last().unwrap().utc_second();
            events.push(TimedEvent::PlayerLine {
                utc_second: prev_second,
                line,
            });
        }
    }

    Ok(events)
}

/// Parse Chat.log into timestamped events.
/// Also extracts login lines for timezone/server detection.
fn parse_chat_log_events(path: &PathBuf) -> Result<Vec<TimedEvent>, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open Chat.log: {}", e))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Failed to read Chat.log: {}", e))?;

    let mut events = Vec::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        // Check for login line first (carries timezone offset)
        if let Some(info) = parse_chat_login_line(line) {
            // Login lines have a timestamp too — extract it
            let utc_second = if is_timestamped_line(line) {
                if let Some(msg) = parse_chat_line(line) {
                    msg.timestamp.and_utc().timestamp()
                } else {
                    0
                }
            } else {
                0
            };

            events.push(TimedEvent::ChatLogin {
                utc_second,
                server_name: info.server_name,
                character_name: info.character_name,
                timezone_offset_seconds: info.timezone_offset_seconds,
            });
            continue;
        }

        // Regular chat message
        if is_timestamped_line(line) {
            if let Some(msg) = parse_chat_line(line) {
                let utc_second = msg.timestamp.and_utc().timestamp();
                events.push(TimedEvent::ChatMessage { utc_second, msg });
            }
        }
    }

    Ok(events)
}

/// Extract a date from a chat log filename like "Chat-26-03-27.log"
fn date_from_chat_filename(path: &PathBuf) -> Option<chrono::NaiveDate> {
    let stem = path.file_stem()?.to_str()?;
    // "Chat-YY-MM-DD"
    let date_part = stem.strip_prefix("Chat-")?;
    chrono::NaiveDate::parse_from_str(date_part, "%y-%m-%d").ok()
}

/// Extract the date from the first chat message timestamp in the file.
/// Fallback when the filename doesn't follow the Chat-YY-MM-DD pattern.
fn date_from_chat_content(events: &[TimedEvent]) -> Option<chrono::NaiveDate> {
    for event in events {
        if let TimedEvent::ChatMessage { msg, .. } = event {
            return Some(msg.timestamp.date());
        }
        if let TimedEvent::ChatLogin { utc_second, .. } = event {
            if *utc_second > 0 {
                let dt = chrono::DateTime::from_timestamp(*utc_second, 0)?;
                return Some(dt.date_naive());
            }
        }
    }
    None
}

/// Core replay logic — processes both logs through the full coordinator pipeline.
fn run_replay(
    player_log_path: PathBuf,
    chat_log_path: PathBuf,
    app: &AppHandle,
    db: &DbPool,
    game_data: GameDataState,
) -> Result<ReplayResult, String> {
    // --- Phase 1: Pre-scan chat log for timezone offset ---
    app.emit(
        "replay-progress",
        ReplayProgress {
            phase: "scanning".into(),
            current: 0,
            total: 2,
            detail: "Scanning chat log for timezone info...".into(),
        },
    )
    .ok();

    let chat_events = parse_chat_log_events(&chat_log_path)?;

    // Find the first timezone offset from login lines
    let mut tz_offset: i32 = 0;
    for event in &chat_events {
        if let TimedEvent::ChatLogin {
            timezone_offset_seconds: Some(offset),
            ..
        } = event
        {
            tz_offset = *offset;
            break;
        }
    }

    // Derive base date from chat log filename, chat content, or today
    let base_date = date_from_chat_filename(&chat_log_path)
        .or_else(|| date_from_chat_content(&chat_events))
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    eprintln!(
        "[replay] Base date: {}, timezone offset: {}s",
        base_date, tz_offset
    );

    // --- Phase 2: Parse Player.log with correct timezone ---
    app.emit(
        "replay-progress",
        ReplayProgress {
            phase: "scanning".into(),
            current: 1,
            total: 2,
            detail: "Parsing Player.log...".into(),
        },
    )
    .ok();

    let player_events = parse_player_log_lines(&player_log_path, base_date)?;

    // --- Phase 3: Apply timezone offset to chat events and merge ---
    // Chat.log timestamps are local time; convert to UTC using the detected offset.
    let chat_events: Vec<TimedEvent> = chat_events
        .into_iter()
        .map(|event| match event {
            TimedEvent::ChatMessage { msg, .. } => {
                let mut msg = msg;
                msg.timestamp = chat_local_to_utc(msg.timestamp, tz_offset);
                let utc_second = msg.timestamp.and_utc().timestamp();
                TimedEvent::ChatMessage { utc_second, msg }
            }
            TimedEvent::ChatLogin {
                server_name,
                character_name,
                timezone_offset_seconds,
                ..
            } => {
                // Recalculate utc_second with offset applied
                TimedEvent::ChatLogin {
                    utc_second: 0, // Login lines sort first regardless
                    server_name,
                    character_name,
                    timezone_offset_seconds,
                }
            }
            other => other,
        })
        .collect();

    let total_events = player_events.len() + chat_events.len();
    let mut all_events: Vec<TimedEvent> = Vec::with_capacity(total_events);
    all_events.extend(player_events);
    all_events.extend(chat_events);

    // Stable sort: ChatLogin first (timezone), then PlayerLine, then ChatMessage
    all_events.sort_by_key(|e| e.sort_key());

    // Diagnostic: show first/last timestamps from each source
    if let Some(first_player) = all_events
        .iter()
        .find(|e| matches!(e, TimedEvent::PlayerLine { .. }))
    {
        if let Some(last_player) = all_events
            .iter()
            .rev()
            .find(|e| matches!(e, TimedEvent::PlayerLine { .. }))
        {
            eprintln!(
                "[replay] Player.log UTC range: {} .. {}",
                first_player.utc_second(),
                last_player.utc_second()
            );
        }
    }
    if let Some(first_chat) = all_events
        .iter()
        .find(|e| matches!(e, TimedEvent::ChatMessage { .. }))
    {
        if let Some(last_chat) = all_events
            .iter()
            .rev()
            .find(|e| matches!(e, TimedEvent::ChatMessage { .. }))
        {
            eprintln!(
                "[replay] Chat.log UTC range: {} .. {}",
                first_chat.utc_second(),
                last_chat.utc_second()
            );
        }
    }

    // --- Phase 4: Process through coordinator pipeline ---
    let mut player_parser = PlayerEventParser::new();
    let mut game_state = GameStateManager::new(game_data.clone());
    game_state.set_base_date(base_date);
    let mut survey_aggregator = SurveySessionAggregator::new(game_data);
    survey_aggregator.set_base_date(base_date);

    let mut result = ReplayResult {
        player_lines_processed: 0,
        chat_messages_processed: 0,
        player_events_emitted: 0,
        chat_status_events_emitted: 0,
    };

    let progress_interval = (total_events / 100).max(50); // emit ~100 progress events

    // Use the same batching strategy as the live coordinator to avoid
    // flooding the Windows message queue. Accumulate player events and
    // domain updates, then flush in consolidated emissions. This
    // dramatically reduces the number of PostMessage calls compared to
    // per-event emission.
    const BATCH_MAX_SIZE: usize = 50;
    const BATCH_MAX_AGE: Duration = Duration::from_millis(100);

    let mut player_event_batch: Vec<PlayerEvent> = Vec::new();
    let mut domains_batch: Vec<&'static str> = Vec::new();
    let mut batch_start = Instant::now();
    let mut emits_since_yield: u32 = 0;
    let mut last_yield = Instant::now();

    /// Flush accumulated player events and domain updates to the frontend.
    /// Yields briefly after each flush so the message queue can drain.
    macro_rules! flush_batches {
        ($app:expr, $gs:expr, $pe:expr, $dom:expr, $start:expr, $emits:expr, $last_yield:expr) => {
            if !$pe.is_empty() {
                let batch_result = $gs.process_events_batch(&$pe, &db);
                $dom.extend(batch_result.domains_updated);

                $app.emit("player-events-batch", &$pe).ok();
                $emits += 1;
                $pe.clear();
            }
            if !$dom.is_empty() {
                $dom.sort_unstable();
                $dom.dedup();
                $app.emit("game-state-updated", &$dom).ok();
                $emits += 1;
                $dom.clear();
            }
            $start = Instant::now();

            // Yield so the webview JS event loop can process the batch
            if $emits >= 4 {
                std::thread::sleep(Duration::from_millis(15));
                $last_yield = Instant::now();
                $emits = 0;
            }
        };
    }

    for (i, event) in all_events.iter().enumerate() {
        // Progress updates
        if i % progress_interval == 0 {
            app.emit(
                "replay-progress",
                ReplayProgress {
                    phase: "processing".into(),
                    current: i,
                    total: total_events,
                    detail: format!("Processing event {}/{}", i, total_events),
                },
            )
            .ok();
        }

        match event {
            TimedEvent::ChatLogin {
                server_name,
                character_name,
                ..
            } => {
                // Flush pending batches before identity change
                flush_batches!(app, game_state, player_event_batch, domains_batch, batch_start, emits_since_yield, last_yield);

                game_state.set_active_character_name(character_name);
                game_state.set_active_server_name(server_name);

                app.emit("character-login", character_name).ok();
                app.emit("server-detected", server_name).ok();
                emits_since_yield += 2;
            }

            TimedEvent::PlayerLine { line, .. } => {
                result.player_lines_processed += 1;

                // Skill updates (legacy)
                if let Some(update) = parse_skill_update(line) {
                    app.emit("skill-update", &update).ok();
                    emits_since_yield += 1;
                }

                // Player events
                let mut p_events = player_parser.process_line(line);

                // Survey aggregator runs first so any survey_use_id it
                // injects into provenance reaches item_transactions via
                // game_state. Matches the live coordinator ordering.
                let active_char = game_state.get_active_character().map(String::from);
                let active_server = game_state.get_active_server().map(String::from);
                for pe in p_events.iter_mut() {
                    if let (Some(character), Some(server), Ok(conn)) =
                        (&active_char, &active_server, db.get())
                    {
                        let _ = survey_aggregator.process_event(pe, &conn, character, server, None);
                    }
                }

                result.player_events_emitted += p_events.len();
                player_event_batch.extend(p_events);
            }

            TimedEvent::ChatMessage { msg, .. } => {
                result.chat_messages_processed += 1;

                // Status channel → ChatStatusParser
                if let Some(status_event) = parse_status_message(msg) {
                    app.emit("chat-status-event", &status_event).ok();
                    emits_since_yield += 1;
                    result.chat_status_events_emitted += 1;
                }
            }
        }

        // Flush when batch is full or old enough
        if player_event_batch.len() >= BATCH_MAX_SIZE
            || (!player_event_batch.is_empty() && batch_start.elapsed() >= BATCH_MAX_AGE)
        {
            flush_batches!(app, game_state, player_event_batch, domains_batch, batch_start, emits_since_yield, last_yield);
        }

        // Extra yield if we've been emitting a lot of non-batched events
        // (skill-update, chat-status-event, character-login, etc.)
        if emits_since_yield >= 20 && last_yield.elapsed() < Duration::from_millis(50) {
            std::thread::sleep(Duration::from_millis(15));
            last_yield = Instant::now();
            emits_since_yield = 0;
        } else if last_yield.elapsed() >= Duration::from_millis(50) {
            last_yield = Instant::now();
            emits_since_yield = 0;
        }
    }

    // Flush any remaining batched events
    flush_batches!(app, game_state, player_event_batch, domains_batch, batch_start, emits_since_yield, last_yield);
    // Suppress unused-assignment warnings from the final macro expansion
    let _ = (batch_start, emits_since_yield, last_yield);

    // Flush pending player events from the parser itself
    let flush_events = player_parser.flush_all_pending();
    if !flush_events.is_empty() {
        let batch_result = game_state.process_events_batch(&flush_events, db);
        app.emit("player-events-batch", &flush_events).ok();
        result.player_events_emitted += flush_events.len();

        if !batch_result.domains_updated.is_empty() {
            let mut domains = batch_result.domains_updated;
            domains.sort_unstable();
            domains.dedup();
            app.emit("game-state-updated", &domains).ok();
        }
        std::thread::sleep(Duration::from_millis(15));
    }

    // Final progress
    app.emit(
        "replay-progress",
        ReplayProgress {
            phase: "complete".into(),
            current: total_events,
            total: total_events,
            detail: format!(
                "Done: {} player events, {} chat messages",
                result.player_events_emitted, result.chat_messages_processed,
            ),
        },
    )
    .ok();

    Ok(result)
}


// ============================================================
// Tauri Command
// ============================================================

/// Replay both a Player.log and Chat.log file through the full coordinator pipeline,
/// interleaved by timestamp. This simulates live tailing with cross-referencing.
#[tauri::command]
pub async fn replay_dual_logs(
    player_log_path: String,
    chat_log_path: String,
    app: AppHandle,
) -> Result<ReplayResult, String> {
    let player_path = PathBuf::from(&player_log_path);
    let chat_path = PathBuf::from(&chat_log_path);

    if !player_path.exists() {
        return Err(format!("Player.log not found: {}", player_log_path));
    }
    if !chat_path.exists() {
        return Err(format!("Chat.log not found: {}", chat_log_path));
    }

    let db = app.state::<DbPool>().inner().clone();
    let game_data = app.state::<GameDataState>().inner().clone();

    // Run on a blocking thread since file I/O is synchronous
    let result = tokio::task::spawn_blocking(move || {
        run_replay(player_path, chat_path, &app, &db, game_data)
    })
    .await
    .map_err(|e| format!("Replay task failed: {}", e))??;

    Ok(result)
}
