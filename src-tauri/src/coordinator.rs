use crate::cdn_commands::GameDataState;
use crate::chat_status_parser::parse_status_message;
use crate::db::chat_commands::insert_chat_messages;
use crate::db::queries::log_positions;
use crate::db::DbPool;
use crate::game_state::GameStateManager;
use crate::log_watchers::{ChatLogWatcher, LogEvent, LogFileWatcher, PlayerLogWatcher};
use crate::settings::SettingsManager;
use crate::survey_parser::KnownSurveyType;
use crate::survey_persistence::SurveySessionTracker;
use crate::watch_rules::evaluate_rules;
use chrono::{Datelike, Local};
use serde::Serialize;
/// DataIngestCoordinator - Central coordinator for all file watching and database operations
///
/// This coordinator manages:
/// - PlayerLogWatcher (master log - tracks active character and chat log path)
/// - ChatLogWatcher (daily chat logs)
/// - Operation locking to prevent conflicts
/// - Database write coordination
/// - Progress event emission to frontend
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

/// Timestamped log line for startup diagnostics.
macro_rules! startup_log {
    ($($arg:tt)*) => {
        eprintln!("[{}] {}", Local::now().format("%H:%M:%S%.3f"), format!($($arg)*));
    };
}

/// Blocking operation type - prevents overlapping heavy operations
/// Note: Player tailing and chat tailing run concurrently and are tracked
/// separately via their watcher Option fields, NOT through this lock.
#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    /// No blocking operation in progress
    Idle,

    /// Full scan of chat logs in progress
    FullChatScan {
        total_files: usize,
        processed_files: usize,
    },

    /// Character export import in progress
    CharacterImport { progress: usize, total: usize },

    /// User-triggered action (takes priority)
    UserAction,
}

/// Progress information for long-running operations
#[derive(Debug, Clone, Serialize)]
pub struct ScanProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
    pub messages_processed: usize,
}

/// Status of the coordinator
#[derive(Debug, Clone, Serialize)]
pub struct CoordinatorStatus {
    pub player_log_active: bool,
    pub chat_log_active: bool,
    pub active_character: Option<String>,
    pub current_chat_log: Option<String>,
    pub operation: String, // Serialized OperationType
}

/// Central coordinator for all data ingestion
pub struct DataIngestCoordinator {
    player_watcher: Option<PlayerLogWatcher>,
    chat_watcher: Option<ChatLogWatcher>,
    operation_lock: Arc<RwLock<OperationType>>,
    db_pool: DbPool,
    settings: Arc<SettingsManager>,
    app_handle: AppHandle,
    survey_tracker: SurveySessionTracker,
    game_state: GameStateManager,
}

impl DataIngestCoordinator {
    /// Create a new coordinator
    pub fn new(
        db_pool: DbPool,
        settings: Arc<SettingsManager>,
        app_handle: AppHandle,
        game_data: GameDataState,
    ) -> Result<Self, String> {
        // Seed game state manager with persisted character+server so that
        // Player.log events during the initial replay have a valid server key.
        let current_settings = settings.get();
        let mut game_state = GameStateManager::new(game_data);
        if let (Some(char_name), Some(server_name)) = (
            &current_settings.active_character_name,
            &current_settings.active_server_name,
        ) {
            game_state.set_active_character_name(char_name);
            game_state.set_active_server_name(server_name);
        }

        // Seed timezone offset from persisted settings (manual override takes precedence)
        let mut survey_tracker = SurveySessionTracker::new();
        if let Some(offset) = current_settings
            .manual_timezone_override
            .or(current_settings.timezone_offset_seconds)
        {
            game_state.set_timezone_offset(offset);
            survey_tracker.set_timezone_offset(offset);
        }

        Ok(Self {
            player_watcher: None,
            chat_watcher: None,
            operation_lock: Arc::new(RwLock::new(OperationType::Idle)),
            db_pool,
            settings,
            app_handle,
            survey_tracker,
            game_state,
        })
    }

    /// Get current coordinator status
    pub fn get_status(&self) -> CoordinatorStatus {
        let operation = self.operation_lock.read().unwrap();
        let operation_str = match &*operation {
            OperationType::Idle => "idle",
            OperationType::FullChatScan { .. } => "full_scan",
            OperationType::CharacterImport { .. } => "character_import",
            OperationType::UserAction => "user_action",
        };

        CoordinatorStatus {
            player_log_active: self
                .player_watcher
                .as_ref()
                .map_or(false, |w| w.is_active()),
            chat_log_active: self.chat_watcher.as_ref().map_or(false, |w| w.is_active()),
            active_character: self
                .player_watcher
                .as_ref()
                .and_then(|w| w.get_active_character().map(String::from)),
            current_chat_log: self.player_watcher.as_ref().and_then(|w| {
                w.get_chat_log_path()
                    .and_then(|p| p.to_str().map(String::from))
            }),
            operation: operation_str.to_string(),
        }
    }

    /// Start player log tailing
    pub fn start_player_log_tailing(&mut self) -> Result<(), String> {
        // Check for blocking operations (full scans, imports, etc.)
        let operation = self.operation_lock.read().unwrap();
        if *operation != OperationType::Idle {
            return Err(format!(
                "Cannot start player log tailing: {:?} in progress",
                *operation
            ));
        }
        drop(operation);

        // Already tailing? No-op.
        if self
            .player_watcher
            .as_ref()
            .map_or(false, |w| w.is_active())
        {
            return Ok(());
        }

        // Get player log path from settings
        let player_log_path = self
            .settings
            .get_player_log_path()
            .ok_or_else(|| "Game data path not configured".to_string())?;

        // Load saved position from database
        let conn = self
            .db_pool
            .get()
            .map_err(|e| format!("Database error: {}", e))?;
        let mut position =
            log_positions::get_position(&conn, player_log_path.to_str().unwrap_or("")).unwrap_or(0);

        // Detect file rotation: if saved position is past current file size,
        // the game restarted and created a fresh Player.log
        if position > 0 {
            if let Ok(meta) = std::fs::metadata(&player_log_path) {
                if meta.len() < position {
                    startup_log!("Player.log was rotated (size {} < saved position {}), starting from beginning",
                        meta.len(), position);
                    position = 0;
                }
            }
        }

        // Load known survey types for the survey parser
        let known_surveys = load_known_surveys(&conn);
        drop(conn);

        // Create watcher
        let mut watcher = if position > 0 {
            startup_log!(
                "Starting Player.log catch-up from byte position {}",
                position
            );
            PlayerLogWatcher::from_position(player_log_path, position, known_surveys)
        } else {
            startup_log!("Starting Player.log from beginning (no saved position)");
            PlayerLogWatcher::new(player_log_path, known_surveys)
        };

        // Start watching
        watcher.start()?;
        startup_log!("Player.log watcher started");

        // Store watcher
        self.player_watcher = Some(watcher);

        // Emit status change event
        self.emit_status_change()?;

        Ok(())
    }

    /// Stop player log tailing
    pub fn stop_player_log_tailing(&mut self) -> Result<(), String> {
        if let Some(mut watcher) = self.player_watcher.take() {
            watcher.stop();

            // Save position to database
            let position = watcher.get_position();
            if let Some(path) = self.settings.get_player_log_path() {
                let conn = self
                    .db_pool
                    .get()
                    .map_err(|e| format!("Database error: {}", e))?;
                log_positions::update_position(
                    &conn,
                    path.to_str().unwrap_or(""),
                    "player",
                    position,
                    watcher.get_active_character(),
                    None,
                )
                .map_err(|e| format!("Failed to save position: {}", e))?;
            }
        }

        // Emit status change event
        self.emit_status_change()?;

        Ok(())
    }

    /// Start chat log tailing
    pub fn start_chat_log_tailing(&mut self, chat_log_path: PathBuf) -> Result<(), String> {
        // Check for blocking operations (full scans block chat tailing)
        let operation = self.operation_lock.read().unwrap();
        if let OperationType::FullChatScan { .. } = &*operation {
            return Err("Cannot start tailing during full scan".to_string());
        }
        drop(operation);

        // Already tailing the same file? No-op.
        if let Some(existing) = &self.chat_watcher {
            if existing.is_active() && existing.get_file_path() == &chat_log_path {
                return Ok(());
            }
            // Different file — stop the old watcher first
            self.stop_chat_log_tailing()?;
        }

        // Load saved position from database
        let conn = self
            .db_pool
            .get()
            .map_err(|e| format!("Database error: {}", e))?;
        let position =
            log_positions::get_position(&conn, chat_log_path.to_str().unwrap_or("")).unwrap_or(0);
        drop(conn);

        // Create watcher — parses all channels; filtering happens at persistence layer
        let mut watcher = if position > 0 {
            startup_log!("Starting chat log catch-up from byte position {}", position);
            ChatLogWatcher::from_position(chat_log_path, position)
        } else {
            startup_log!("Starting chat log from beginning");
            ChatLogWatcher::new(chat_log_path)
        };

        // Set player name if known
        if let Some(pw) = &self.player_watcher {
            if let Some(name) = pw.get_active_character() {
                watcher.set_player_name(name.to_string());
            }
        }

        // Start watching
        watcher.start()?;

        // Store watcher
        self.chat_watcher = Some(watcher);

        // Emit status change event
        self.emit_status_change()?;

        Ok(())
    }

    /// Stop chat log tailing
    pub fn stop_chat_log_tailing(&mut self) -> Result<(), String> {
        if let Some(mut watcher) = self.chat_watcher.take() {
            watcher.stop();

            // Save position to database using the actual file path
            let position = watcher.get_position();
            let file_path_str = watcher.get_file_path().to_string_lossy().to_string();
            let file_name = watcher.get_file_name().to_string();

            let conn = self
                .db_pool
                .get()
                .map_err(|e| format!("Database error: {}", e))?;
            let metadata = serde_json::json!({ "file_name": file_name }).to_string();
            log_positions::update_position(
                &conn,
                &file_path_str,
                "chat",
                position,
                None,
                Some(&metadata),
            )
            .map_err(|e| format!("Failed to save position: {}", e))?;
        }

        // Emit status change event
        self.emit_status_change()?;

        Ok(())
    }

    /// Poll all active watchers and process events
    pub fn poll(&mut self) -> Result<(), String> {
        // Poll player log watcher
        if let Some(watcher) = &mut self.player_watcher {
            let events = watcher.poll()?;
            self.process_player_events(events)?;
        }

        // After the first poll cycle, switch game state to live mode so future
        // logins will properly clear transient state. During the initial catch-up
        // replay we skip clearing to preserve data built for each character.
        if !self.game_state.is_live() {
            startup_log!("Player.log catch-up complete — switching to live tailing mode");
            self.game_state.set_live_mode();
        }

        // Poll chat log watcher
        if let Some(watcher) = &mut self.chat_watcher {
            let events = watcher.poll()?;
            self.process_chat_events(events)?;
        }

        // Persist watcher positions every poll cycle so a crash doesn't
        // lose all progress and cause a full re-parse on next launch.
        self.save_watcher_positions();

        Ok(())
    }

    /// Persist current watcher byte offsets to the database.
    /// Called every poll cycle so a crash only loses ~1 polling interval of progress.
    fn save_watcher_positions(&self) {
        let conn = match self.db_pool.get() {
            Ok(c) => c,
            Err(_) => return,
        };

        if let Some(watcher) = &self.player_watcher {
            if let Some(path) = self.settings.get_player_log_path() {
                log_positions::update_position(
                    &conn,
                    path.to_str().unwrap_or(""),
                    "player",
                    watcher.get_position(),
                    watcher.get_active_character(),
                    None,
                )
                .ok();
            }
        }

        if let Some(watcher) = &self.chat_watcher {
            let file_path_str = watcher.get_file_path().to_string_lossy().to_string();
            let file_name = watcher.get_file_name().to_string();
            let metadata = serde_json::json!({ "file_name": file_name }).to_string();
            log_positions::update_position(
                &conn,
                &file_path_str,
                "chat",
                watcher.get_position(),
                None,
                Some(&metadata),
            )
            .ok();
        }
    }

    /// Process events from player log
    fn process_player_events(&mut self, events: Vec<LogEvent>) -> Result<(), String> {
        // Throttle: on catch-up after startup, this can receive thousands of events.
        // Yield periodically so the Windows message queue doesn't overflow.
        let mut last_yield = Instant::now();
        let mut emits_since_yield: u32 = 0;

        for event in events {
            match event {
                LogEvent::CharacterLogin { character_name, .. } => {
                    startup_log!("Character detected from Player.log: {}", character_name);
                    // Player.log knows the character name but NOT the server.
                    // Update the character name in settings; the chat log's
                    // ServerDetected + CharacterLogin pair is the authoritative
                    // source that calls set_active_character with both values.
                    let mut settings = self.settings.get();
                    settings.active_character_name = Some(character_name.clone());
                    self.settings.update(settings).ok();

                    // Update game state character name only (server stays as-is)
                    self.game_state.set_active_character_name(&character_name);

                    self.app_handle
                        .emit("character-login", &character_name)
                        .ok();
                    emits_since_yield += 1;
                }
                LogEvent::ChatLogPath { path, .. } => {
                    startup_log!("Chat log path detected: {}", path);
                    // Chat log path changed - start/switch chat watcher
                    // start_chat_log_tailing handles stopping the old watcher if needed
                    self.start_chat_log_tailing(PathBuf::from(&path))?;
                }
                LogEvent::AreaTransition { area, .. } => {
                    self.app_handle.emit("area-transition", &area).ok();
                    emits_since_yield += 1;
                }
                LogEvent::SkillUpdated(update) => {
                    self.app_handle.emit("skill-update", &update).ok();
                    emits_since_yield += 1;
                }
                LogEvent::SurveyParsed(survey_event) => {
                    // Persist to DB synchronously first, then emit to frontend
                    let result = self
                        .survey_tracker
                        .process_event(&survey_event, &self.db_pool);
                    // Wrap the event with session_id so the frontend can track it
                    let mut payload = serde_json::to_value(&survey_event).unwrap_or_default();
                    if let (serde_json::Value::Object(ref mut map), Some(sid)) =
                        (&mut payload, result.session_id)
                    {
                        map.insert(
                            "session_id".to_string(),
                            serde_json::Value::Number(sid.into()),
                        );
                    }
                    self.app_handle.emit("survey-event", &payload).ok();
                    emits_since_yield += 1;

                    // If the session auto-ended, notify frontend so it can patch in
                    // elapsed/XP data that only the frontend knows about
                    if result.session_ended {
                        if let Some(sid) = result.session_id {
                            self.app_handle.emit("survey-session-ended", sid).ok();
                            emits_since_yield += 1;
                        }
                    }
                }
                LogEvent::PlayerEventParsed(player_event) => {
                    // Persist to game state tables
                    let result = self.game_state.process_event(&player_event, &self.db_pool);

                    // Notify frontend which domains changed
                    if !result.domains_updated.is_empty() {
                        self.app_handle
                            .emit("game-state-updated", &result.domains_updated)
                            .ok();
                        emits_since_yield += 1;
                    }

                    // Still emit raw events for stores that haven't migrated yet
                    self.app_handle.emit("player-event", &player_event).ok();
                    emits_since_yield += 1;
                }
                _ => {
                    // Other events not yet implemented
                }
            }

            // Yield periodically to let the Windows message queue drain
            if emits_since_yield >= 100 && last_yield.elapsed() < Duration::from_millis(50) {
                std::thread::sleep(Duration::from_millis(5));
                last_yield = Instant::now();
                emits_since_yield = 0;
            } else if last_yield.elapsed() >= Duration::from_millis(50) {
                last_yield = Instant::now();
                emits_since_yield = 0;
            }
        }

        Ok(())
    }

    /// Process events from chat log
    fn process_chat_events(&mut self, events: Vec<LogEvent>) -> Result<(), String> {
        let mut messages = Vec::new();
        let mut last_yield = Instant::now();
        let mut emits_since_yield: u32 = 0;

        for event in events {
            match event {
                LogEvent::ChatMessage(msg) => {
                    // Run Status channel messages through the structured parser
                    if let Some(status_event) = parse_status_message(&msg) {
                        // Cross-reference with survey tracker for loot quantity correction.
                        // No active-session gate — correct_loot_from_chat_status falls back
                        // to last_session_id so corrections work after session auto-end.
                        if let Some(correction) = self
                            .survey_tracker
                            .correct_loot_from_chat_status(&status_event, &self.db_pool)
                        {
                            self.app_handle
                                .emit("survey-loot-correction", &correction)
                                .ok();
                            emits_since_yield += 1;
                        }
                        self.app_handle
                            .emit("chat-status-event", &status_event)
                            .ok();
                        emits_since_yield += 1;
                    }
                    messages.push(msg);
                }
                LogEvent::ServerDetected {
                    server_name,
                    character_name,
                    timezone_offset_seconds,
                } => {
                    startup_log!(
                        "Server detected: {} (character: {})",
                        server_name,
                        character_name
                    );

                    // Store timezone offset for UTC timestamp conversion
                    if let Some(offset) = timezone_offset_seconds {
                        startup_log!("Timezone offset detected: {}s from UTC", offset);
                        let mut settings = self.settings.get();
                        settings.timezone_offset_seconds = Some(offset);
                        self.settings.update(settings).ok();
                        self.game_state.set_timezone_offset(offset);
                        self.survey_tracker.set_timezone_offset(offset);
                    }

                    // Auto-create server record
                    if let Ok(conn) = self.db_pool.get() {
                        conn.execute(
                            "INSERT INTO servers (server_name) VALUES (?1) ON CONFLICT DO NOTHING",
                            rusqlite::params![server_name],
                        )
                        .ok();
                    }

                    // Update active server in settings
                    let mut settings = self.settings.get();
                    settings.active_server_name = Some(server_name.clone());
                    self.settings.update(settings).ok();

                    // Update game state so process_event doesn't early-return
                    self.game_state.set_active_server_name(&server_name);

                    // Emit to frontend
                    self.app_handle.emit("server-detected", &server_name).ok();
                    emits_since_yield += 1;
                }
                LogEvent::CharacterLogin { character_name, .. } => {
                    // Chat log also detects character login — update active character
                    // and emit so player_watcher can stay in sync
                    self.app_handle
                        .emit("character-login", &character_name)
                        .ok();
                    emits_since_yield += 1;

                    // Auto-register character with current server
                    if let Ok(conn) = self.db_pool.get() {
                        conn.execute(
                            "INSERT INTO user_characters (character_name, server_name, source, last_login_time)
                             VALUES (?1, COALESCE(?2, 'Unknown'), 'login', CURRENT_TIMESTAMP)
                             ON CONFLICT(character_name, server_name) DO UPDATE SET
                                last_login_time = CURRENT_TIMESTAMP,
                                updated_at = CURRENT_TIMESTAMP",
                            rusqlite::params![
                                character_name,
                                self.settings.get().active_server_name,
                            ],
                        ).ok();
                    }

                    // Update active character in settings
                    let mut settings = self.settings.get();
                    settings.active_character_name = Some(character_name.clone());
                    let server = settings
                        .active_server_name
                        .clone()
                        .unwrap_or_else(|| "Unknown".to_string());
                    self.settings.update(settings).ok();

                    // Update game state active character
                    self.game_state
                        .set_active_character(&character_name, &server, &self.db_pool);
                }
                _ => {}
            }

            // Yield periodically to let the Windows message queue drain
            if emits_since_yield >= 100 && last_yield.elapsed() < Duration::from_millis(50) {
                std::thread::sleep(Duration::from_millis(5));
                last_yield = Instant::now();
                emits_since_yield = 0;
            } else if last_yield.elapsed() >= Duration::from_millis(50) {
                last_yield = Instant::now();
                emits_since_yield = 0;
            }
        }

        // Batch insert messages
        if !messages.is_empty() {
            let conn = self
                .db_pool
                .get()
                .map_err(|e| format!("Database error: {}", e))?;

            // Use the actual file name from the watcher, not the position
            let log_file = self
                .chat_watcher
                .as_ref()
                .map(|w| w.get_file_name().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let excluded_channels = &self.settings.get().excluded_chat_channels;
            let inserted = insert_chat_messages(&conn, &messages, &log_file, excluded_channels)
                .map_err(|e| format!("Failed to insert messages: {}", e))?;

            if inserted > 0 {
                self.app_handle
                    .emit("chat-messages-inserted", inserted)
                    .ok();
            }

            // Evaluate watch rules against new messages
            let settings = self.settings.get();
            if !settings.watch_rules.is_empty() {
                for msg in &messages {
                    let triggered = evaluate_rules(msg, &settings.watch_rules);
                    for event in triggered {
                        self.app_handle.emit("watch-rule-triggered", &event).ok();
                    }
                }
            }
        }

        Ok(())
    }

    /// Emit status change event to frontend
    fn emit_status_change(&self) -> Result<(), String> {
        let status = self.get_status();
        self.app_handle
            .emit("coordinator-status", status)
            .map_err(|e| format!("Failed to emit event: {}", e))
    }
}

// ============================================================
// Tauri Commands (frontend interface)
// ============================================================

use std::sync::Mutex;
use tauri::State;

/// Start player log tailing (called from frontend)
#[tauri::command]
pub fn start_player_tailing(
    coordinator: State<'_, Arc<Mutex<DataIngestCoordinator>>>,
) -> Result<(), String> {
    let mut coord = coordinator.lock().unwrap();
    coord.start_player_log_tailing()
}

/// Stop player log tailing (called from frontend)
#[tauri::command]
pub fn stop_player_tailing(
    coordinator: State<'_, Arc<Mutex<DataIngestCoordinator>>>,
) -> Result<(), String> {
    let mut coord = coordinator.lock().unwrap();
    coord.stop_player_log_tailing()
}

/// Start chat log tailing (called from frontend)
#[tauri::command]
pub fn start_chat_tailing(
    coordinator: State<'_, Arc<Mutex<DataIngestCoordinator>>>,
    settings: State<'_, Arc<SettingsManager>>,
) -> Result<(), String> {
    // Get today's chat log file
    let chat_logs_dir = settings
        .get_chat_logs_dir()
        .ok_or_else(|| "Chat logs directory not configured".to_string())?;

    let today = chrono::Local::now();
    let date_str = format!(
        "{}-{:02}-{:02}",
        today.year() % 100,
        today.month(),
        today.day()
    );
    let chat_log_file = chat_logs_dir.join(format!("Chat-{}.log", date_str));

    let mut coord = coordinator.lock().unwrap();
    coord.start_chat_log_tailing(chat_log_file)
}

/// Stop chat log tailing (called from frontend)
#[tauri::command]
pub fn stop_chat_tailing(
    coordinator: State<'_, Arc<Mutex<DataIngestCoordinator>>>,
) -> Result<(), String> {
    let mut coord = coordinator.lock().unwrap();
    coord.stop_chat_log_tailing()
}

/// Get coordinator status (called from frontend)
#[tauri::command]
pub fn get_coordinator_status(
    coordinator: State<'_, Arc<Mutex<DataIngestCoordinator>>>,
) -> Result<CoordinatorStatus, String> {
    let coord = coordinator.lock().unwrap();
    Ok(coord.get_status())
}

/// Poll all watchers (called periodically from frontend or background task)
#[tauri::command]
pub fn poll_watchers(
    coordinator: State<'_, Arc<Mutex<DataIngestCoordinator>>>,
) -> Result<(), String> {
    let mut coord = coordinator.lock().unwrap();
    coord.poll()
}

// ============================================================
// Helpers
// ============================================================

/// Load known survey types from the database for the survey parser.
/// Returns a HashMap keyed by internal_name.
fn load_known_surveys(conn: &rusqlite::Connection) -> HashMap<String, KnownSurveyType> {
    let mut map = HashMap::new();
    let mut stmt = match conn.prepare("SELECT internal_name, name, is_motherlode FROM survey_types")
    {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[coordinator] Failed to load survey types: {e}");
            return map;
        }
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

    startup_log!("Loaded {} known survey types", map.len());
    map
}
