/// Log file watching infrastructure
///
/// This module provides the core abstractions for watching and parsing log files.
/// It includes:
/// - LogFileWatcher trait for unified file watching
/// - PlayerLogWatcher for monitoring Player.log
/// - ChatLogWatcher for monitoring chat log files

use std::collections::HashMap;
use std::path::PathBuf;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::fs::File;
use chrono::NaiveDateTime;
use crate::chat_parser::{ChatMessage, parse_chat_lines};
use crate::parsers::{SkillUpdate, parse_skill_update};
use crate::survey_parser::{SurveyParser, SurveyEvent, KnownSurveyType};
use crate::player_event_parser::{PlayerEventParser, PlayerEvent};

/// Type alias for pattern matcher functions
/// Takes a line and returns an optional LogEvent if the pattern matches
pub type PatternMatcher = fn(&str, &mut PlayerLogWatcher) -> Option<LogEvent>;

/// Events that can be emitted by log file watchers
#[derive(Debug, Clone)]
pub enum LogEvent {
    /// Chat message parsed
    ChatMessage(ChatMessage),

    /// Character logged in
    CharacterLogin {
        character_name: String,
        timestamp: NaiveDateTime,
        area: Option<String>,
    },

    /// Character logged out
    CharacterLogout {
        character_name: String,
        timestamp: NaiveDateTime,
    },

    /// Chat log file path announced
    ChatLogPath {
        path: String,
        timestamp: NaiveDateTime,
    },

    /// Area transition detected
    AreaTransition {
        area: String,
        timestamp: NaiveDateTime,
    },

    /// Survey completed
    SurveyCompleted {
        survey_type: String,
        quality: Option<u32>,
        timestamp: NaiveDateTime,
    },

    /// XP gained
    XpGained {
        skill: String,
        amount: u32,
        timestamp: NaiveDateTime,
    },

    /// Item looted
    ItemLooted {
        item_name: String,
        quantity: u32,
        timestamp: NaiveDateTime,
    },

    /// Skill update (ProcessUpdateSkill line)
    SkillUpdated(SkillUpdate),

    /// Survey event (crafting started, survey completed with loot)
    SurveyParsed(SurveyEvent),

    /// Player log event (items, skills, NPC interactions, vendor, storage, etc.)
    PlayerEventParsed(PlayerEvent),

    /// Generic log line that wasn't parsed
    Unparsed {
        line: String,
        timestamp: NaiveDateTime,
    },
}

/// Trait for watching and parsing log files
pub trait LogFileWatcher {
    /// Start watching the log file
    fn start(&mut self) -> Result<(), String>;

    /// Stop watching the log file
    fn stop(&mut self);

    /// Poll the file for new content and return parsed events
    fn poll(&mut self) -> Result<Vec<LogEvent>, String>;

    /// Get the current file position
    fn get_position(&self) -> u64;

    /// Check if the watcher is currently active
    fn is_active(&self) -> bool;
}

/// Watcher for Player.log - the master log file
pub struct PlayerLogWatcher {
    file_path: PathBuf,
    current_position: u64,
    active: bool,
    active_character: Option<String>,
    current_chat_log: Option<PathBuf>,
    pattern_matchers: Vec<PatternMatcher>,
    survey_parser: SurveyParser,
    player_event_parser: PlayerEventParser,
}

impl PlayerLogWatcher {
    /// Create a new PlayerLogWatcher with default pattern matchers
    pub fn new(file_path: PathBuf, known_surveys: HashMap<String, KnownSurveyType>) -> Self {
        let mut watcher = Self {
            file_path,
            current_position: 0,
            active: false,
            active_character: None,
            current_chat_log: None,
            pattern_matchers: Vec::new(),
            survey_parser: SurveyParser::new(known_surveys),
            player_event_parser: PlayerEventParser::new(),
        };

        watcher.register_core_patterns();
        watcher
    }

    /// Create from existing position (resume from database)
    pub fn from_position(file_path: PathBuf, position: u64, known_surveys: HashMap<String, KnownSurveyType>) -> Self {
        let mut watcher = Self {
            file_path,
            current_position: position,
            active: false,
            active_character: None,
            current_chat_log: None,
            pattern_matchers: Vec::new(),
            survey_parser: SurveyParser::new(known_surveys),
            player_event_parser: PlayerEventParser::new(),
        };

        watcher.register_core_patterns();
        watcher
    }

    /// Register a custom pattern matcher
    pub fn register_pattern(&mut self, matcher: PatternMatcher) {
        self.pattern_matchers.push(matcher);
    }

    /// Register the core patterns that are always active
    fn register_core_patterns(&mut self) {
        self.register_pattern(match_character_login);
        self.register_pattern(match_chat_log_path);
        self.register_pattern(match_area_transition);
        self.register_pattern(match_skill_update);
    }

    /// Get the currently active character name
    pub fn get_active_character(&self) -> Option<&str> {
        self.active_character.as_deref()
    }

    /// Get the current chat log path
    pub fn get_chat_log_path(&self) -> Option<&PathBuf> {
        self.current_chat_log.as_ref()
    }

    /// Parse a single line from Player.log
    fn parse_line(&mut self, line: &str) -> Option<LogEvent> {
        let matchers = self.pattern_matchers.clone();

        for matcher in matchers {
            if let Some(event) = matcher(line, self) {
                return Some(event);
            }
        }

        None
    }
}

// ============================================================
// Core Pattern Matchers
// ============================================================

/// Match character login pattern: "Logged in as character [CharacterName]"
fn match_character_login(line: &str, watcher: &mut PlayerLogWatcher) -> Option<LogEvent> {
    if let Some(start) = line.find("Logged in as character [") {
        if let Some(end) = line[start..].find(']') {
            let name_start = start + "Logged in as character [".len();
            let name_end = start + end;
            let character_name = line[name_start..name_end].to_string();

            watcher.active_character = Some(character_name.clone());

            return Some(LogEvent::CharacterLogin {
                character_name,
                timestamp: chrono::Local::now().naive_local(),
                area: None,
            });
        }
    }
    None
}

/// Match chat log path announcement: "Logging chat to C:/Users/.../ChatLogs/Chat-26-03-06.log"
fn match_chat_log_path(line: &str, watcher: &mut PlayerLogWatcher) -> Option<LogEvent> {
    if let Some(start) = line.find("Logging chat to ") {
        let path_start = start + "Logging chat to ".len();
        let path = line[path_start..].trim().to_string();

        watcher.current_chat_log = Some(PathBuf::from(&path));

        return Some(LogEvent::ChatLogPath {
            path,
            timestamp: chrono::Local::now().naive_local(),
        });
    }
    None
}

/// Match area transition: "LOADING LEVEL AreaCasino"
fn match_area_transition(line: &str, _watcher: &mut PlayerLogWatcher) -> Option<LogEvent> {
    if let Some(start) = line.find("LOADING LEVEL ") {
        let area_start = start + "LOADING LEVEL ".len();
        let area = line[area_start..].trim().to_string();

        return Some(LogEvent::AreaTransition {
            area,
            timestamp: chrono::Local::now().naive_local(),
        });
    }
    None
}

/// Match skill update: "ProcessUpdateSkill" lines
fn match_skill_update(line: &str, _watcher: &mut PlayerLogWatcher) -> Option<LogEvent> {
    parse_skill_update(line).map(LogEvent::SkillUpdated)
}

impl LogFileWatcher for PlayerLogWatcher {
    fn start(&mut self) -> Result<(), String> {
        if !self.file_path.exists() {
            return Err(format!("Player.log not found at {:?}", self.file_path));
        }

        self.active = true;
        Ok(())
    }

    fn stop(&mut self) {
        self.active = false;
    }

    fn poll(&mut self) -> Result<Vec<LogEvent>, String> {
        if !self.active {
            return Ok(Vec::new());
        }

        let mut file = File::open(&self.file_path)
            .map_err(|e| format!("Failed to open Player.log: {}", e))?;

        file.seek(SeekFrom::Start(self.current_position))
            .map_err(|e| format!("Failed to seek in Player.log: {}", e))?;

        let reader = BufReader::new(file);
        let mut events = Vec::new();

        for line in reader.lines() {
            match line {
                Ok(line_content) => {
                    self.current_position += line_content.len() as u64 + 1;

                    if let Some(event) = self.parse_line(&line_content) {
                        events.push(event);
                    }

                    // Feed every line through the player event parser first
                    let player_events = self.player_event_parser.process_line(&line_content);

                    // Feed player events + raw line into the survey parser
                    // (raw line still needed for ProcessMapFx which is survey-specific)
                    let survey_events = self.survey_parser.process_events(&player_events, &line_content);
                    for se in survey_events {
                        events.push(LogEvent::SurveyParsed(se));
                    }

                    for pe in player_events {
                        events.push(LogEvent::PlayerEventParsed(pe));
                    }
                }
                Err(e) => {
                    eprintln!("Error reading line from Player.log: {}", e);
                    break;
                }
            }
        }

        // Flush any remaining pending deletes at end of poll
        let flush_events = self.player_event_parser.flush_all_pending();
        for pe in flush_events {
            events.push(LogEvent::PlayerEventParsed(pe));
        }

        Ok(events)
    }

    fn get_position(&self) -> u64 {
        self.current_position
    }

    fn is_active(&self) -> bool {
        self.active
    }
}

/// Watcher for chat log files
///
/// Chat logs are daily files (Chat-YY-MM-DD.log) that span multiple play sessions.
/// They contain login/logout markers: `******** Logged In As [Name]` / `******** Logged Out`
///
/// This watcher handles:
/// - Session tracking within daily files
/// - Message parsing and deduplication
/// - Position tracking for incremental reads
/// - Settings-driven channel exclusion
pub struct ChatLogWatcher {
    file_path: PathBuf,
    current_position: u64,
    active: bool,
    player_name: Option<String>,
    current_session_start: Option<NaiveDateTime>,
    excluded_channels: Vec<String>,
}

impl ChatLogWatcher {
    /// Create a new ChatLogWatcher with excluded channels from settings
    pub fn new(file_path: PathBuf, excluded_channels: Vec<String>) -> Self {
        Self {
            file_path,
            current_position: 0,
            active: false,
            player_name: None,
            current_session_start: None,
            excluded_channels,
        }
    }

    /// Create from existing position (resume from database)
    pub fn from_position(file_path: PathBuf, position: u64, excluded_channels: Vec<String>) -> Self {
        Self {
            file_path,
            current_position: position,
            active: false,
            player_name: None,
            current_session_start: None,
            excluded_channels,
        }
    }

    /// Set the player name for this chat log
    pub fn set_player_name(&mut self, name: String) {
        self.player_name = Some(name);
    }

    /// Get the current session start time
    pub fn get_session_start(&self) -> Option<NaiveDateTime> {
        self.current_session_start
    }

    /// Get the file name (just the filename component, not the full path)
    pub fn get_file_name(&self) -> &str {
        self.file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }

    /// Get the full file path
    pub fn get_file_path(&self) -> &PathBuf {
        &self.file_path
    }

    /// Check a line for session markers (login/logout).
    /// Returns Some(LogEvent) if the line is a session marker.
    fn check_session_marker(&mut self, line: &str) -> Option<LogEvent> {
        // Check for login marker: ******** Logged In As [Name]
        if line.contains("******** Logged In As") {
            if let Some(start) = line.find('[') {
                if let Some(end) = line.find(']') {
                    let name = line[start + 1..end].to_string();
                    self.player_name = Some(name.clone());
                    self.current_session_start = Some(chrono::Local::now().naive_local());

                    return Some(LogEvent::CharacterLogin {
                        character_name: name,
                        timestamp: chrono::Local::now().naive_local(),
                        area: None,
                    });
                }
            }
        }

        // Check for logout marker: ******** Logged Out
        if line.contains("******** Logged Out") {
            if let Some(player) = &self.player_name {
                let event = LogEvent::CharacterLogout {
                    character_name: player.clone(),
                    timestamp: chrono::Local::now().naive_local(),
                };
                self.current_session_start = None;
                return Some(event);
            }
        }

        None
    }
}

impl LogFileWatcher for ChatLogWatcher {
    fn start(&mut self) -> Result<(), String> {
        if !self.file_path.exists() {
            return Err(format!("Chat log not found at {:?}", self.file_path));
        }

        self.active = true;
        Ok(())
    }

    fn stop(&mut self) {
        self.active = false;
    }

    fn poll(&mut self) -> Result<Vec<LogEvent>, String> {
        use std::io::Read;

        if !self.active {
            return Ok(Vec::new());
        }

        let mut file = File::open(&self.file_path)
            .map_err(|e| format!("Failed to open chat log: {}", e))?;

        let file_size = file.metadata()
            .map_err(|e| format!("Failed to get file metadata: {}", e))?
            .len();

        // Handle file shrink (game recreated file)
        if self.current_position > file_size {
            self.current_position = 0;
        }

        if self.current_position >= file_size {
            return Ok(Vec::new());
        }

        file.seek(SeekFrom::Start(self.current_position))
            .map_err(|e| format!("Failed to seek in chat log: {}", e))?;

        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .map_err(|e| format!("Failed to read chat log: {}", e))?;

        self.current_position = file_size;

        let content_str = String::from_utf8_lossy(&content);
        let mut events = Vec::new();

        // Check each line for session markers first
        for line in content_str.lines() {
            if let Some(event) = self.check_session_marker(line) {
                events.push(event);
            }
        }

        // Parse all chat messages using the multiline-aware parser
        let messages = parse_chat_lines(&content_str, &self.excluded_channels);
        for msg in messages {
            events.push(LogEvent::ChatMessage(msg));
        }

        Ok(events)
    }

    fn get_position(&self) -> u64 {
        self.current_position
    }

    fn is_active(&self) -> bool {
        self.active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_character_login() {
        let mut watcher = PlayerLogWatcher::new(PathBuf::from("test.log"), HashMap::new());

        let event = watcher.parse_line("Logged in as character [TestCharacter]");
        assert!(event.is_some());

        if let Some(LogEvent::CharacterLogin { character_name, .. }) = event {
            assert_eq!(character_name, "TestCharacter");
        } else {
            panic!("Expected CharacterLogin event");
        }

        assert_eq!(watcher.get_active_character(), Some("TestCharacter"));
    }

    #[test]
    fn test_parse_chat_log_path() {
        let mut watcher = PlayerLogWatcher::new(PathBuf::from("test.log"), HashMap::new());

        let event = watcher.parse_line("Logging chat to C:/Users/Test/ChatLogs/Chat-26-03-06.log");
        assert!(event.is_some());

        if let Some(LogEvent::ChatLogPath { path, .. }) = event {
            assert_eq!(path, "C:/Users/Test/ChatLogs/Chat-26-03-06.log");
        } else {
            panic!("Expected ChatLogPath event");
        }

        assert!(watcher.get_chat_log_path().is_some());
    }

    #[test]
    fn test_parse_area_transition() {
        let mut watcher = PlayerLogWatcher::new(PathBuf::from("test.log"), HashMap::new());

        let event = watcher.parse_line("LOADING LEVEL AreaCasino");
        assert!(event.is_some());

        if let Some(LogEvent::AreaTransition { area, .. }) = event {
            assert_eq!(area, "AreaCasino");
        } else {
            panic!("Expected AreaTransition event");
        }
    }

    #[test]
    fn test_chat_login_marker() {
        let mut watcher = ChatLogWatcher::new(PathBuf::from("test.log"), vec![]);

        let event = watcher.check_session_marker("******** Logged In As [TestCharacter]");
        assert!(event.is_some());

        if let Some(LogEvent::CharacterLogin { character_name, .. }) = event {
            assert_eq!(character_name, "TestCharacter");
        } else {
            panic!("Expected CharacterLogin event");
        }

        assert_eq!(watcher.player_name, Some("TestCharacter".to_string()));
        assert!(watcher.current_session_start.is_some());
    }

    #[test]
    fn test_chat_logout_marker() {
        let mut watcher = ChatLogWatcher::new(PathBuf::from("test.log"), vec![]);
        watcher.player_name = Some("TestCharacter".to_string());

        let event = watcher.check_session_marker("******** Logged Out");
        assert!(event.is_some());

        if let Some(LogEvent::CharacterLogout { character_name, .. }) = event {
            assert_eq!(character_name, "TestCharacter");
        } else {
            panic!("Expected CharacterLogout event");
        }

        assert!(watcher.current_session_start.is_none());
    }

    #[test]
    fn test_chat_message_parsing() {
        let line = "26-03-09 14:23:45\t[Global] TestPlayer: Hello world!";
        let messages = parse_chat_lines(line, &[]);

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].channel, Some("Global".to_string()));
        assert_eq!(messages[0].sender, Some("TestPlayer".to_string()));
        assert_eq!(messages[0].message, "Hello world!");
    }

    #[test]
    fn test_chat_message_excluded_channel() {
        let excluded = vec!["Global".to_string()];

        let line = "26-03-09 14:23:45\t[Global] TestPlayer: Hello world!";
        let messages = parse_chat_lines(line, &excluded);

        // Should be empty because Global is excluded
        assert!(messages.is_empty());
    }

    #[test]
    fn test_chat_watcher_get_file_name() {
        let watcher = ChatLogWatcher::new(
            PathBuf::from("/some/path/Chat-26-03-09.log"),
            vec![],
        );
        assert_eq!(watcher.get_file_name(), "Chat-26-03-09.log");
    }

    #[test]
    fn test_custom_pattern_registration() {
        let mut watcher = PlayerLogWatcher::new(PathBuf::from("test.log"), HashMap::new());

        fn match_xp_gain(line: &str, _watcher: &mut PlayerLogWatcher) -> Option<LogEvent> {
            if line.contains("You gain") && line.contains("experience in") {
                Some(LogEvent::XpGained {
                    skill: "TestSkill".to_string(),
                    amount: 100,
                    timestamp: chrono::Local::now().naive_local(),
                })
            } else {
                None
            }
        }

        watcher.register_pattern(match_xp_gain);

        let event = watcher.parse_line("You gain 100 experience in TestSkill");
        assert!(event.is_some());

        if let Some(LogEvent::XpGained { skill, amount, .. }) = event {
            assert_eq!(skill, "TestSkill");
            assert_eq!(amount, 100);
        } else {
            panic!("Expected XpGained event from custom pattern");
        }
    }

    #[test]
    fn test_pattern_registration_order() {
        let mut watcher = PlayerLogWatcher::new(PathBuf::from("test.log"), HashMap::new());

        let event1 = watcher.parse_line("Logged in as character [TestChar]");
        assert!(matches!(event1, Some(LogEvent::CharacterLogin { .. })));

        let event2 = watcher.parse_line("LOADING LEVEL TestArea");
        assert!(matches!(event2, Some(LogEvent::AreaTransition { .. })));
    }
}
