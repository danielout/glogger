/// Survey persistence — handles all survey DB writes synchronously from Rust.
///
/// This eliminates the race condition where the async frontend couldn't keep up
/// with rapidly-fired events from log parsing. The coordinator and legacy
/// emit_events path both call into this module to persist survey data.

use crate::survey_parser::SurveyEvent;
use crate::db::DbPool;
use chrono::Local;
use rusqlite::Connection;

/// Convert a log timestamp like "16:17:47" to a full ISO datetime using today's date
fn to_datetime(ts: &str) -> String {
    let today = Local::now().format("%Y-%m-%d");
    format!("{today} {ts}")
}

/// Result of processing a survey event — tells the caller what happened
pub struct ProcessResult {
    /// The session_id that was used (if any)
    pub session_id: Option<i64>,
    /// Whether this event caused the session to auto-end
    pub session_ended: bool,
}

/// Tracks an active survey session for DB persistence.
/// One instance lives in the coordinator and one in the legacy commands path.
pub struct SurveySessionTracker {
    /// The DB session_stats row ID for the current session (if any)
    current_session_id: Option<i64>,
    /// How many regular (non-motherlode) maps have been started — these produce completions
    completable_maps: u32,
    /// How many surveys have been completed in this session
    surveys_completed: u32,
}

impl SurveySessionTracker {
    pub fn new() -> Self {
        Self {
            current_session_id: None,
            completable_maps: 0,
            surveys_completed: 0,
        }
    }

    /// Process a survey event and persist it to the database.
    pub fn process_event(&mut self, event: &SurveyEvent, db: &DbPool) -> ProcessResult {
        let conn = match db.get() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[survey-persist] DB connection error: {e}");
                return ProcessResult { session_id: None, session_ended: false };
            }
        };

        match event {
            SurveyEvent::MapCrafted { timestamp, map_name, internal_name, ingredients_consumed } => {
                let dt = to_datetime(timestamp);
                let is_motherlode = map_name.ends_with("Map");

                // If no active session, create one
                if self.current_session_id.is_none() {
                    match conn.execute(
                        "INSERT INTO survey_session_stats (
                            start_time, maps_started, surveys_completed,
                            surveying_xp_gained, mining_xp_gained, geology_xp_gained,
                            total_revenue, total_cost, total_profit, profit_per_hour,
                            elapsed_seconds, is_manual
                        ) VALUES (?1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)",
                        rusqlite::params![dt],
                    ) {
                        Ok(_) => {
                            let session_id = conn.last_insert_rowid();
                            self.current_session_id = Some(session_id);
                            self.completable_maps = if is_motherlode { 0 } else { 1 };
                            self.surveys_completed = 0;
                            eprintln!("[survey-persist] Created session {session_id} for {map_name}");
                        }
                        Err(e) => {
                            eprintln!("[survey-persist] Failed to create session: {e}");
                        }
                    }
                } else {
                    // Session already active — increment maps_started in DB
                    if let Some(sid) = self.current_session_id {
                        conn.execute(
                            "UPDATE survey_session_stats SET maps_started = maps_started + 1 WHERE id = ?1",
                            rusqlite::params![sid],
                        ).ok();
                    }
                    // Only count non-motherlode maps for auto-end
                    if !is_motherlode {
                        self.completable_maps += 1;
                    }
                }

                // Log map_crafted event
                let event_id = conn.execute(
                    "INSERT INTO survey_events (
                        timestamp, session_id, event_type, map_type, survey_type, speed_bonus_earned
                    ) VALUES (?1, ?2, 'map_crafted', ?3, NULL, 0)",
                    rusqlite::params![dt, self.current_session_id, map_name],
                ).ok().map(|_| conn.last_insert_rowid());

                // Store consumed ingredients
                if let Some(eid) = event_id {
                    for ingredient in ingredients_consumed {
                        conn.execute(
                            "INSERT INTO survey_loot_items (
                                event_id, item_id, item_name, quantity, is_speed_bonus, is_primary
                            ) VALUES (?1, NULL, ?2, ?3, 0, 0)",
                            rusqlite::params![eid, ingredient.item_name, ingredient.quantity],
                        ).ok();
                    }
                }

                ProcessResult { session_id: self.current_session_id, session_ended: false }
            }

            SurveyEvent::SurveyUsed { timestamp, survey_name } => {
                let dt = to_datetime(timestamp);

                // Log survey_used event (informational only — no session creation or counting)
                if let Some(sid) = self.current_session_id {
                    conn.execute(
                        "INSERT INTO survey_events (
                            timestamp, session_id, event_type, map_type, survey_type, speed_bonus_earned
                        ) VALUES (?1, ?2, 'survey_used', NULL, ?3, 0)",
                        rusqlite::params![dt, sid, survey_name],
                    ).ok();
                }

                ProcessResult { session_id: self.current_session_id, session_ended: false }
            }

            SurveyEvent::Completed { timestamp, survey_name, loot_items, speed_bonus_earned } => {
                let dt = to_datetime(timestamp);
                let session_id = self.current_session_id;

                // Log completed event
                let event_id = match conn.execute(
                    "INSERT INTO survey_events (
                        timestamp, session_id, event_type, map_type, survey_type, speed_bonus_earned
                    ) VALUES (?1, ?2, 'completed', ?3, ?3, ?4)",
                    rusqlite::params![dt, session_id, survey_name, speed_bonus_earned],
                ) {
                    Ok(_) => conn.last_insert_rowid(),
                    Err(e) => {
                        eprintln!("[survey-persist] Failed to log completed event: {e}");
                        return ProcessResult { session_id, session_ended: false };
                    }
                };

                // Log each loot item
                for item in loot_items {
                    conn.execute(
                        "INSERT INTO survey_loot_items (
                            event_id, item_id, item_name, quantity, is_speed_bonus, is_primary
                        ) VALUES (?1, NULL, ?2, ?3, ?4, ?5)",
                        rusqlite::params![
                            event_id,
                            item.item_name,
                            item.quantity,
                            item.is_speed_bonus,
                            item.is_primary,
                        ],
                    ).ok();
                }

                // Update session stats
                self.surveys_completed += 1;
                if let Some(sid) = session_id {
                    conn.execute(
                        "UPDATE survey_session_stats SET
                            surveys_completed = ?1,
                            end_time = ?2
                         WHERE id = ?3",
                        rusqlite::params![self.surveys_completed, dt, sid],
                    ).ok();
                }

                // Auto-end: if completed all completable maps, session is done
                let mut session_ended = false;
                if self.completable_maps > 0 && self.surveys_completed >= self.completable_maps {
                    eprintln!(
                        "[survey-persist] Session auto-ended: {} completed >= {} completable maps",
                        self.surveys_completed, self.completable_maps
                    );

                    // Finalize: compute and store summary stats before clearing
                    if let Some(sid) = session_id {
                        finalize_session(&conn, sid);
                    }

                    self.current_session_id = None;
                    self.completable_maps = 0;
                    self.surveys_completed = 0;
                    session_ended = true;
                }

                ProcessResult { session_id, session_ended }
            }
        }
    }
}

/// Compute and store summary stats for a completed session.
/// Called on auto-end. Revenue, cost, profit, speed bonus count, survey types, and maps used
/// are all computed from the session's events and loot items already in the DB.
///
/// Elapsed seconds and XP gains are NOT computed here — the frontend patches those in
/// via update_survey_session_stats since only it knows about pause durations and XP baselines.
fn finalize_session(conn: &Connection, session_id: i64) {
    // Revenue: sum of loot quantity * item vendor value (only from completed events)
    let total_revenue: f64 = conn.query_row(
        "SELECT COALESCE(SUM(sli.quantity * COALESCE(i.value, 0)), 0)
         FROM survey_events se
         JOIN survey_loot_items sli ON sli.event_id = se.id
         LEFT JOIN items i ON sli.item_id = i.id
         WHERE se.session_id = ?1 AND se.event_type = 'completed'",
        [session_id],
        |row| row.get(0),
    ).unwrap_or(0.0);

    // Cost: sum of survey map crafting costs for all maps crafted
    let total_cost: f64 = conn.query_row(
        "SELECT COALESCE(SUM(COALESCE(st.crafting_cost, 0)), 0)
         FROM survey_events se
         LEFT JOIN survey_types st ON se.map_type = st.name COLLATE NOCASE
         WHERE se.session_id = ?1 AND se.event_type = 'map_crafted'",
        [session_id],
        |row| row.get(0),
    ).unwrap_or(0.0);

    let total_profit = total_revenue - total_cost;

    // Elapsed seconds from timestamps
    let elapsed_seconds: i64 = conn.query_row(
        "SELECT COALESCE(
            CAST((julianday(end_time) - julianday(start_time)) * 86400 AS INTEGER),
            0
         ) FROM survey_session_stats WHERE id = ?1",
        [session_id],
        |row| row.get(0),
    ).unwrap_or(0).max(1);

    let hours = elapsed_seconds as f64 / 3600.0;
    let profit_per_hour = if hours > 0.0 { total_profit / hours } else { 0.0 };

    // Speed bonus count
    let speed_bonus_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM survey_events
         WHERE session_id = ?1 AND event_type = 'completed' AND speed_bonus_earned = 1",
        [session_id],
        |row| row.get(0),
    ).unwrap_or(0);

    // Survey types used (distinct completed survey names)
    let survey_types_used: String = {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT survey_type FROM survey_events
             WHERE session_id = ?1 AND event_type = 'completed' AND survey_type IS NOT NULL
             ORDER BY survey_type"
        ).unwrap();
        let types: Vec<String> = stmt.query_map([session_id], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        types.join(", ")
    };

    // Maps used summary (map name with count)
    let maps_used_summary: String = {
        let mut stmt = conn.prepare(
            "SELECT map_type, COUNT(*) as cnt FROM survey_events
             WHERE session_id = ?1 AND event_type = 'map_crafted' AND map_type IS NOT NULL
             GROUP BY map_type ORDER BY cnt DESC"
        ).unwrap();
        let maps: Vec<String> = stmt.query_map([session_id], |row| {
            let name: String = row.get(0)?;
            let cnt: i64 = row.get(1)?;
            Ok(if cnt > 1 { format!("{name} x{cnt}") } else { name })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
        maps.join(", ")
    };

    // Write it all
    conn.execute(
        "UPDATE survey_session_stats SET
            total_revenue = ?1,
            total_cost = ?2,
            total_profit = ?3,
            profit_per_hour = ?4,
            elapsed_seconds = ?5,
            speed_bonus_count = ?6,
            survey_types_used = ?7,
            maps_used_summary = ?8
         WHERE id = ?9",
        rusqlite::params![
            total_revenue,
            total_cost,
            total_profit,
            profit_per_hour,
            elapsed_seconds,
            speed_bonus_count,
            survey_types_used,
            maps_used_summary,
            session_id,
        ],
    ).ok();

    eprintln!(
        "[survey-persist] Finalized session {session_id}: revenue={total_revenue}, cost={total_cost}, profit={total_profit}, profit/hr={profit_per_hour:.0}, bonuses={speed_bonus_count}"
    );
}
