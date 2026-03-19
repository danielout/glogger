/// DataIngestCoordinator - Central coordinator for all file watching and database operations
///
/// This coordinator manages:
/// - PlayerLogWatcher (master log - tracks active character and chat log path)
/// - ChatLogWatcher (daily chat logs)
/// - Operation locking to prevent conflicts
/// - Database write coordination
/// - Progress event emission to frontend

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use crate::log_watchers::{PlayerLogWatcher, ChatLogWatcher, LogFileWatcher, LogEvent};
use crate::db::DbPool;
use crate::db::chat_commands::insert_chat_messages;
use crate::db::queries::log_positions;
use crate::settings::SettingsManager;
use crate::watch_rules::evaluate_rules;
use crate::survey_persistence::SurveySessionTracker;
use crate::survey_parser::KnownSurveyType;
use serde::Serialize;
use chrono::Datelike;

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
    CharacterImport {
        progress: usize,
        total: usize,
    },

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
}

impl DataIngestCoordinator {
    /// Create a new coordinator
    pub fn new(
        db_pool: DbPool,
        settings: Arc<SettingsManager>,
        app_handle: AppHandle,
    ) -> Result<Self, String> {
        Ok(Self {
            player_watcher: None,
            chat_watcher: None,
            operation_lock: Arc::new(RwLock::new(OperationType::Idle)),
            db_pool,
            settings,
            app_handle,
            survey_tracker: SurveySessionTracker::new(),
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
            player_log_active: self.player_watcher.as_ref().map_or(false, |w| w.is_active()),
            chat_log_active: self.chat_watcher.as_ref().map_or(false, |w| w.is_active()),
            active_character: self.player_watcher.as_ref().and_then(|w| w.get_active_character().map(String::from)),
            current_chat_log: self.player_watcher.as_ref().and_then(|w| w.get_chat_log_path().and_then(|p| p.to_str().map(String::from))),
            operation: operation_str.to_string(),
        }
    }

    /// Start player log tailing
    pub fn start_player_log_tailing(&mut self) -> Result<(), String> {
        // Check for blocking operations (full scans, imports, etc.)
        let operation = self.operation_lock.read().unwrap();
        if *operation != OperationType::Idle {
            return Err(format!("Cannot start player log tailing: {:?} in progress", *operation));
        }
        drop(operation);

        // Already tailing? No-op.
        if self.player_watcher.as_ref().map_or(false, |w| w.is_active()) {
            return Ok(());
        }

        // Get player log path from settings
        let player_log_path = self.settings.get_player_log_path()
            .ok_or_else(|| "Game data path not configured".to_string())?;

        // Load saved position from database
        let conn = self.db_pool.get().map_err(|e| format!("Database error: {}", e))?;
        let position = log_positions::get_position(&conn, player_log_path.to_str().unwrap_or("")).unwrap_or(0);

        // Load known survey types for the survey parser
        let known_surveys = load_known_surveys(&conn);
        drop(conn);

        // Create watcher
        let mut watcher = if position > 0 {
            PlayerLogWatcher::from_position(player_log_path, position, known_surveys)
        } else {
            PlayerLogWatcher::new(player_log_path, known_surveys)
        };

        // Start watching
        watcher.start()?;

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
                let conn = self.db_pool.get().map_err(|e| format!("Database error: {}", e))?;
                log_positions::update_position(
                    &conn,
                    path.to_str().unwrap_or(""),
                    "player",
                    position,
                    watcher.get_active_character(),
                    None,
                ).map_err(|e| format!("Failed to save position: {}", e))?;
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
        let conn = self.db_pool.get().map_err(|e| format!("Database error: {}", e))?;
        let position = log_positions::get_position(&conn, chat_log_path.to_str().unwrap_or("")).unwrap_or(0);
        drop(conn);

        // Get excluded channels from settings
        let excluded_channels = self.settings.get().excluded_chat_channels;

        // Create watcher with excluded channels
        let mut watcher = if position > 0 {
            ChatLogWatcher::from_position(chat_log_path, position, excluded_channels)
        } else {
            ChatLogWatcher::new(chat_log_path, excluded_channels)
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

            let conn = self.db_pool.get().map_err(|e| format!("Database error: {}", e))?;
            let metadata = serde_json::json!({ "file_name": file_name }).to_string();
            log_positions::update_position(
                &conn,
                &file_path_str,
                "chat",
                position,
                None,
                Some(&metadata),
            ).map_err(|e| format!("Failed to save position: {}", e))?;
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

        // Poll chat log watcher
        if let Some(watcher) = &mut self.chat_watcher {
            let events = watcher.poll()?;
            self.process_chat_events(events)?;
        }

        Ok(())
    }

    /// Process events from player log
    fn process_player_events(&mut self, events: Vec<LogEvent>) -> Result<(), String> {
        for event in events {
            match event {
                LogEvent::CharacterLogin { character_name, .. } => {
                    // Emit character login event
                    self.app_handle.emit("character-login", &character_name).ok();

                    // Auto-register character in user_characters table
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
                    settings.active_character_name = Some(character_name);
                    self.settings.update(settings).ok();
                }
                LogEvent::ChatLogPath { path, .. } => {
                    // Chat log path changed - start/switch chat watcher
                    // start_chat_log_tailing handles stopping the old watcher if needed
                    self.start_chat_log_tailing(PathBuf::from(&path))?;
                }
                LogEvent::AreaTransition { area, .. } => {
                    self.app_handle.emit("area-transition", &area).ok();
                }
                LogEvent::SkillUpdated(update) => {
                    self.app_handle.emit("skill-update", &update).ok();
                }
                LogEvent::SurveyParsed(survey_event) => {
                    // Persist to DB synchronously first, then emit to frontend
                    let result = self.survey_tracker.process_event(&survey_event, &self.db_pool);
                    self.app_handle.emit("survey-event", &survey_event).ok();

                    // If the session auto-ended, notify frontend so it can patch in
                    // elapsed/XP data that only the frontend knows about
                    if result.session_ended {
                        if let Some(sid) = result.session_id {
                            self.app_handle.emit("survey-session-ended", sid).ok();
                        }
                    }
                }
                LogEvent::PlayerEventParsed(player_event) => {
                    // Emit to frontend — no DB persistence yet, features will add their own
                    self.app_handle.emit("player-event", &player_event).ok();
                }
                _ => {
                    // Other events not yet implemented
                }
            }
        }

        Ok(())
    }

    /// Process events from chat log
    fn process_chat_events(&mut self, events: Vec<LogEvent>) -> Result<(), String> {
        let mut messages = Vec::new();

        for event in events {
            if let LogEvent::ChatMessage(msg) = event {
                messages.push(msg);
            }
        }

        // Batch insert messages
        if !messages.is_empty() {
            let conn = self.db_pool.get().map_err(|e| format!("Database error: {}", e))?;

            // Use the actual file name from the watcher, not the position
            let log_file = self.chat_watcher.as_ref()
                .map(|w| w.get_file_name().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let excluded_channels = &self.settings.get().excluded_chat_channels;
            let inserted = insert_chat_messages(&conn, &messages, &log_file, excluded_channels)
                .map_err(|e| format!("Failed to insert messages: {}", e))?;

            if inserted > 0 {
                self.app_handle.emit("chat-messages-inserted", inserted).ok();
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
        self.app_handle.emit("coordinator-status", status)
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
    let chat_logs_dir = settings.get_chat_logs_dir()
        .ok_or_else(|| "Chat logs directory not configured".to_string())?;

    let today = chrono::Local::now();
    let date_str = format!("{}-{:02}-{:02}",
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
    let mut stmt = match conn.prepare(
        "SELECT internal_name, name, is_motherlode FROM survey_types"
    ) {
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
            map.insert(internal_name, KnownSurveyType {
                display_name,
                is_motherlode,
            });
        }
    }

    eprintln!("[coordinator] Loaded {} known survey types", map.len());
    map
}
