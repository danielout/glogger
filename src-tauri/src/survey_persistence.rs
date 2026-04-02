use crate::chat_status_parser::ChatStatusEvent;
use crate::db::DbPool;
use crate::parsers::to_utc_datetime;
/// Survey persistence — handles all survey DB writes synchronously from Rust.
///
/// This eliminates the race condition where the async frontend couldn't keep up
/// with rapidly-fired events from log parsing. The coordinator and legacy
/// emit_events path both call into this module to persist survey data.
use crate::survey_parser::SurveyEvent;
use rusqlite::Connection;

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
    /// The most recently used session ID — persists after auto-end so that
    /// late-arriving chat corrections can still find the right session.
    last_session_id: Option<i64>,
    /// How many regular (non-motherlode) maps have been started — these produce completions
    completable_maps: u32,
    /// How many surveys have been completed in this session
    surveys_completed: u32,
    /// Timezone offset in seconds from UTC for timestamp conversion
    timezone_offset_seconds: i32,
}

impl SurveySessionTracker {
    pub fn new() -> Self {
        Self {
            current_session_id: None,
            last_session_id: None,
            completable_maps: 0,
            surveys_completed: 0,
            timezone_offset_seconds: 0,
        }
    }

    /// Set the timezone offset (seconds from UTC) for timestamp conversion.
    pub fn set_timezone_offset(&mut self, offset_seconds: i32) {
        self.timezone_offset_seconds = offset_seconds;
    }

    /// Convert a Player.log HH:MM:SS timestamp to a full UTC datetime string.
    fn to_utc(&self, ts: &str) -> String {
        to_utc_datetime(ts, self.timezone_offset_seconds)
    }

    /// Process a survey event and persist it to the database.
    pub fn process_event(&mut self, event: &SurveyEvent, db: &DbPool) -> ProcessResult {
        let conn = match db.get() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[survey-persist] DB connection error: {e}");
                return ProcessResult {
                    session_id: None,
                    session_ended: false,
                };
            }
        };

        match event {
            SurveyEvent::MapCrafted {
                timestamp,
                map_name,
                internal_name,
                ingredients_consumed,
            } => {
                let dt = self.to_utc(timestamp);
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
                            self.last_session_id = Some(session_id);
                            self.completable_maps = if is_motherlode { 0 } else { 1 };
                            self.surveys_completed = 0;
                            eprintln!(
                                "[survey-persist] Created session {session_id} for {map_name}"
                            );
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
                let event_id = conn
                    .execute(
                        "INSERT INTO survey_events (
                        timestamp, session_id, event_type, map_type, survey_type, speed_bonus_earned
                    ) VALUES (?1, ?2, 'map_crafted', ?3, NULL, 0)",
                        rusqlite::params![dt, self.current_session_id, map_name],
                    )
                    .ok()
                    .map(|_| conn.last_insert_rowid());

                // Store consumed ingredients
                if let Some(eid) = event_id {
                    for ingredient in ingredients_consumed {
                        conn.execute(
                            "INSERT INTO survey_loot_items (
                                event_id, item_id, item_name, quantity, is_speed_bonus, is_primary
                            ) VALUES (?1, NULL, ?2, ?3, 0, 0)",
                            rusqlite::params![eid, ingredient.item_name, ingredient.quantity],
                        )
                        .ok();
                    }
                }

                ProcessResult {
                    session_id: self.current_session_id,
                    session_ended: false,
                }
            }

            SurveyEvent::SurveyUsed {
                timestamp,
                survey_name,
            } => {
                let dt = self.to_utc(timestamp);

                // Log survey_used event (informational only — no session creation or counting)
                if let Some(sid) = self.current_session_id {
                    conn.execute(
                        "INSERT INTO survey_events (
                            timestamp, session_id, event_type, map_type, survey_type, speed_bonus_earned
                        ) VALUES (?1, ?2, 'survey_used', NULL, ?3, 0)",
                        rusqlite::params![dt, sid, survey_name],
                    ).ok();
                }

                ProcessResult {
                    session_id: self.current_session_id,
                    session_ended: false,
                }
            }

            SurveyEvent::MotherlodeCompleted {
                timestamp,
                map_name,
                loot_items,
            } => {
                let dt = self.to_utc(timestamp);
                let session_id = self.current_session_id;

                // Log motherlode_completed event
                let event_id = match conn.execute(
                    "INSERT INTO survey_events (
                        timestamp, session_id, event_type, map_type, survey_type, speed_bonus_earned
                    ) VALUES (?1, ?2, 'motherlode_completed', ?3, ?3, 0)",
                    rusqlite::params![dt, session_id, map_name],
                ) {
                    Ok(_) => conn.last_insert_rowid(),
                    Err(e) => {
                        eprintln!("[survey-persist] Failed to log motherlode_completed event: {e}");
                        return ProcessResult {
                            session_id,
                            session_ended: false,
                        };
                    }
                };

                // Log each loot item
                for item in loot_items {
                    eprintln!(
                        "[survey-persist] Motherlode loot: {} x{} (event_id={}, session={})",
                        item.item_name,
                        item.quantity,
                        event_id,
                        session_id.unwrap_or(-1)
                    );
                    conn.execute(
                        "INSERT INTO survey_loot_items (
                            event_id, item_id, item_name, quantity, is_speed_bonus, is_primary
                        ) VALUES (?1, NULL, ?2, ?3, 0, 1)",
                        rusqlite::params![event_id, item.item_name, item.quantity],
                    )
                    .ok();
                }

                // Update end_time but do NOT increment surveys_completed
                // (motherlodes don't count toward auto-end)
                if let Some(sid) = session_id {
                    conn.execute(
                        "UPDATE survey_session_stats SET end_time = ?1 WHERE id = ?2",
                        rusqlite::params![dt, sid],
                    )
                    .ok();

                    // Always finalize after motherlode completion so economics stay
                    // up-to-date. Motherlode-only sessions never auto-end, so without
                    // this they'd show 0g revenue in the historical view.
                    finalize_session(&conn, sid);
                }

                ProcessResult {
                    session_id,
                    session_ended: false,
                }
            }

            SurveyEvent::Completed {
                timestamp,
                survey_name,
                loot_items,
                speed_bonus_earned,
            } => {
                let dt = self.to_utc(timestamp);
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
                        return ProcessResult {
                            session_id,
                            session_ended: false,
                        };
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
                    )
                    .ok();
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
                    )
                    .ok();

                    // Always finalize after completion so economics stay up-to-date.
                    // This ensures abandoned sessions (not manually ended) still have
                    // accurate revenue/cost/profit in the historical view.
                    finalize_session(&conn, sid);
                }

                // Auto-end: if completed all completable maps, session is done
                let mut session_ended = false;
                if self.completable_maps > 0 && self.surveys_completed >= self.completable_maps {
                    eprintln!(
                        "[survey-persist] Session auto-ended: {} completed >= {} completable maps",
                        self.surveys_completed, self.completable_maps
                    );

                    self.current_session_id = None;
                    self.completable_maps = 0;
                    self.surveys_completed = 0;
                    session_ended = true;
                }

                ProcessResult {
                    session_id,
                    session_ended,
                }
            }
        }
    }

    /// Correct loot item quantities using ChatStatusEvent::ItemGained from the chat log.
    ///
    /// Player.log's ProcessAddItem doesn't include stack size for new items, so we
    /// record quantity=1. The chat log's "[Status] Gypsum x9 added to inventory."
    /// gives us the real quantity. This method updates the most recent matching
    /// loot row in the current or last session.
    ///
    /// Uses `current_session_id` first, falling back to `last_session_id` so that
    /// corrections still work after a session auto-ends (e.g., the last regular
    /// survey completes before the motherlode's chat status message arrives).
    pub fn correct_loot_from_chat_status(
        &self,
        event: &ChatStatusEvent,
        db: &DbPool,
    ) -> Option<LootCorrection> {
        // Extract item info first, before checking session
        let (item_name, quantity) = match event {
            ChatStatusEvent::ItemGained {
                item_name,
                quantity,
                ..
            } => {
                if *quantity <= 1 {
                    return None; // No correction needed for quantity 1
                }
                (item_name.as_str(), *quantity)
            }
            _ => return None,
        };

        let session_id = match self.current_session_id.or(self.last_session_id) {
            Some(id) => id,
            None => return None,
        };

        let conn = db.get().ok()?;

        // Find the most recent loot row in this session where:
        // - item_name matches (display name from chat log)
        // - quantity < the chat log quantity (needs correction)
        // - event is a motherlode_completed or completed
        // Order by event_id DESC to get the most recent match.
        let result: Option<(i64, u32)> = conn
            .query_row(
                "SELECT sli.rowid, sli.quantity
             FROM survey_loot_items sli
             JOIN survey_events se ON sli.event_id = se.id
             WHERE se.session_id = ?1
               AND sli.item_name = ?2
               AND sli.quantity < ?3
               AND se.event_type IN ('motherlode_completed', 'completed')
             ORDER BY sli.rowid DESC
             LIMIT 1",
                rusqlite::params![session_id, item_name, quantity],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        if let Some((rowid, old_quantity)) = result {
            let delta = quantity - old_quantity;
            conn.execute(
                "UPDATE survey_loot_items SET quantity = ?1 WHERE rowid = ?2",
                rusqlite::params![quantity, rowid],
            )
            .ok();
            eprintln!(
                "[survey-persist] Corrected {} quantity: {} → {} (from chat status)",
                item_name, old_quantity, quantity
            );
            Some(LootCorrection {
                item_name: item_name.to_string(),
                old_quantity,
                new_quantity: quantity,
                delta,
            })
        } else {
            // Debug: why didn't we find a match?
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM survey_loot_items sli
                 JOIN survey_events se ON sli.event_id = se.id
                 WHERE se.session_id = ?1 AND sli.item_name = ?2",
                    rusqlite::params![session_id, item_name],
                    |row| row.get(0),
                )
                .unwrap_or(-1);
            if count > 0 {
                let existing_qty: u32 = conn
                    .query_row(
                        "SELECT sli.quantity FROM survey_loot_items sli
                     JOIN survey_events se ON sli.event_id = se.id
                     WHERE se.session_id = ?1 AND sli.item_name = ?2
                     ORDER BY sli.rowid DESC LIMIT 1",
                        rusqlite::params![session_id, item_name],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                eprintln!(
                    "[survey-persist] No correction for {} (chat qty={}, db has {} rows, latest qty={})",
                    item_name, quantity, count, existing_qty
                );
            }
            None
        }
    }

    /// Check if there's an active session
    pub fn has_active_session(&self) -> bool {
        self.current_session_id.is_some()
    }

    /// Get the current active session ID (for diagnostics)
    pub fn current_session_id(&self) -> Option<i64> {
        self.current_session_id
    }

    /// Get the last known session ID (for diagnostics)
    pub fn last_session_id(&self) -> Option<i64> {
        self.last_session_id
    }
}

/// A loot quantity correction from chat status cross-referencing
#[derive(Debug, Clone, serde::Serialize)]
pub struct LootCorrection {
    pub item_name: String,
    pub old_quantity: u32,
    pub new_quantity: u32,
    pub delta: u32,
}

/// Compute and store summary stats for a session.
/// Called on auto-end and re-called by patch_survey_session (manual end).
/// Revenue, cost, profit, speed bonus count, survey types, and maps used
/// are all computed from the session's events and loot items already in the DB.
///
/// Elapsed seconds and XP gains are NOT computed here — the frontend patches those in
/// via patch_survey_session since only it knows about pause durations and XP baselines.
pub fn finalize_session(conn: &Connection, session_id: i64) {
    // Revenue: sum of loot quantity * item vendor value (from completed + motherlode events)
    // JOIN on item_name since item_id is not populated during loot persistence
    let total_revenue: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(sli.quantity * COALESCE(i.value, 0)), 0)
         FROM survey_events se
         JOIN survey_loot_items sli ON sli.event_id = se.id
         LEFT JOIN items i ON sli.item_name = i.name COLLATE NOCASE
         WHERE se.session_id = ?1 AND se.event_type IN ('completed', 'motherlode_completed')",
            [session_id],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    // Cost: sum of survey map crafting costs for all maps crafted
    let total_cost: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(COALESCE(st.crafting_cost, 0)), 0)
         FROM survey_events se
         LEFT JOIN survey_types st ON se.map_type = st.name COLLATE NOCASE
         WHERE se.session_id = ?1 AND se.event_type = 'map_crafted'",
            [session_id],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let total_profit = total_revenue - total_cost;

    // Elapsed seconds from timestamps
    let elapsed_seconds: i64 = conn
        .query_row(
            "SELECT COALESCE(
            CAST((julianday(end_time) - julianday(start_time)) * 86400 AS INTEGER),
            0
         ) FROM survey_session_stats WHERE id = ?1",
            [session_id],
            |row| row.get(0),
        )
        .unwrap_or(0)
        .max(1);

    let hours = elapsed_seconds as f64 / 3600.0;
    let profit_per_hour = if hours > 0.0 {
        total_profit / hours
    } else {
        0.0
    };

    // Speed bonus count
    let speed_bonus_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM survey_events
         WHERE session_id = ?1 AND event_type = 'completed' AND speed_bonus_earned = 1",
            [session_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Survey types used (distinct completed/motherlode survey names)
    let survey_types_used: String = {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT survey_type FROM survey_events
             WHERE session_id = ?1 AND event_type IN ('completed', 'motherlode_completed') AND survey_type IS NOT NULL
             ORDER BY survey_type"
        ).unwrap();
        let types: Vec<String> = stmt
            .query_map([session_id], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        types.join(", ")
    };

    // Maps used summary (map name with count)
    let maps_used_summary: String = {
        let mut stmt = conn
            .prepare(
                "SELECT map_type, COUNT(*) as cnt FROM survey_events
             WHERE session_id = ?1 AND event_type = 'map_crafted' AND map_type IS NOT NULL
             GROUP BY map_type ORDER BY cnt DESC",
            )
            .unwrap();
        let maps: Vec<String> = stmt
            .query_map([session_id], |row| {
                let name: String = row.get(0)?;
                let cnt: i64 = row.get(1)?;
                Ok(if cnt > 1 {
                    format!("{name} x{cnt}")
                } else {
                    name
                })
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
    )
    .ok();

    eprintln!(
        "[survey-persist] Finalized session {session_id}: revenue={total_revenue}, cost={total_cost}, profit={total_profit}, profit/hr={profit_per_hour:.0}, bonuses={speed_bonus_count}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use r2d2_sqlite::SqliteConnectionManager;

    /// Create an in-memory DB pool with the survey tables needed for testing.
    fn test_db() -> DbPool {
        let manager = SqliteConnectionManager::memory();
        let pool = r2d2::Pool::builder().max_size(1).build(manager).unwrap();
        let conn = pool.get().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE survey_session_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL DEFAULT 'Survey Session',
                start_time TIMESTAMP NOT NULL,
                end_time TIMESTAMP,
                maps_started INTEGER NOT NULL DEFAULT 0,
                surveys_completed INTEGER NOT NULL DEFAULT 0,
                surveying_xp_gained INTEGER NOT NULL DEFAULT 0,
                mining_xp_gained INTEGER NOT NULL DEFAULT 0,
                geology_xp_gained INTEGER NOT NULL DEFAULT 0,
                total_revenue INTEGER NOT NULL DEFAULT 0,
                total_cost INTEGER NOT NULL DEFAULT 0,
                total_profit INTEGER NOT NULL DEFAULT 0,
                profit_per_hour INTEGER NOT NULL DEFAULT 0,
                elapsed_seconds INTEGER NOT NULL DEFAULT 0,
                is_manual BOOLEAN DEFAULT 0,
                speed_bonus_count INTEGER NOT NULL DEFAULT 0,
                survey_types_used TEXT,
                maps_used_summary TEXT,
                notes TEXT NOT NULL DEFAULT '',
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE survey_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TIMESTAMP NOT NULL,
                session_id INTEGER,
                event_type TEXT NOT NULL,
                map_type TEXT,
                survey_type TEXT,
                speed_bonus_earned BOOLEAN DEFAULT 0,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE survey_loot_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id INTEGER NOT NULL,
                item_id INTEGER,
                item_name TEXT NOT NULL,
                quantity INTEGER NOT NULL DEFAULT 1,
                is_speed_bonus BOOLEAN NOT NULL DEFAULT 0,
                is_primary BOOLEAN NOT NULL DEFAULT 0,
                obtained_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
        ",
        )
        .unwrap();
        pool
    }

    /// Set up a tracker with an active session and a motherlode loot row.
    /// Returns (tracker, db, session_id, event_id).
    fn setup_session_with_loot(
        item_name: &str,
        quantity: u32,
    ) -> (SurveySessionTracker, DbPool, i64, i64) {
        let db = test_db();
        let conn = db.get().unwrap();

        // Create session
        conn.execute(
            "INSERT INTO survey_session_stats (start_time) VALUES ('2026-03-27 15:00:00')",
            [],
        )
        .unwrap();
        let session_id = conn.last_insert_rowid();

        // Create motherlode_completed event
        conn.execute(
            "INSERT INTO survey_events (timestamp, session_id, event_type, map_type) \
             VALUES ('2026-03-27 15:01:00', ?1, 'motherlode_completed', 'Test Motherlode Map')",
            [session_id],
        )
        .unwrap();
        let event_id = conn.last_insert_rowid();

        // Create loot row with initial quantity (typically 1 from Player.log)
        conn.execute(
            "INSERT INTO survey_loot_items (event_id, item_name, quantity, is_primary) \
             VALUES (?1, ?2, ?3, 1)",
            rusqlite::params![event_id, item_name, quantity],
        )
        .unwrap();

        let mut tracker = SurveySessionTracker::new();
        tracker.current_session_id = Some(session_id);
        tracker.last_session_id = Some(session_id);

        (tracker, db, session_id, event_id)
    }

    #[test]
    fn test_correct_loot_quantity_from_chat_status() {
        let (tracker, db, _, _) = setup_session_with_loot("Gypsum", 1);

        let event = ChatStatusEvent::ItemGained {
            timestamp: "2026-03-27 15:01:01".to_string(),
            item_name: "Gypsum".to_string(),
            quantity: 9,
        };

        let correction = tracker.correct_loot_from_chat_status(&event, &db);
        assert!(correction.is_some(), "Should produce a correction");

        let c = correction.unwrap();
        assert_eq!(c.item_name, "Gypsum");
        assert_eq!(c.old_quantity, 1);
        assert_eq!(c.new_quantity, 9);
        assert_eq!(c.delta, 8);

        // Verify DB was updated
        let conn = db.get().unwrap();
        let db_qty: u32 = conn
            .query_row(
                "SELECT quantity FROM survey_loot_items WHERE item_name = 'Gypsum'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(db_qty, 9);
    }

    #[test]
    fn test_no_correction_for_quantity_1() {
        let (tracker, db, _, _) = setup_session_with_loot("Gypsum", 1);

        let event = ChatStatusEvent::ItemGained {
            timestamp: "2026-03-27 15:01:01".to_string(),
            item_name: "Gypsum".to_string(),
            quantity: 1,
        };

        let correction = tracker.correct_loot_from_chat_status(&event, &db);
        assert!(
            correction.is_none(),
            "quantity=1 should not trigger correction"
        );
    }

    #[test]
    fn test_no_correction_when_no_active_session() {
        let db = test_db();
        let tracker = SurveySessionTracker::new(); // no session

        let event = ChatStatusEvent::ItemGained {
            timestamp: "2026-03-27 15:01:01".to_string(),
            item_name: "Gypsum".to_string(),
            quantity: 9,
        };

        let correction = tracker.correct_loot_from_chat_status(&event, &db);
        assert!(correction.is_none(), "No session = no correction");
    }

    #[test]
    fn test_no_correction_for_non_matching_item() {
        let (tracker, db, _, _) = setup_session_with_loot("Gypsum", 1);

        let event = ChatStatusEvent::ItemGained {
            timestamp: "2026-03-27 15:01:01".to_string(),
            item_name: "Iron Ore".to_string(),
            quantity: 5,
        };

        let correction = tracker.correct_loot_from_chat_status(&event, &db);
        assert!(correction.is_none(), "Non-matching item should not correct");
    }

    #[test]
    fn test_no_correction_when_db_quantity_already_correct() {
        // DB already has quantity=9, chat says 9 — no correction needed
        let (tracker, db, _, _) = setup_session_with_loot("Gypsum", 9);

        let event = ChatStatusEvent::ItemGained {
            timestamp: "2026-03-27 15:01:01".to_string(),
            item_name: "Gypsum".to_string(),
            quantity: 9,
        };

        let correction = tracker.correct_loot_from_chat_status(&event, &db);
        assert!(correction.is_none(), "Already correct = no correction");
    }

    #[test]
    fn test_no_correction_for_non_item_event() {
        let (tracker, db, _, _) = setup_session_with_loot("Gypsum", 1);

        let event = ChatStatusEvent::XpGained {
            timestamp: "2026-03-27 15:01:01".to_string(),
            skill: "Mining".to_string(),
            amount: 130,
        };

        let correction = tracker.correct_loot_from_chat_status(&event, &db);
        assert!(
            correction.is_none(),
            "Non-ItemGained events should be ignored"
        );
    }

    #[test]
    fn test_corrects_most_recent_matching_row() {
        let db = test_db();
        let session_id;

        // Scope the setup connection so it's dropped before correct_loot_from_chat_status
        {
            let conn = db.get().unwrap();

            conn.execute(
                "INSERT INTO survey_session_stats (start_time) VALUES ('2026-03-27 15:00:00')",
                [],
            )
            .unwrap();
            session_id = conn.last_insert_rowid();

            // First motherlode
            conn.execute(
                "INSERT INTO survey_events (timestamp, session_id, event_type) \
                 VALUES ('2026-03-27 15:01:00', ?1, 'motherlode_completed')",
                [session_id],
            )
            .unwrap();
            let event1_id = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO survey_loot_items (event_id, item_name, quantity, is_primary) \
                 VALUES (?1, 'Gypsum', 1, 1)",
                [event1_id],
            )
            .unwrap();

            // Second motherlode (more recent)
            conn.execute(
                "INSERT INTO survey_events (timestamp, session_id, event_type) \
                 VALUES ('2026-03-27 15:05:00', ?1, 'motherlode_completed')",
                [session_id],
            )
            .unwrap();
            let event2_id = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO survey_loot_items (event_id, item_name, quantity, is_primary) \
                 VALUES (?1, 'Gypsum', 1, 1)",
                [event2_id],
            )
            .unwrap();
        }

        let mut tracker = SurveySessionTracker::new();
        tracker.current_session_id = Some(session_id);

        let event = ChatStatusEvent::ItemGained {
            timestamp: "2026-03-27 15:05:01".to_string(),
            item_name: "Gypsum".to_string(),
            quantity: 9,
        };

        let correction = tracker.correct_loot_from_chat_status(&event, &db);
        assert!(correction.is_some());

        // Only the most recent (second) row should be updated
        let conn = db.get().unwrap();
        let rows: Vec<(i64, u32)> = {
            let mut stmt = conn.prepare(
                "SELECT event_id, quantity FROM survey_loot_items WHERE item_name = 'Gypsum' ORDER BY id"
            ).unwrap();
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        };
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].1, 1, "First row should be unchanged");
        assert_eq!(rows[1].1, 9, "Second (most recent) row should be corrected");
    }

    #[test]
    fn test_correction_works_for_completed_events_too() {
        let db = test_db();
        let session_id;

        {
            let conn = db.get().unwrap();

            conn.execute(
                "INSERT INTO survey_session_stats (start_time) VALUES ('2026-03-27 15:00:00')",
                [],
            )
            .unwrap();
            session_id = conn.last_insert_rowid();

            // Regular survey completed event (not motherlode)
            conn.execute(
                "INSERT INTO survey_events (timestamp, session_id, event_type) \
                 VALUES ('2026-03-27 15:01:00', ?1, 'completed')",
                [session_id],
            )
            .unwrap();
            let event_id = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO survey_loot_items (event_id, item_name, quantity, is_primary) \
                 VALUES (?1, 'Metal Slab', 1, 1)",
                [event_id],
            )
            .unwrap();
        }

        let mut tracker = SurveySessionTracker::new();
        tracker.current_session_id = Some(session_id);

        let event = ChatStatusEvent::ItemGained {
            timestamp: "2026-03-27 15:01:01".to_string(),
            item_name: "Metal Slab".to_string(),
            quantity: 26,
        };

        let correction = tracker.correct_loot_from_chat_status(&event, &db);
        assert!(correction.is_some());
        let c = correction.unwrap();
        assert_eq!(c.new_quantity, 26);
        assert_eq!(c.delta, 25);
    }

    #[test]
    fn test_correction_works_after_session_auto_end() {
        // Simulates: session auto-ended (current_session_id = None) but
        // last_session_id still points to the ended session. Chat corrections
        // arriving after auto-end should still find the loot via last_session_id.
        let db = test_db();
        let session_id;

        {
            let conn = db.get().unwrap();

            conn.execute(
                "INSERT INTO survey_session_stats (start_time) VALUES ('2026-03-27 15:00:00')",
                [],
            )
            .unwrap();
            session_id = conn.last_insert_rowid();

            conn.execute(
                "INSERT INTO survey_events (timestamp, session_id, event_type) \
                 VALUES ('2026-03-27 15:01:00', ?1, 'motherlode_completed')",
                [session_id],
            )
            .unwrap();
            let event_id = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO survey_loot_items (event_id, item_name, quantity, is_primary) \
                 VALUES (?1, 'Gypsum', 1, 1)",
                [event_id],
            )
            .unwrap();
        }

        let mut tracker = SurveySessionTracker::new();
        // Session has auto-ended: current is None, but last still set
        tracker.current_session_id = None;
        tracker.last_session_id = Some(session_id);

        let event = ChatStatusEvent::ItemGained {
            timestamp: "2026-03-27 15:01:02".to_string(),
            item_name: "Gypsum".to_string(),
            quantity: 9,
        };

        let correction = tracker.correct_loot_from_chat_status(&event, &db);
        assert!(
            correction.is_some(),
            "Should correct via last_session_id after auto-end"
        );

        let c = correction.unwrap();
        assert_eq!(c.item_name, "Gypsum");
        assert_eq!(c.old_quantity, 1);
        assert_eq!(c.new_quantity, 9);
    }
}
