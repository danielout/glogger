//! Debug capture system for recording raw log lines and game state snapshots.
//!
//! When a capture is active, raw Player.log and Chat.log lines are written to
//! temporary files on disk to avoid unbounded memory growth during long sessions.
//! Game state is snapshotted at start and stop. The whole bundle is assembled
//! into a single JSON file for debugging and support purposes.

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
    pub started_at: Option<String>,
    pub line_count: usize,
    pub player_line_count: usize,
    pub chat_line_count: usize,
}

/// Result returned after stopping and saving a capture.
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
    started_at: Option<String>,
    player_line_count: usize,
    chat_line_count: usize,
    state_at_start: Option<GameStateSnapshot>,
    /// Temp file for captured lines. Created on start, deleted after export.
    temp_path: Option<PathBuf>,
    temp_writer: Option<BufWriter<fs::File>>,
}

impl DebugCaptureState {
    pub fn new() -> Self {
        Self {
            active: false,
            started_at: None,
            player_line_count: 0,
            chat_line_count: 0,
            state_at_start: None,
            temp_path: None,
            temp_writer: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn status(&self) -> DebugCaptureStatus {
        DebugCaptureStatus {
            active: self.active,
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
        self.started_at = Some(Utc::now().to_rfc3339());
        self.player_line_count = 0;
        self.chat_line_count = 0;
        self.state_at_start = Some(snapshot);
        self.temp_writer = Some(BufWriter::new(file));
        self.temp_path = Some(temp_path);

        Ok(())
    }

    /// Stop the capture and write the full bundle JSON to `output_path`.
    /// Returns stats about the capture.
    pub fn stop(
        &mut self,
        snapshot: GameStateSnapshot,
        notes: String,
        app_version: String,
        output_path: &str,
    ) -> Result<DebugCaptureResult, String> {
        if !self.active {
            return Err("No debug capture is active".to_string());
        }

        self.active = false;

        // Flush and close the temp writer
        if let Some(mut writer) = self.temp_writer.take() {
            writer
                .flush()
                .map_err(|e| format!("Failed to flush capture temp file: {e}"))?;
        }

        let started_at = self.started_at.take().unwrap_or_default();
        let state_at_start = self
            .state_at_start
            .take()
            .ok_or_else(|| "Missing start snapshot".to_string())?;

        let result = DebugCaptureResult {
            line_count: self.player_line_count + self.chat_line_count,
            player_line_count: self.player_line_count,
            chat_line_count: self.chat_line_count,
        };

        // Read captured lines from temp file
        let lines = self.read_temp_lines();

        // Build the output JSON and write directly to the destination
        let bundle = serde_json::json!({
            "format_version": 1,
            "app_version": app_version,
            "captured_by": "glogger debug capture",
            "started_at": started_at,
            "stopped_at": Utc::now().to_rfc3339(),
            "notes": notes,
            "state_at_start": state_at_start,
            "state_at_stop": snapshot,
            "line_count": result.line_count,
            "player_line_count": result.player_line_count,
            "chat_line_count": result.chat_line_count,
            "lines": lines,
        });

        let out_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {e}"))?;
        let writer = BufWriter::new(out_file);
        serde_json::to_writer_pretty(writer, &bundle)
            .map_err(|e| format!("Failed to write capture JSON: {e}"))?;

        // Clean up temp file
        self.cleanup_temp();
        self.player_line_count = 0;
        self.chat_line_count = 0;

        Ok(result)
    }

    /// Discard the active capture without saving.
    pub fn discard(&mut self) {
        self.active = false;
        self.started_at = None;
        self.state_at_start = None;
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
    fn read_temp_lines(&self) -> Vec<CapturedLine> {
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
