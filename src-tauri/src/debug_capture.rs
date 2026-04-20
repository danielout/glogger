//! Debug capture system for recording raw log lines and game state snapshots.
//!
//! When a capture is active, raw Player.log and Chat.log lines are written to
//! temporary files on disk to avoid unbounded memory growth during long sessions.
//! Game state is snapshotted at start and stop. The whole bundle is assembled
//! into a single JSON file for debugging and support purposes.
//!
//! ## Two-phase stop/save flow
//!
//! 1. `stop()` — ends recording and takes the final game state snapshot. The
//!    temp file and snapshots are kept so the user can review stats and edit
//!    notes before saving.
//! 2. `save()` — writes the bundle JSON to the chosen path with the user's
//!    notes and selected filter mode (normal or full).
//! 3. `discard()` — can be called at any time (active or stopped) to throw
//!    away captured data without saving.

use crate::db::DbPool;
use chrono::{Local, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

/// A single captured log line with its source and timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedLine {
    /// When this line was captured (local clock, HH:MM:SS.mmm).
    pub captured_at: String,
    /// "player" or "chat"
    pub source: String,
    /// The raw line content.
    pub line: String,
}

/// Snapshot of game state tables taken at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateSnapshot {
    pub taken_at: String,
    pub character: Option<String>,
    pub server: Option<String>,
    pub skills: Vec<serde_json::Value>,
    pub attributes: Vec<serde_json::Value>,
    pub inventory: Vec<serde_json::Value>,
    pub equipment: Vec<serde_json::Value>,
    pub effects: Vec<serde_json::Value>,
    pub favor: Vec<serde_json::Value>,
    pub currencies: Vec<serde_json::Value>,
    pub world: serde_json::Value,
    pub active_skills: serde_json::Value,
}

/// Status of the debug capture, sent to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct DebugCaptureStatus {
    pub active: bool,
    /// True when recording has stopped but data hasn't been saved or discarded yet.
    pub pending_save: bool,
    pub started_at: Option<String>,
    pub line_count: usize,
    pub player_line_count: usize,
    pub chat_line_count: usize,
}

/// Result returned after saving a capture.
#[derive(Debug, Clone, Serialize)]
pub struct DebugCaptureResult {
    pub line_count: usize,
    pub player_line_count: usize,
    pub chat_line_count: usize,
}

/// Manages the state of an active debug capture.
///
/// Raw log lines are appended to a temp file as tab-separated records
/// (`source\tcaptured_at\tline`) to keep memory usage constant.
pub struct DebugCaptureState {
    active: bool,
    /// True when recording has stopped but data hasn't been saved/discarded yet.
    pending_save: bool,
    started_at: Option<String>,
    stopped_at: Option<String>,
    player_line_count: usize,
    chat_line_count: usize,
    state_at_start: Option<GameStateSnapshot>,
    state_at_stop: Option<GameStateSnapshot>,
    app_version: Option<String>,
    /// Temp file for captured lines. Created on start, deleted after export.
    temp_path: Option<PathBuf>,
    temp_writer: Option<BufWriter<fs::File>>,
}

impl DebugCaptureState {
    pub fn new() -> Self {
        Self {
            active: false,
            pending_save: false,
            started_at: None,
            stopped_at: None,
            player_line_count: 0,
            chat_line_count: 0,
            state_at_start: None,
            state_at_stop: None,
            app_version: None,
            temp_path: None,
            temp_writer: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_pending_save(&self) -> bool {
        self.pending_save
    }

    pub fn status(&self) -> DebugCaptureStatus {
        DebugCaptureStatus {
            active: self.active,
            pending_save: self.pending_save,
            started_at: self.started_at.clone(),
            line_count: self.player_line_count + self.chat_line_count,
            player_line_count: self.player_line_count,
            chat_line_count: self.chat_line_count,
        }
    }

    /// Start a new capture. Takes an initial game state snapshot.
    /// `temp_dir` is the directory to create the temp file in (typically app data dir).
    pub fn start(
        &mut self,
        snapshot: GameStateSnapshot,
        temp_dir: &std::path::Path,
    ) -> Result<(), String> {
        // Clean up any leftover temp file from a previous capture
        self.cleanup_temp();

        let temp_path = temp_dir.join("debug-capture-lines.tmp");
        let file = fs::File::create(&temp_path)
            .map_err(|e| format!("Failed to create capture temp file: {e}"))?;

        self.active = true;
        self.pending_save = false;
        self.started_at = Some(Utc::now().to_rfc3339());
        self.stopped_at = None;
        self.player_line_count = 0;
        self.chat_line_count = 0;
        self.state_at_start = Some(snapshot);
        self.state_at_stop = None;
        self.app_version = None;
        self.temp_writer = Some(BufWriter::new(file));
        self.temp_path = Some(temp_path);

        Ok(())
    }

    /// Stop recording and take the final snapshot. The capture data is kept
    /// in the temp file so the user can review and edit notes before saving.
    pub fn stop(
        &mut self,
        snapshot: GameStateSnapshot,
        app_version: String,
    ) -> Result<DebugCaptureResult, String> {
        if !self.active {
            return Err("No debug capture is active".to_string());
        }

        self.active = false;
        self.pending_save = true;
        self.stopped_at = Some(Utc::now().to_rfc3339());
        self.state_at_stop = Some(snapshot);
        self.app_version = Some(app_version);

        // Flush and close the temp writer (keep the file for save)
        if let Some(mut writer) = self.temp_writer.take() {
            writer
                .flush()
                .map_err(|e| format!("Failed to flush capture temp file: {e}"))?;
        }

        Ok(DebugCaptureResult {
            line_count: self.player_line_count + self.chat_line_count,
            player_line_count: self.player_line_count,
            chat_line_count: self.chat_line_count,
        })
    }

    /// Save the stopped capture to a file. `filter_mode` is "normal" (filter
    /// noise) or "full" (keep everything).
    pub fn save(
        &mut self,
        notes: String,
        filter_mode: String,
        output_path: &str,
    ) -> Result<DebugCaptureResult, String> {
        if !self.pending_save {
            return Err("No stopped capture is pending save".to_string());
        }

        let started_at = self.started_at.take().unwrap_or_default();
        let stopped_at = self.stopped_at.take().unwrap_or_default();
        let state_at_start = self
            .state_at_start
            .take()
            .ok_or_else(|| "Missing start snapshot".to_string())?;
        let state_at_stop = self
            .state_at_stop
            .take()
            .ok_or_else(|| "Missing stop snapshot".to_string())?;
        let app_version = self.app_version.take().unwrap_or_else(|| "unknown".to_string());

        // Read captured lines from temp file
        let use_filter = filter_mode == "normal";
        let lines = self.read_temp_lines(use_filter);

        let filtered_player = lines.iter().filter(|l| l.source == "player").count();
        let filtered_chat = lines.iter().filter(|l| l.source == "chat").count();

        let result = DebugCaptureResult {
            line_count: filtered_player + filtered_chat,
            player_line_count: filtered_player,
            chat_line_count: filtered_chat,
        };

        // Build the output JSON
        let bundle = serde_json::json!({
            "format_version": 2,
            "app_version": app_version,
            "captured_by": "glogger debug capture",
            "filter_mode": filter_mode,
            "started_at": started_at,
            "stopped_at": stopped_at,
            "notes": notes,
            "state_at_start": state_at_start,
            "state_at_stop": state_at_stop,
            "line_count": result.line_count,
            "player_line_count": result.player_line_count,
            "chat_line_count": result.chat_line_count,
            "unfiltered_line_count": self.player_line_count + self.chat_line_count,
            "lines": lines,
        });

        let out_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {e}"))?;
        let writer = BufWriter::new(out_file);
        serde_json::to_writer_pretty(writer, &bundle)
            .map_err(|e| format!("Failed to write capture JSON: {e}"))?;

        // Clean up
        self.cleanup_temp();
        self.pending_save = false;
        self.player_line_count = 0;
        self.chat_line_count = 0;

        Ok(result)
    }

    /// Discard the capture (active or pending save) without saving.
    pub fn discard(&mut self) {
        self.active = false;
        self.pending_save = false;
        self.started_at = None;
        self.stopped_at = None;
        self.state_at_start = None;
        self.state_at_stop = None;
        self.app_version = None;
        self.player_line_count = 0;
        self.chat_line_count = 0;
        if let Some(mut writer) = self.temp_writer.take() {
            let _ = writer.flush();
        }
        self.cleanup_temp();
    }

    /// Append a raw log line to the temp file.
    pub fn push_line(&mut self, source: &'static str, line: String) {
        if !self.active {
            return;
        }
        match source {
            "player" => self.player_line_count += 1,
            "chat" => self.chat_line_count += 1,
            _ => {}
        }
        if let Some(writer) = &mut self.temp_writer {
            let ts = Local::now().format("%H:%M:%S%.3f");
            // Tab-separated: source \t timestamp \t line
            let _ = writeln!(writer, "{}\t{}\t{}", source, ts, line);
        }
    }

    /// Read all captured lines from the temp file into CapturedLine structs.
    /// When `filter_noise` is true, known noise patterns are excluded.
    fn read_temp_lines(&self, filter_noise: bool) -> Vec<CapturedLine> {
        let path = match &self.temp_path {
            Some(p) if p.exists() => p,
            _ => return vec![],
        };
        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return vec![],
        };
        let reader = BufReader::new(file);
        reader
            .lines()
            .filter_map(|line| {
                let line = line.ok()?;
                let mut parts = line.splitn(3, '\t');
                let source = parts.next()?.to_string();
                let captured_at = parts.next()?.to_string();
                let content = parts.next()?.to_string();

                if filter_noise && source == "player" && is_noise_line(&content) {
                    return None;
                }

                Some(CapturedLine {
                    captured_at,
                    source,
                    line: content,
                })
            })
            .collect()
    }

    /// Remove the temp file if it exists.
    fn cleanup_temp(&mut self) {
        self.temp_writer = None;
        if let Some(path) = self.temp_path.take() {
            let _ = fs::remove_file(&path);
        }
    }
}

impl Drop for DebugCaptureState {
    fn drop(&mut self) {
        self.cleanup_temp();
    }
}

/// Check whether a player log line is engine/rendering noise that should be
/// filtered out in "normal" capture mode. Chat lines are never filtered.
fn is_noise_line(line: &str) -> bool {
    // Empty or very short fragment lines (truncation artifacts)
    if line.len() < 4 {
        return true;
    }

    // Exact prefix matches — high-confidence noise patterns
    if line.starts_with("Download appearance loop")
        || line.starts_with("LoadAssetAsync")
        || line.starts_with("IsDoneLoading")
        || line.starts_with("Successfully downloaded Texture")
        || line.starts_with("Cannot remove: entity doesn't have particle")
        || line.starts_with("Ref-count cleanup")
        || line.starts_with("ClearCursor")
        || line.starts_with("Animator.GotoState")
        || line.starts_with("BoxColliders created at Runtime")
        || line.starts_with("Combined Static Meshes")
        || line.starts_with("Either create the Box Collider")
        || line.starts_with("MecanimEx:")
        || line.starts_with("Told to do animation")
        || line.starts_with("Shader ")
        || line.starts_with("New Network State")
    {
        return true;
    }

    // Contains matches
    if line.contains("ProcessMusicPerformance(MusicPerformanceManager+PerformanceInfo)") {
        return true;
    }

    // Sound events: "1234.567: Playing sound ..." or just "Playing sound ..."
    if line.contains(": Playing sound ") || line.starts_with("Playing sound ") {
        return true;
    }

    // Asset bundle loading noise
    if line.starts_with("Completed ") && line.contains("asset bundle") {
        return true;
    }

    false
}

/// Take a snapshot of all game state tables from the database.
pub fn snapshot_game_state(
    db: &DbPool,
    character: Option<&str>,
    server: Option<&str>,
) -> GameStateSnapshot {
    let conn = match db.get() {
        Ok(c) => c,
        Err(_) => {
            return GameStateSnapshot {
                taken_at: Utc::now().to_rfc3339(),
                character: character.map(String::from),
                server: server.map(String::from),
                skills: vec![],
                attributes: vec![],
                inventory: vec![],
                equipment: vec![],
                effects: vec![],
                favor: vec![],
                currencies: vec![],
                world: serde_json::Value::Null,
                active_skills: serde_json::Value::Null,
            };
        }
    };

    let (char_filter, server_filter) = match (character, server) {
        (Some(c), Some(s)) => (c.to_string(), s.to_string()),
        _ => (String::new(), String::new()),
    };

    GameStateSnapshot {
        taken_at: Utc::now().to_rfc3339(),
        character: character.map(String::from),
        server: server.map(String::from),
        skills: query_table_as_json(&conn, "game_state_skills", &char_filter, &server_filter),
        attributes: query_table_as_json(
            &conn,
            "game_state_attributes",
            &char_filter,
            &server_filter,
        ),
        inventory: query_table_as_json(
            &conn,
            "game_state_inventory",
            &char_filter,
            &server_filter,
        ),
        equipment: query_table_as_json(
            &conn,
            "game_state_equipment",
            &char_filter,
            &server_filter,
        ),
        effects: query_table_as_json(&conn, "game_state_effects", &char_filter, &server_filter),
        favor: query_table_as_json(&conn, "game_state_favor", &char_filter, &server_filter),
        currencies: query_table_as_json(
            &conn,
            "game_state_currencies",
            &char_filter,
            &server_filter,
        ),
        world: query_world_state(&conn, &char_filter, &server_filter),
        active_skills: query_active_skills(&conn, &char_filter, &server_filter),
    }
}

/// Generic helper: query all rows from a game_state table as JSON values.
fn query_table_as_json(
    conn: &rusqlite::Connection,
    table: &str,
    character: &str,
    server: &str,
) -> Vec<serde_json::Value> {
    // These tables are safe — names come from our own code, not user input.
    let sql = if character.is_empty() {
        format!("SELECT * FROM {} LIMIT 5000", table)
    } else {
        format!(
            "SELECT * FROM {} WHERE character_name = ?1 AND server_name = ?2 LIMIT 5000",
            table
        )
    };

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let column_names: Vec<String> = stmt
        .column_names()
        .iter()
        .map(|c| c.to_string())
        .collect();

    let params: Vec<&dyn rusqlite::types::ToSql> = if character.is_empty() {
        vec![]
    } else {
        vec![&character, &server]
    };

    let rows = stmt.query_map(params.as_slice(), |row| {
        let mut map = serde_json::Map::new();
        for (i, name) in column_names.iter().enumerate() {
            let val: rusqlite::Result<rusqlite::types::Value> = row.get(i);
            match val {
                Ok(rusqlite::types::Value::Null) => {
                    map.insert(name.clone(), serde_json::Value::Null);
                }
                Ok(rusqlite::types::Value::Integer(n)) => {
                    map.insert(name.clone(), serde_json::json!(n));
                }
                Ok(rusqlite::types::Value::Real(f)) => {
                    map.insert(name.clone(), serde_json::json!(f));
                }
                Ok(rusqlite::types::Value::Text(s)) => {
                    map.insert(name.clone(), serde_json::json!(s));
                }
                Ok(rusqlite::types::Value::Blob(b)) => {
                    map.insert(
                        name.clone(),
                        serde_json::json!(format!("<blob {} bytes>", b.len())),
                    );
                }
                Err(_) => {
                    map.insert(name.clone(), serde_json::Value::Null);
                }
            }
        }
        Ok(serde_json::Value::Object(map))
    });

    match rows {
        Ok(mapped) => mapped.filter_map(|r| r.ok()).collect(),
        Err(_) => vec![],
    }
}

/// Query the composite world state (weather, combat, mount, area).
fn query_world_state(
    conn: &rusqlite::Connection,
    character: &str,
    server: &str,
) -> serde_json::Value {
    let mut world = serde_json::Map::new();

    for (key, table) in &[
        ("weather", "game_state_weather"),
        ("combat", "game_state_combat"),
        ("mount", "game_state_mount"),
        ("area", "game_state_area"),
    ] {
        let rows = query_table_as_json(conn, table, character, server);
        if let Some(row) = rows.into_iter().next() {
            world.insert(key.to_string(), row);
        }
    }

    serde_json::Value::Object(world)
}

/// Query active skills.
fn query_active_skills(
    conn: &rusqlite::Connection,
    character: &str,
    server: &str,
) -> serde_json::Value {
    let rows = query_table_as_json(conn, "game_state_active_skills", character, server);
    rows.into_iter().next().unwrap_or(serde_json::Value::Null)
}
