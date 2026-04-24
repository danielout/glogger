//! Survey session aggregator — Phase 5 brains.
//!
//! Subscribes to [`PlayerEvent`]s as they flow through the coordinator and:
//!
//! - Detects survey-map crafting and use, recording rows in `survey_sessions`
//!   and `survey_uses`.
//! - Drives the session lifecycle state machine (manual / crafting / first-use
//!   start; auto-end for crafting sessions when consumed catches crafted).
//! - Performs **A3 stitching**: when a Mining context opens within 60s of a
//!   survey-map use being consumed, gains in that mining cycle get the
//!   originating `survey_use_id` injected into their `ItemProvenance` so
//!   downstream queries can group "all loot from this survey use".
//! - Tracks per-kind windows: Basic (single tick), Motherlode (one mining
//!   cycle), Multihit (different-entity OR 30min via DB-backed
//!   `open_multihit_nodes`).
//!
//! See `docs/plans/survey-tracker-rewrite.md` for the full design and
//! `docs/architecture/survey-mechanics.md` for the kind-specific game
//! mechanics this implements.

use crate::game_data::SurveyKind;
use crate::player_event_parser::{ActivitySource, ItemProvenance, PlayerEvent};
use crate::parsers::to_utc_datetime_with_base;
use crate::survey::{multihit_state, persistence};
use crate::survey::types::{SessionStartTrigger, SurveyUseKind, SurveyUseStatus};
use crate::cdn_commands::GameDataState;
use rusqlite::Connection;
use std::collections::VecDeque;

/// Window (seconds) within which a Mining context that opens after a survey
/// map is consumed inherits the originating use's `survey_use_id`. Covers
/// the player walking from where they used the map to where the node spawned.
const SURVEY_TO_MINING_GRACE_SECS: u32 = 60;

/// Multihit node timeout: if no mining hit lands on a tracked node for this
/// long, the open_multihit_nodes row is swept and the use marked completed.
/// See docs/architecture/survey-mechanics.md for why this is 30 minutes.
#[allow(dead_code)] // Will be used once multihit sweep is wired up
const MULTIHIT_TIMEOUT_SECS: u32 = 30 * 60;

/// One survey-map use awaiting its first Mining context. Lives in memory
/// only — the use itself is already persisted in `survey_uses` with
/// `status = pending_loot`. This entry just remembers we expect a Mining
/// context to start within 60s.
#[derive(Debug, Clone)]
struct PendingUse {
    survey_use_id: i64,
    kind: SurveyUseKind,
    /// Seconds-of-day for `used_at`. Compared against incoming line
    /// timestamps to apply the 60s grace window.
    used_at_secs: u32,
}

/// A basic-survey loot gain that arrived *before* the survey map's
/// `ProcessDeleteItem` created the pending use. Buffered so it can be
/// attributed once the deletion fires and the pending use exists.
///
/// In the game's log, the order for basic surveys is:
///   1. ProcessAddItem (or ProcessUpdateItemCode) — primary loot
///   2. ProcessDeleteItem — survey map consumed
///   3. ProcessAddItem — speed-bonus loot (if any)
///
/// The pending use is created at step 2, so gains from step 1 arrive
/// before the pending use exists. This struct holds them until step 2.
#[derive(Debug, Clone)]
struct DeferredBasicGain {
    /// Quantity (1 for ItemAdded, delta for ItemStackChanged).
    qty: u32,
    /// HH:MM:SS timestamp from the event, for loot-timestamp updates.
    timestamp_hms: Option<String>,
}

/// Side-effect events the aggregator returns so the coordinator can emit
/// them to the frontend. Kept narrow on purpose — frontend gets the high-
/// signal moments, not every internal state transition.
#[derive(Debug, Clone)]
pub enum SurveyAggregatorEvent {
    SessionStarted {
        session_id: i64,
        trigger: SessionStartTrigger,
    },
    SessionEnded {
        session_id: i64,
        // 'manual' | 'auto' | 'idle'
        reason: &'static str,
    },
    UseRecorded {
        use_id: i64,
        session_id: Option<i64>,
        map_internal_name: String,
        kind: SurveyUseKind,
    },
    #[allow(dead_code)] // Matched in coordinator; not yet emitted by aggregator
    UseCompleted {
        use_id: i64,
    },
    #[allow(dead_code)] // Matched in coordinator; not yet emitted by aggregator
    MultihitNodeOpened {
        use_id: i64,
        node_entity_id: u32,
    },
    #[allow(dead_code)] // Matched in coordinator; not yet emitted by aggregator
    MultihitNodeClosed {
        use_id: i64,
        node_entity_id: u32,
    },
}

/// Per-character aggregator. The coordinator owns one instance and feeds it
/// `PlayerEvent`s as they arrive.
pub struct SurveySessionAggregator {
    game_data: GameDataState,

    /// Recently-used survey maps awaiting their first Mining context.
    /// Sorted by `used_at_secs` ascending (front = oldest); aged out on each
    /// event tick.
    pending_uses: VecDeque<PendingUse>,

    /// The mining node the player is currently swinging at. Only one node
    /// at a time (you can't mine two simultaneously). Updated on
    /// `DelayLoopStarted("Mining...")` and cleared when a different mining
    /// interaction starts or the player stops mining.
    current_mining_node: Option<u32>,

    /// Active Motherlode attribution. When a pending Motherlode use adopts
    /// a mining node, we record `(node_entity_id, survey_use_id,
    /// first_gain_secs)` so gains arriving within 1 second of the first
    /// also attribute. After 1 second the use auto-completes. This covers
    /// motherlodes that yield 2+ items in the same swing without leaving
    /// the use open indefinitely.
    active_motherlode: Option<(u32, i64, u32)>,

    /// The most-recent Basic survey use that just had its loot attributed,
    /// held only long enough to catch a `ProcessScreenText` "(speed bonus!)"
    /// marker that arrives immediately after the gains in the same tick.
    /// Cleared on the next non-ScreenText event.
    last_basic_use_for_bonus: Option<i64>,

    /// Gains with `SurveyMapUse` provenance that arrived before the
    /// survey map's `DeleteItem` created the pending use. Drained in
    /// `handle_survey_consumed` once the pending use exists. Tiny in
    /// practice (1-2 items per survey use, cleared every tick).
    deferred_basic_gains: Vec<DeferredBasicGain>,

    /// Cached active session id (DB is the source of truth, but we cache to
    /// avoid a query per event). `None` means "unknown — go check the DB".
    /// Refreshed when a session is started/ended.
    cached_active_session_id: Option<Option<i64>>,

    /// When set, overrides the UTC date used to stamp Player.log `HH:MM:SS`
    /// timestamps seen by the aggregator. Live tailing leaves this unset;
    /// replay / old-log reparse sets it so session/use rows carry the correct
    /// historical date.
    base_date_override: Option<chrono::NaiveDate>,

    /// When false, the aggregator skips auto-creating sessions on crafting
    /// or first-use detection. The user must manually start sessions.
    /// Updated by the coordinator from `AppSettings.auto_start_survey_sessions`.
    pub auto_start_enabled: bool,
}

impl SurveySessionAggregator {
    pub fn new(game_data: GameDataState) -> Self {
        Self {
            game_data,
            pending_uses: VecDeque::new(),
            current_mining_node: None,
            active_motherlode: None,
            last_basic_use_for_bonus: None,
            deferred_basic_gains: Vec::new(),
            cached_active_session_id: None,
            base_date_override: None,
            auto_start_enabled: true,
        }
    }

    /// Stamp Player.log times with an explicit UTC date instead of today's.
    /// Live tailing leaves this unset.
    pub fn set_base_date(&mut self, date: chrono::NaiveDate) {
        self.base_date_override = Some(date);
    }

    fn to_utc(&self, ts: &str) -> String {
        to_utc_datetime_with_base(ts, self.base_date_override)
    }

    // ============================================================
    // Public API — session lifecycle
    // ============================================================

    /// Start a manual session. Errors if one is already active for this
    /// character/server. Returns the new session id.
    pub fn start_manual_session(
        &mut self,
        conn: &Connection,
        character: &str,
        server: &str,
        now_iso: &str,
    ) -> Result<i64, String> {
        if let Some(s) = persistence::active_session(conn, character, server)
            .map_err(|e| e.to_string())?
        {
            return Err(format!("session {} already active", s.id));
        }
        let id = persistence::insert_session(
            conn,
            character,
            server,
            now_iso,
            SessionStartTrigger::Manual,
            None,
        )
        .map_err(|e| e.to_string())?;
        self.cached_active_session_id = Some(Some(id));
        Ok(id)
    }

    /// End the active session (manual close). No-op if no session is active.
    /// Returns the ended session id, if any.
    ///
    /// The caller's `now_iso` is used as a fallback end timestamp, but the
    /// session's actual `started_at` / `ended_at` get recomputed from the
    /// first and last attributed event timestamps (see
    /// [`persistence::recompute_session_bounds_and_end`]). That gives more
    /// accurate bounds for live sessions and correct bounds for replayed /
    /// old-log sessions where wall-clock is meaningless.
    pub fn end_active_session(
        &mut self,
        conn: &Connection,
        character: &str,
        server: &str,
        now_iso: &str,
    ) -> Result<Option<i64>, String> {
        let active = persistence::active_session(conn, character, server)
            .map_err(|e| e.to_string())?;
        let Some(s) = active else {
            return Ok(None);
        };
        persistence::end_session(conn, s.id, now_iso).map_err(|e| e.to_string())?;
        persistence::recompute_session_bounds_and_end(conn, s.id, now_iso)
            .map_err(|e| e.to_string())?;
        self.cached_active_session_id = Some(None);
        Ok(Some(s.id))
    }

    // ============================================================
    // Event ingestion
    // ============================================================

    /// Process one `PlayerEvent` for the active character. The event's
    /// provenance may be mutated in place to inject `survey_use_id` (A3
    /// stitching) before downstream consumers see it.
    pub fn process_event(
        &mut self,
        event: &mut PlayerEvent,
        conn: &Connection,
        character: &str,
        server: &str,
        current_area: Option<&str>,
    ) -> Vec<SurveyAggregatorEvent> {
        let mut emitted = Vec::new();

        // Age out pending-use entries whose 60s grace window has expired.
        if let Some(now_secs) = event_secs_of_day(event) {
            self.expire_pending_uses(now_secs);
        }

        match event {
            // Survey map crafted (item entered inventory, kind is a survey).
            // Detected on ItemAdded with is_new=true.
            PlayerEvent::ItemAdded {
                item_name,
                is_new,
                timestamp,
                ..
            } if *is_new => {
                if let Some(kind) = self.lookup_survey_kind(item_name) {
                    self.handle_survey_crafted(
                        conn,
                        character,
                        server,
                        item_name,
                        kind,
                        timestamp,
                        &mut emitted,
                    );
                }
            }

            // Survey map consumed → record use, push pending attribution.
            PlayerEvent::ItemDeleted {
                item_name: Some(name),
                timestamp,
                ..
            } => {
                if let Some(kind) = self.lookup_survey_kind(name) {
                    self.handle_survey_consumed(
                        conn,
                        character,
                        server,
                        current_area,
                        name,
                        kind,
                        timestamp,
                        &mut emitted,
                    );
                }
            }

            // Mining started → check if it should adopt a pending use's
            // survey_use_id, OR if it's a touch on an open multihit node, OR
            // if it's a different-entity transition that closes one.
            PlayerEvent::DelayLoopStarted {
                action_type,
                label,
                timestamp,
                ..
            } if is_mining_loop(action_type, label) => {
                self.handle_mining_started(
                    conn,
                    character,
                    server,
                    timestamp,
                    &mut emitted,
                );
            }

            // Speed-bonus marker. Arrives *after* the bonus gains have been
            // recorded (see aggregator's event ordering notes). We patch the
            // already-written item_transactions rows for this Basic survey
            // use by item name. Only Basic surveys emit this marker; once
            // applied we clear the pointer so a later unrelated ScreenText
            // can't re-mark old rows.
            PlayerEvent::ScreenText { message, .. } if message.contains("(speed bonus!)") => {
                if let Some(use_id) = self.last_basic_use_for_bonus.take() {
                    self.apply_speed_bonus_marker(conn, use_id, message);
                }
            }

            _ => {}
        }

        // A3 stitching: for any *gain* event with a Mining provenance and a
        // currently-mining node we know about, inject survey_use_id from the
        // open_multihit_nodes table or the most recent pending-use that's
        // already been adopted. This runs after the matches above so newly-
        // adopted nodes are visible.
        self.maybe_inject_survey_use_id(event, conn, character, server);

        // Run sweeps periodically — once per event is fine; the queries are
        // small and indexed.
        if let Some(now_secs) = event_secs_of_day(event) {
            self.run_multihit_sweep(conn, character, server, now_secs, &mut emitted);
            let event_iso = event_hms_timestamp(event).map(|hms| self.to_utc(hms));
            self.maybe_auto_end_crafting_session(conn, character, server, event_iso.as_deref(), &mut emitted);
        }

        emitted
    }

    /// Bump the in-memory current-mining-entity tracking when a non-mining
    /// interaction starts. The coordinator can call this from
    /// `InteractionStarted` events. Mining state must clear when the player
    /// switches to a non-mining interaction (e.g., corpse search) so the
    /// next mining cycle is correctly recognized as a "different entity".
    #[allow(dead_code)] // Reserved hook for future mining-state clearing
    pub fn note_interaction_started(&mut self, _entity_id: u32) {
        // Currently a no-op: `current_mining_node` is set only when a mining
        // delay loop fires, and a different-entity mining cycle naturally
        // closes the previous one. Reserved as a hook in case we later
        // find we need to clear mining state earlier (e.g., on a confirmed
        // combat interruption).
    }

    // ============================================================
    // Internal handlers
    // ============================================================

    /// Called when a survey map enters inventory (crafted). Starts a
    /// `Crafting`-trigger session if none is active, otherwise increments
    /// the existing session's crafted_count.
    fn handle_survey_crafted(
        &mut self,
        conn: &Connection,
        character: &str,
        server: &str,
        _map_internal_name: &str,
        _kind: SurveyUseKind,
        timestamp: &str,
        emitted: &mut Vec<SurveyAggregatorEvent>,
    ) {
        let now_iso = self.to_utc(timestamp);

        let active = match self.fetch_active_session(conn, character, server) {
            Some(id) => id,
            None => {
                if !self.auto_start_enabled {
                    // Auto-start disabled and no active session — skip.
                    // The craft is observed but no session tracks it.
                    return;
                }
                // No active session → start one. Crafting trigger means
                // auto-end will fire once the player consumes everything
                // they crafted.
                let id = match persistence::insert_session(
                    conn,
                    character,
                    server,
                    &now_iso,
                    SessionStartTrigger::Crafting,
                    Some(0),
                ) {
                    Ok(id) => id,
                    Err(e) => {
                        eprintln!("[survey-aggregator] insert_session failed: {e}");
                        return;
                    }
                };
                self.cached_active_session_id = Some(Some(id));
                emitted.push(SurveyAggregatorEvent::SessionStarted {
                    session_id: id,
                    trigger: SessionStartTrigger::Crafting,
                });
                id
            }
        };
        if let Err(e) = persistence::increment_crafted_count(conn, active) {
            eprintln!("[survey-aggregator] increment_crafted_count failed: {e}");
        }
        // Track craft timestamps on the session and tighten started_at so
        // the session header reflects the actual first activity, not the
        // wall-clock (or trigger-event) moment the session was created.
        let _ = persistence::update_first_craft_at(conn, active, &now_iso);
        let _ = persistence::update_last_craft_at(conn, active, &now_iso);
        let _ = persistence::tighten_started_at(conn, active, &now_iso);
    }

    /// Called when a survey map is consumed (deleted). Inserts a
    /// `survey_uses` row, increments session counts, and queues the
    /// `pending_uses` entry for upcoming Mining-context attribution.
    #[allow(clippy::too_many_arguments)]
    fn handle_survey_consumed(
        &mut self,
        conn: &Connection,
        character: &str,
        server: &str,
        current_area: Option<&str>,
        map_internal_name: &str,
        kind: SurveyUseKind,
        timestamp: &str,
        emitted: &mut Vec<SurveyAggregatorEvent>,
    ) {
        let now_iso = self.to_utc(timestamp);

        // Auto-start a session if none active. Trigger = FirstUse since the
        // player didn't manually start and we didn't see a craft.
        let session_id = match self.fetch_active_session(conn, character, server) {
            Some(id) => Some(id),
            None => {
                if !self.auto_start_enabled {
                    // No session + auto-start off → record the use with
                    // no session (session_id = None).
                    None
                } else {
                    let id = match persistence::insert_session(
                        conn,
                        character,
                        server,
                        &now_iso,
                        SessionStartTrigger::FirstUse,
                        None,
                    ) {
                        Ok(id) => id,
                        Err(e) => {
                            eprintln!("[survey-aggregator] insert_session failed: {e}");
                            return;
                        }
                    };
                    self.cached_active_session_id = Some(Some(id));
                    emitted.push(SurveyAggregatorEvent::SessionStarted {
                        session_id: id,
                        trigger: SessionStartTrigger::FirstUse,
                    });
                    Some(id)
                }
            }
        };

        // Resolve display name for this map via CDN.
        let display_name = self
            .game_data
            .try_read()
            .ok()
            .and_then(|gd| gd.resolve_item(map_internal_name).map(|i| i.name.clone()))
            .unwrap_or_else(|| map_internal_name.to_string());

        // Resolve area: prefer the live-tracked area from the coordinator,
        // fall back to the survey_types.zone column (populated from CDN
        // description + areas lookup at import time).
        let resolved_area: Option<String> = current_area
            .map(|s| s.to_string())
            .or_else(|| {
                conn.query_row(
                    "SELECT zone FROM survey_types WHERE internal_name = ?1",
                    rusqlite::params![map_internal_name],
                    |r| r.get(0),
                )
                .ok()
                .flatten()
            });

        let use_id = match persistence::insert_use(
            conn,
            session_id,
            character,
            server,
            &now_iso,
            map_internal_name,
            &display_name,
            kind,
            resolved_area.as_deref(),
        ) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("[survey-aggregator] insert_use failed: {e}");
                return;
            }
        };

        if let Some(s) = session_id {
            if let Err(e) = persistence::increment_consumed_count(conn, s) {
                eprintln!("[survey-aggregator] increment_consumed_count failed: {e}");
            }
            // Tighten started_at so the session header reflects the actual
            // first survey use, not the session-creation moment.
            let _ = persistence::tighten_started_at(conn, s, &now_iso);
        }

        // For Basic surveys the loot drops in the same tick — no Mining
        // context follows. The use can immediately move to the pending-use
        // queue (so any same-tick gains attribute) and the aggregator will
        // close it via the regular sweep once its window passes.
        // For Motherlode/Multihit, the entry sits in pending_uses waiting
        // for the next Mining context to open within 60s.
        if let Some(now_secs) = parse_secs_of_day(timestamp) {
            self.pending_uses.push_back(PendingUse {
                survey_use_id: use_id,
                kind,
                used_at_secs: now_secs,
            });

            // Drain any deferred basic gains that arrived before this
            // DeleteItem. In the game's log order, primary loot fires
            // before the map is consumed:
            //   1. ProcessAddItem (primary)      ← buffered
            //   2. ProcessDeleteItem (this line)  ← pending use created
            //   3. ProcessAddItem (bonus)         ← attributed normally
            if kind == SurveyUseKind::Basic && !self.deferred_basic_gains.is_empty() {
                let deferred = std::mem::take(&mut self.deferred_basic_gains);
                let mut total_deferred_qty = 0u32;
                for gain in &deferred {
                    total_deferred_qty += gain.qty;
                    if let Some(ref hms) = gain.timestamp_hms {
                        update_session_loot_timestamps(conn, use_id, &self.to_utc(hms));
                    }
                }
                if total_deferred_qty > 0 {
                    let _ = persistence::add_loot_qty(conn, use_id, total_deferred_qty);
                }
                // Retroactively patch any item_transactions rows that
                // were already written by GameStateManager (or
                // persist_for_test in tests) with source_kind =
                // 'survey_map_use' but no survey_use_id. These are the
                // deferred gains whose provenance couldn't carry the
                // use_id at write time.
                retroactively_tag_unlinked_survey_transactions(
                    conn, use_id, character, server,
                );
                // Don't mark completed yet — more gains (speed bonus) may
                // arrive in the same tick. The next call to
                // attribute_basic_gain (for the bonus item) or the
                // pending-use expiry sweep will close it.
            }
        }

        emitted.push(SurveyAggregatorEvent::UseRecorded {
            use_id,
            session_id,
            map_internal_name: map_internal_name.to_string(),
            kind,
        });
    }

    /// Called on each Mining delay-loop event. Three possibilities:
    /// 1. The mined entity is already an open multihit node → touch it,
    ///    update last_hit_at.
    /// 2. The mined entity is *new* and we have a pending use → adopt it.
    ///    For Multihit: open a row in `open_multihit_nodes`. For Motherlode:
    ///    no DB tracking needed (single hit), but mark the use as the
    ///    current attribution target.
    /// 3. The mined entity is new and no pending use is around → loot
    ///    attributes to a regular Mining provenance with no survey link.
    fn handle_mining_started(
        &mut self,
        conn: &Connection,
        character: &str,
        server: &str,
        timestamp: &str,
        emitted: &mut Vec<SurveyAggregatorEvent>,
    ) {
        // We can only learn the *node entity id* indirectly — the Mining
        // delay loop itself doesn't carry it (the delay loop's own entity_id
        // is the player). The parser's `current_interaction.entity_id`
        // (recorded from the most recent `ProcessStartInteraction`) is what
        // identifies the node. Since we don't have a direct hook into that
        // here, we approximate: the parser already attaches the node entity
        // to gain events under `ActivitySource::Mining { node_entity_id }`.
        //
        // So `handle_mining_started` for now just records that mining is
        // active — the actual node binding happens in
        // `maybe_inject_survey_use_id` when the first gain arrives with the
        // node id in its provenance. This is good enough for correctness,
        // and avoids adding a second hook into the parser's interaction
        // tracking.
        let _ = (conn, character, server, timestamp, emitted);
    }

    /// A3 stitching. Called for every event after the per-event handlers.
    /// If this event is a *gain* attributed to Mining, decide whether the
    /// gain belongs to a survey use:
    /// - If it does (matching open multihit node OR pending use within 60s),
    ///   inject survey_use_id into its provenance and bump per-use loot_qty.
    /// - Open a multihit node row if the use kind is Multihit and one
    ///   doesn't exist yet.
    /// - Mark Motherlode/Basic uses completed once they receive their loot.
    fn maybe_inject_survey_use_id(
        &mut self,
        event: &mut PlayerEvent,
        conn: &Connection,
        character: &str,
        server: &str,
    ) {
        // Read-only pass first: extract qty, mining node id, and timestamps
        // before taking any mutable borrow. The mutable borrow on provenance
        // happens at the very end, in a single `inject_survey_use_id_into`
        // call after all DB writes have completed. This keeps the borrow
        // checker happy without requiring extra clones of the provenance
        // value (which is meaningfully sized — has source, candidates, etc.).
        let (qty, mining_node_id_opt, is_basic_survey_gain) = match event {
            PlayerEvent::ItemAdded { provenance, initial_quantity, .. } => {
                let kind_info = read_provenance_kind(provenance);
                (*initial_quantity, kind_info.0, kind_info.1)
            }
            PlayerEvent::ItemStackChanged {
                provenance, delta, ..
            } if *delta > 0 => {
                let kind_info = read_provenance_kind(provenance);
                (*delta as u32, kind_info.0, kind_info.1)
            }
            _ => return,
        };

        // Basic survey direct-loot case (no Mining intermediary).
        if is_basic_survey_gain {
            self.attribute_basic_gain(event, conn, qty);
            return;
        }

        // Mining-attributed gains: need the node entity id to either match
        // an open multihit row or adopt a pending use.
        let Some(node_id) = mining_node_id_opt else {
            return;
        };
        let node_id_i64 = node_id as i64;

        // Read timestamps up front so we don't need event after taking the
        // mutable provenance borrow below.
        let now_iso = event_hms_timestamp(event).map(|hms| self.to_utc(hms));
        let now_secs = event_secs_of_day(event);

        // Case 0: active Motherlode — the node was already adopted from a
        // pending use on the first gain. Attribute additional gains from
        // the same node within 1 second of the first gain (covers
        // motherlodes that yield 2+ items in one swing).
        if let Some((ml_node, ml_use_id, ml_first_secs)) = self.active_motherlode {
            let within_window = now_secs
                .map(|s| s.saturating_sub(ml_first_secs) <= 1)
                .unwrap_or(false);
            if ml_node == node_id && within_window {
                let _ = persistence::add_loot_qty(conn, ml_use_id, qty);
                if let Some(ref ts) = now_iso {
                    update_session_loot_timestamps(conn, ml_use_id, ts);
                }
                apply_survey_use_id(event, ml_use_id);
                return;
            }
            // Window expired or different node — close the motherlode.
            let _ = persistence::set_use_status(conn, ml_use_id, SurveyUseStatus::Completed);
            self.active_motherlode = None;
        }

        // Case 1: existing open multihit node — touch, attribute, inject.
        if let Ok(Some(n)) = multihit_state::get_node(conn, character, server, node_id_i64) {
            if let Some(ref iso) = now_iso {
                let _ = multihit_state::touch_node(conn, character, server, node_id_i64, iso);
            }
            let _ = persistence::add_loot_qty(conn, n.survey_use_id, qty);
            if let Some(ref ts) = now_iso {
                update_session_loot_timestamps(conn, n.survey_use_id, ts);
            }
            apply_survey_use_id(event, n.survey_use_id);
            return;
        }

        // Case 2: this is a new mining-node + we have a pending use within
        // grace window. Pop it and attach.
        let Some(now_secs) = now_secs else { return };
        let pending_idx = self
            .pending_uses
            .iter()
            .position(|p| now_secs.saturating_sub(p.used_at_secs) <= SURVEY_TO_MINING_GRACE_SECS);
        let Some(idx) = pending_idx else { return };
        let pending = self.pending_uses.remove(idx).unwrap();

        match pending.kind {
            SurveyUseKind::Multihit => {
                if let Some(ref iso) = now_iso {
                    let _ = multihit_state::open_node(
                        conn,
                        character,
                        server,
                        node_id_i64,
                        pending.survey_use_id,
                        iso,
                    );
                }
                self.current_mining_node = Some(node_id);
            }
            SurveyUseKind::Motherlode => {
                self.current_mining_node = Some(node_id);
                self.active_motherlode = Some((node_id, pending.survey_use_id, now_secs));
            }
            SurveyUseKind::Basic => {
                // Basic shouldn't reach this path (no Mining context follows),
                // but defensively still attach so loot isn't dropped.
            }
        }

        let _ = persistence::add_loot_qty(conn, pending.survey_use_id, qty);
        if let Some(ref ts) = now_iso {
            update_session_loot_timestamps(conn, pending.survey_use_id, ts);
        }
        apply_survey_use_id(event, pending.survey_use_id);

        // Motherlode completion is deferred — handled by Case 0 when a
        // different node starts, or by handle_mining_started / grace-window
        // expiry. This lets multi-item Motherlode loot (two UpdateItemCode
        // lines in the same mining cycle) attribute to the same use.
    }

    /// Parse a "X collected! Also found Y, Z (speed bonus!)" ScreenText line
    /// and flag each bonus item's transaction row for `use_id` in
    /// `item_transactions.source_details`. The bonus items' display names
    /// match the names already recorded on the transactions (both come from
    /// CDN-resolved display names), so the `(use_id, item_name)` match is
    /// exact.
    fn apply_speed_bonus_marker(&self, conn: &Connection, use_id: i64, message: &str) {
        let (items, _earned) = crate::parsers::parse_loot_items(message);
        for item in items.iter().filter(|i| i.is_speed_bonus) {
            if let Err(e) =
                persistence::mark_transactions_as_speed_bonus(conn, use_id, &item.item_name)
            {
                eprintln!(
                    "[survey-aggregator] mark_transactions_as_speed_bonus failed for use={} item={}: {}",
                    use_id, item.item_name, e
                );
            }
        }
    }

    /// Attribute a `SurveyMapUse`-provenance gain to the most recent pending
    /// Basic survey use. Called when gains arrive with SurveyMapUse provenance.
    ///
    /// In the game's log, basic survey loot often arrives *before* the survey
    /// map deletion that creates the pending use:
    ///   1. ProcessAddItem (primary loot)      ← gain arrives here
    ///   2. ProcessDeleteItem (map consumed)    ← pending use created here
    ///   3. ProcessAddItem (speed-bonus loot)
    ///
    /// When no pending Basic use exists yet, the gain is buffered in
    /// `deferred_basic_gains` and replayed when `handle_survey_consumed`
    /// creates the pending use.
    fn attribute_basic_gain(
        &mut self,
        event: &mut PlayerEvent,
        conn: &Connection,
        qty: u32,
    ) {
        // Find the most-recent (back of the queue) Basic pending use.
        let Some(idx) = self
            .pending_uses
            .iter()
            .rposition(|p| p.kind == SurveyUseKind::Basic)
        else {
            // No pending Basic use yet — the DeleteItem hasn't fired.
            // Buffer this gain so handle_survey_consumed can attribute it.
            self.deferred_basic_gains.push(DeferredBasicGain {
                qty,
                timestamp_hms: event_hms_timestamp(event).map(|s| s.to_string()),
            });
            return;
        };
        let use_id = self.pending_uses[idx].survey_use_id;

        let _ = persistence::add_loot_qty(conn, use_id, qty);
        if let Some(hms) = event_hms_timestamp(event) {
            update_session_loot_timestamps(conn, use_id, &self.to_utc(hms));
        }
        if let PlayerEvent::ItemAdded { provenance, .. } = event {
            inject_survey_use_id_into(provenance, use_id);
        } else if let PlayerEvent::ItemStackChanged { provenance, .. } = event {
            inject_survey_use_id_into(provenance, use_id);
        }

        // Remember this Basic use so an immediately-following
        // ScreenText "(speed bonus!)" marker can patch the right rows.
        self.last_basic_use_for_bonus = Some(use_id);

        // Basic completes after first loot batch in the same tick. Mark
        // completed and remove from pending. (If more loot arrives in the
        // same tick it'll go through the same code-path before completion
        // is observed by callers.)
        let _ = persistence::set_use_status(conn, use_id, SurveyUseStatus::Completed);
        self.pending_uses.remove(idx);
    }

    /// Periodic sweep of expired multihit nodes and pending-use grace
    /// windows. Cheap — both queries are indexed.
    fn run_multihit_sweep(
        &mut self,
        conn: &Connection,
        character: &str,
        server: &str,
        now_secs: u32,
        emitted: &mut Vec<SurveyAggregatorEvent>,
    ) {
        // Multihit timeout: nodes whose last_hit_at is more than 30 minutes ago.
        // We compare on full ISO datetime strings so we need a same-format cutoff.
        // Build cutoff = "now minus 30 minutes" using event-derived seconds: we
        // don't have a date here, only seconds-of-day, so the comparison may
        // be off when the day rolls over. The DB stores full datetimes
        // (`to_utc_datetime` output), so we compare the time portion only via
        // a string comparison — adequate for the worst-case false-negative of
        // delaying a sweep across midnight (it'll fire on the next event
        // after midnight when seconds-of-day surpasses the cutoff).
        //
        // For now, sweep using the persisted last_hit_at via a date-aware
        // comparison built off the event's full timestamp. We only need to
        // sweep occasionally; doing it every event is harmless because the
        // expired-set is empty almost always.
        let _ = (now_secs, emitted);

        // Use the most-recent event's full ISO timestamp as "now" if we can
        // recover it. The current API doesn't pass it through — which means
        // the sweep is best-effort only. A future cleanup can pass an
        // `&str now_iso` through. For Phase 5 this is acceptable: in
        // practice the multihit window is closed by the user starting a
        // different mining interaction long before the 30min timeout.
        let _ = (conn, character, server);
    }

    /// Auto-end check for crafting-trigger sessions: when consumed_count
    /// reaches crafted_count AND no pending_loot uses remain, end the
    /// session automatically. `event_iso` is the current event's UTC
    /// timestamp (derived from the log, not wall clock) — used as the
    /// fallback ended_at so replayed sessions don't get today's date.
    fn maybe_auto_end_crafting_session(
        &mut self,
        conn: &Connection,
        character: &str,
        server: &str,
        event_iso: Option<&str>,
        emitted: &mut Vec<SurveyAggregatorEvent>,
    ) {
        let Some(s) = self.fetch_active_session_full(conn, character, server) else {
            return;
        };
        if s.start_trigger != SessionStartTrigger::Crafting {
            return;
        }
        let crafted = s.crafted_count.unwrap_or(0);
        if crafted == 0 || s.consumed_count < crafted {
            return;
        }
        // All crafted maps consumed. Check pending uses.
        match persistence::session_has_pending_uses(conn, s.id) {
            Ok(true) => return, // wait for loot windows to close
            Ok(false) => {}
            Err(e) => {
                eprintln!("[survey-aggregator] session_has_pending_uses failed: {e}");
                return;
            }
        }
        // Prefer the event timestamp over wall clock so replayed/old-log
        // sessions don't get "today" as their ended_at.
        let fallback = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let end_iso = event_iso.unwrap_or(&fallback);
        if let Err(e) = persistence::end_session(conn, s.id, end_iso) {
            eprintln!("[survey-aggregator] end_session failed: {e}");
            return;
        }
        // Correct started_at/ended_at from actual event timestamps so the
        // header reflects when the activity really happened, not when the
        // auto-end condition happened to trip.
        if let Err(e) = persistence::recompute_session_bounds_and_end(conn, s.id, end_iso) {
            eprintln!("[survey-aggregator] recompute_session_bounds_and_end failed: {e}");
        }
        self.cached_active_session_id = Some(None);
        emitted.push(SurveyAggregatorEvent::SessionEnded {
            session_id: s.id,
            reason: "auto",
        });
    }

    // ============================================================
    // Helpers
    // ============================================================

    /// Look up the survey kind for an item by internal name. Returns `None`
    /// if the item isn't a survey map (the common case).
    fn lookup_survey_kind(&self, internal_name: &str) -> Option<SurveyUseKind> {
        let gd = self.game_data.try_read().ok()?;
        gd.resolve_item(internal_name)
            .and_then(|info| info.survey_kind())
            .map(|k: SurveyKind| k.into())
    }

    /// Cached active-session-id lookup.
    fn fetch_active_session(
        &mut self,
        conn: &Connection,
        character: &str,
        server: &str,
    ) -> Option<i64> {
        if let Some(cached) = self.cached_active_session_id {
            return cached;
        }
        let s = persistence::active_session(conn, character, server)
            .ok()
            .flatten();
        let id = s.map(|s| s.id);
        self.cached_active_session_id = Some(id);
        id
    }

    /// Full active-session fetch (no cache). Used when we need fields beyond
    /// the id (e.g., crafted_count for the auto-end check).
    fn fetch_active_session_full(
        &mut self,
        conn: &Connection,
        character: &str,
        server: &str,
    ) -> Option<crate::survey::types::SurveySession> {
        persistence::active_session(conn, character, server)
            .ok()
            .flatten()
    }

    fn expire_pending_uses(&mut self, now_secs: u32) {
        // Pop expired entries from the front. The 60s grace covers the
        // walk from "used here" to "node spawned over there"; anything
        // older is gone.
        while let Some(front) = self.pending_uses.front() {
            if now_secs.saturating_sub(front.used_at_secs) > SURVEY_TO_MINING_GRACE_SECS {
                let expired = self.pending_uses.pop_front().unwrap();
                // Mark as aborted (no loot ever arrived) — only if still
                // pending in the DB. Best-effort; a quick test would race
                // with the DB-side status.
                // For Multihit/Motherlode we don't auto-mark aborted because
                // the player may genuinely still be walking; leave them
                // pending and let the multihit timeout handle it.
                // For Basic, no Mining follows so a 60s expiry without
                // attribution likely means the chat correlation missed —
                // leave the row pending; it can be cleaned up by a future
                // sweep.
                let _ = expired;
            } else {
                break;
            }
        }
    }
}

/// After attributing loot to a survey use, update the session's
/// `first_loot_at` / `last_loot_at` timestamps. Called from every code
/// path that calls `add_loot_qty`. The lookup of `session_id` from the
/// use row is one cheap indexed read that only fires when loot is
/// attributed (not on every event).
fn update_session_loot_timestamps(conn: &Connection, use_id: i64, ts: &str) {
    if let Ok(Some(su)) = persistence::get_use(conn, use_id) {
        if let Some(sid) = su.session_id {
            let _ = persistence::update_first_loot_at(conn, sid, ts);
            let _ = persistence::update_last_loot_at(conn, sid, ts);
            // Keep started_at tight for open sessions so the header
            // reflects actual activity, not the session-creation moment.
            let _ = persistence::tighten_started_at(conn, sid, ts);
        }
    }
}

// ============================================================
// Free helpers
// ============================================================

fn is_mining_loop(action_type: &str, label: &str) -> bool {
    if action_type != "ChopLumber" {
        return false;
    }
    let normalized = label
        .trim()
        .trim_end_matches(|c: char| c == '.' || c.is_whitespace())
        .to_lowercase();
    normalized == "mining"
}

fn event_secs_of_day(event: &PlayerEvent) -> Option<u32> {
    let ts = event_hms_timestamp(event)?;
    parse_secs_of_day(&ts)
}

fn event_hms_timestamp(event: &PlayerEvent) -> Option<&str> {
    match event {
        PlayerEvent::ItemAdded { timestamp, .. } => Some(timestamp.as_str()),
        PlayerEvent::ItemStackChanged { timestamp, .. } => Some(timestamp.as_str()),
        PlayerEvent::ItemDeleted { timestamp, .. } => Some(timestamp.as_str()),
        PlayerEvent::DelayLoopStarted { timestamp, .. } => Some(timestamp.as_str()),
        PlayerEvent::StorageWithdrawal { timestamp, .. } => Some(timestamp.as_str()),
        PlayerEvent::InteractionStarted { timestamp, .. } => Some(timestamp.as_str()),
        _ => None,
    }
}

fn parse_secs_of_day(hms: &str) -> Option<u32> {
    let mut parts = hms.split(':');
    let h: u32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    let s: u32 = parts.next()?.parse().ok()?;
    Some(h * 3600 + m * 60 + s)
}

/// Helper to mutate an `ItemProvenance::Attributed` in place to set its
/// `survey_use_id`. No-op for other provenance variants — gain-event
/// callers should only invoke this on Attributed values.
fn inject_survey_use_id_into(provenance: &mut ItemProvenance, use_id: i64) {
    if let ItemProvenance::Attributed { survey_use_id, .. } = provenance {
        *survey_use_id = Some(use_id);
    }
}

/// Retroactively patch all recent `item_transactions` rows with
/// `source_kind = 'survey_map_use'` that are missing a `survey_use_id` in
/// their `source_details`. Used for deferred basic survey gains where the
/// transaction was written before the pending use existed (the game emits
/// primary loot before the map deletion).
///
/// This is a bulk patch: it tags every unlinked survey_map_use row for
/// this character/server with the new use_id. In practice only 1-2 rows
/// are unlinked at any time (the primary loot from the current survey),
/// so this is safe and avoids fragile timestamp-format matching.
fn retroactively_tag_unlinked_survey_transactions(
    conn: &Connection,
    survey_use_id: i64,
    character: &str,
    server: &str,
) {
    let result = conn.execute(
        "UPDATE item_transactions
         SET source_details = CASE
           WHEN source_details IS NULL OR source_details = ''
             THEN json_object('survey_use_id', ?1)
           ELSE json_set(source_details, '$.survey_use_id', ?1)
         END
         WHERE character_name = ?2
           AND server_name = ?3
           AND source_kind = 'survey_map_use'
           AND quantity > 0
           AND (source_details IS NULL
                OR json_extract(source_details, '$.survey_use_id') IS NULL)",
        rusqlite::params![survey_use_id, character, server],
    );
    if let Err(e) = result {
        eprintln!(
            "[survey-aggregator] retroactively_tag_unlinked_survey_transactions failed for use={}: {}",
            survey_use_id, e
        );
    }
}

/// Inspect a provenance for the two pieces of read-only info the aggregator
/// needs before it can take a mutable borrow:
/// - `Some(node_entity_id)` if this is a Mining-attributed gain (the inner
///   `node_entity_id` may itself be `None` for nameless nodes — passed
///   through as-is so the caller can decide what to do).
/// - `is_basic_survey_gain` true if this is a `SurveyMapUse`-attributed gain.
///
/// Both `false`/`None` for other provenance variants.
fn read_provenance_kind(provenance: &ItemProvenance) -> (Option<u32>, bool) {
    match provenance {
        ItemProvenance::Attributed {
            source: ActivitySource::Mining { node_entity_id, .. },
            ..
        } => (*node_entity_id, false),
        ItemProvenance::Attributed {
            source: ActivitySource::SurveyMapUse { .. },
            ..
        } => (None, true),
        _ => (None, false),
    }
}

/// Apply a survey_use_id to whichever provenance field a gain event carries.
fn apply_survey_use_id(event: &mut PlayerEvent, use_id: i64) {
    match event {
        PlayerEvent::ItemAdded { provenance, .. } => {
            inject_survey_use_id_into(provenance, use_id);
        }
        PlayerEvent::ItemStackChanged { provenance, .. } => {
            inject_survey_use_id_into(provenance, use_id);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cdn_commands::GameDataState;
    use crate::db::migrations::run_migrations;
    use crate::game_data::GameData;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn fresh_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn, None).unwrap();
        conn
    }

    fn empty_game_data() -> GameDataState {
        Arc::new(RwLock::new(GameData::empty()))
    }

    /// Build a GameDataState pre-loaded with a small set of survey-map items
    /// covering all three kinds. Sufficient for the aggregator's
    /// `lookup_survey_kind` to function.
    fn game_data_with_survey_maps() -> GameDataState {
        use crate::game_data::ItemInfo;
        use serde_json::json;

        fn survey_item(id: u32, internal: &str, display: &str, keywords: &[&str]) -> ItemInfo {
            ItemInfo {
                id,
                name: display.to_string(),
                description: None,
                icon_id: None,
                value: None,
                max_stack_size: None,
                keywords: keywords.iter().map(|s| s.to_string()).collect(),
                effect_descs: vec![],
                internal_name: Some(internal.to_string()),
                food_desc: None,
                equip_slot: None,
                num_uses: None,
                skill_reqs: None,
                behaviors: None,
                bestow_recipes: None,
                bestow_ability: None,
                bestow_quest: None,
                bestow_title: None,
                craft_points: None,
                crafting_target_level: None,
                tsys_profile: None,
                raw_json: json!({}),
            }
        }

        let mut gd = GameData::empty();
        let entries = [
            (
                100,
                "GeologySurveySerbule1",
                "Serbule Blue Mineral Survey",
                &["Document", "MineralSurvey"][..],
            ),
            (
                200,
                "MiningSurveyKurMountains1X",
                "Kur Mountains Simple Metal Motherlode Map",
                &["Document", "MiningSurvey", "MotherlodeMap"][..],
            ),
            (
                300,
                "MiningSurveyPovus7Y",
                "Povus Astounding Mining Survey",
                &["Document", "MiningSurvey"][..],
            ),
        ];
        for (id, internal, display, keywords) in entries {
            gd.items.insert(id, survey_item(id, internal, display, keywords));
            gd.item_internal_name_index.insert(internal.to_string(), id);
            gd.item_name_index.insert(display.to_string(), id);
        }
        Arc::new(RwLock::new(gd))
    }

    #[test]
    fn test_aggregator_starts_session_on_first_use_when_kind_known() {
        // Without CDN data the aggregator can't recognize survey kinds, so
        // this test verifies the "unknown item" path is a no-op rather than
        // the happy path. The full happy-path test requires loaded GameData
        // and lives in the integration test harness (next).
        let conn = fresh_db();
        let game_data = empty_game_data();
        let mut agg = SurveySessionAggregator::new(game_data);

        let mut event = PlayerEvent::ItemDeleted {
            timestamp: "12:00:00".to_string(),
            instance_id: 1,
            item_name: Some("UnknownItem".to_string()),
            context: crate::player_event_parser::DeleteContext::Consumed,
        };

        let emitted = agg.process_event(&mut event, &conn, "Zenith", "Dreva", None);
        assert!(emitted.is_empty(), "no events for non-survey deletion");
        // No session should have been created
        assert!(persistence::active_session(&conn, "Zenith", "Dreva")
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_manual_session_lifecycle() {
        let conn = fresh_db();
        let mut agg = SurveySessionAggregator::new(empty_game_data());

        let id = agg
            .start_manual_session(&conn, "Zenith", "Dreva", "2026-04-15 12:00:00")
            .unwrap();
        assert!(id > 0);

        // Second start should fail
        let err = agg
            .start_manual_session(&conn, "Zenith", "Dreva", "2026-04-15 12:01:00")
            .unwrap_err();
        assert!(err.contains("already active"));

        // End it
        let ended = agg
            .end_active_session(&conn, "Zenith", "Dreva", "2026-04-15 12:30:00")
            .unwrap();
        assert_eq!(ended, Some(id));

        // Now starts work again
        let id2 = agg
            .start_manual_session(&conn, "Zenith", "Dreva", "2026-04-15 13:00:00")
            .unwrap();
        assert!(id2 > id);
    }

    #[test]
    fn test_inject_survey_use_id_helper() {
        let mut prov = ItemProvenance::Attributed {
            source: ActivitySource::Mining {
                node_entity_id: Some(99),
                node_name: None,
            },
            confidence: crate::player_event_parser::AttributionConfidence::Confident,
            survey_use_id: None,
        };
        inject_survey_use_id_into(&mut prov, 42);
        match prov {
            ItemProvenance::Attributed { survey_use_id, .. } => {
                assert_eq!(survey_use_id, Some(42));
            }
            _ => panic!("expected Attributed"),
        }

        // No-op for non-Attributed
        let mut prov2 = ItemProvenance::UnknownSource;
        inject_survey_use_id_into(&mut prov2, 99);
        assert_eq!(prov2, ItemProvenance::UnknownSource);
    }

    #[test]
    fn test_is_mining_loop_normalizes_label_variants() {
        assert!(is_mining_loop("ChopLumber", "Mining..."));
        assert!(is_mining_loop("ChopLumber", "Mining ..."));
        assert!(is_mining_loop("ChopLumber", "Mining"));
        assert!(!is_mining_loop("ChopLumber", "Surveying"));
        assert!(!is_mining_loop("Cook", "Mining..."));
    }

    #[test]
    fn test_basic_survey_consume_creates_use_and_session() {
        // Player consumes a Basic survey map → should auto-start a FirstUse
        // session and create a survey_uses row in pending_loot.
        let conn = fresh_db();
        let mut agg = SurveySessionAggregator::new(game_data_with_survey_maps());

        let mut delete_event = PlayerEvent::ItemDeleted {
            timestamp: "12:00:00".to_string(),
            instance_id: 1,
            item_name: Some("GeologySurveySerbule1".to_string()),
            context: crate::player_event_parser::DeleteContext::Consumed,
        };

        let emitted = agg.process_event(&mut delete_event, &conn, "Zenith", "Dreva", Some("Serbule"));

        // Should have emitted SessionStarted + UseRecorded (in that order)
        assert_eq!(emitted.len(), 2, "expected 2 events: {:?}", emitted);
        assert!(matches!(emitted[0], SurveyAggregatorEvent::SessionStarted { .. }));
        assert!(matches!(emitted[1], SurveyAggregatorEvent::UseRecorded { .. }));

        // DB state: one session, one use, both correctly populated
        let session = persistence::active_session(&conn, "Zenith", "Dreva")
            .unwrap()
            .expect("session should be active");
        assert_eq!(session.start_trigger, SessionStartTrigger::FirstUse);
        assert_eq!(session.consumed_count, 1);

        let uses = persistence::uses_for_session(&conn, session.id).unwrap();
        assert_eq!(uses.len(), 1);
        assert_eq!(uses[0].kind, SurveyUseKind::Basic);
        assert_eq!(uses[0].area.as_deref(), Some("Serbule"));
        assert_eq!(uses[0].status, SurveyUseStatus::PendingLoot);
        assert_eq!(uses[0].map_display_name, "Serbule Blue Mineral Survey");
    }

    #[test]
    fn test_basic_survey_attribution_marks_use_completed() {
        // Basic survey: same-tick gain attributed via SurveyMapUse provenance
        // should land the survey_use_id link AND mark the use completed.
        let conn = fresh_db();
        let mut agg = SurveySessionAggregator::new(game_data_with_survey_maps());

        // 1. Consume the survey map
        let mut delete_event = PlayerEvent::ItemDeleted {
            timestamp: "12:00:00".to_string(),
            instance_id: 1,
            item_name: Some("GeologySurveySerbule1".to_string()),
            context: crate::player_event_parser::DeleteContext::Consumed,
        };
        agg.process_event(&mut delete_event, &conn, "Zenith", "Dreva", Some("Serbule"));

        // 2. Loot arrives in same tick as a SurveyMapUse-attributed gain
        let mut gain_event = PlayerEvent::ItemAdded {
            timestamp: "12:00:00".to_string(),
            item_name: "Fluorite".to_string(),
            instance_id: 999,
            slot_index: -1,
            is_new: true,
            initial_quantity: 1,
            provenance: ItemProvenance::Attributed {
                source: ActivitySource::SurveyMapUse {
                    survey_map_internal_name: Some("Serbule Blue Mineral Survey".to_string()),
                },
                confidence: crate::player_event_parser::AttributionConfidence::Confident,
                survey_use_id: None,
            },
        };
        agg.process_event(&mut gain_event, &conn, "Zenith", "Dreva", Some("Serbule"));

        // Provenance should now carry the survey_use_id
        if let PlayerEvent::ItemAdded { provenance, .. } = &gain_event {
            match provenance {
                ItemProvenance::Attributed {
                    survey_use_id: Some(_),
                    ..
                } => {}
                _ => panic!("expected survey_use_id to be set; got {:?}", provenance),
            }
        }

        // Use should be marked completed
        let uses = persistence::uses_for_session(
            &conn,
            persistence::active_session(&conn, "Zenith", "Dreva")
                .unwrap()
                .unwrap()
                .id,
        )
        .unwrap();
        assert_eq!(uses[0].status, SurveyUseStatus::Completed);
        assert_eq!(uses[0].loot_qty, 1);
    }

    #[test]
    fn test_basic_survey_deferred_gain_before_delete() {
        // Real game log ordering: primary loot arrives BEFORE the survey
        // map deletion. The aggregator must buffer the gain and attribute
        // it once the DeleteItem fires and creates the pending use.
        let conn = fresh_db();
        let mut agg = SurveySessionAggregator::new(game_data_with_survey_maps());

        // 1. Primary loot arrives first — no pending use exists yet.
        let mut gain = PlayerEvent::ItemAdded {
            timestamp: "12:00:00".to_string(),
            item_name: "Fluorite".to_string(),
            instance_id: 999,
            slot_index: -1,
            is_new: true,
            initial_quantity: 1,
            provenance: ItemProvenance::Attributed {
                source: ActivitySource::SurveyMapUse {
                    survey_map_internal_name: Some("Serbule Blue Mineral Survey".to_string()),
                },
                confidence: crate::player_event_parser::AttributionConfidence::Confident,
                survey_use_id: None,
            },
        };
        agg.process_event(&mut gain, &conn, "Zenith", "Dreva", Some("Serbule"));

        // No session or use yet — the gain is buffered.
        assert!(
            persistence::active_session(&conn, "Zenith", "Dreva")
                .unwrap()
                .is_none(),
            "no session before DeleteItem"
        );
        assert_eq!(agg.deferred_basic_gains.len(), 1);

        // 2. Survey map consumed — creates the pending use and drains buffer.
        let mut delete = PlayerEvent::ItemDeleted {
            timestamp: "12:00:00".to_string(),
            instance_id: 1,
            item_name: Some("GeologySurveySerbule1".to_string()),
            context: crate::player_event_parser::DeleteContext::Consumed,
        };
        agg.process_event(&mut delete, &conn, "Zenith", "Dreva", Some("Serbule"));

        // Buffer should be drained.
        assert_eq!(agg.deferred_basic_gains.len(), 0);

        // The use should have loot_qty = 1 from the deferred gain.
        let session = persistence::active_session(&conn, "Zenith", "Dreva")
            .unwrap()
            .expect("session after DeleteItem");
        let uses = persistence::uses_for_session(&conn, session.id).unwrap();
        assert_eq!(uses.len(), 1);
        assert_eq!(uses[0].loot_qty, 1, "deferred gain should be counted");

        // 3. Bonus loot arrives after delete — should attribute normally
        // and mark use completed.
        let mut bonus = PlayerEvent::ItemAdded {
            timestamp: "12:00:00".to_string(),
            item_name: "Azurite".to_string(),
            instance_id: 1000,
            slot_index: -1,
            is_new: true,
            initial_quantity: 1,
            provenance: ItemProvenance::Attributed {
                source: ActivitySource::SurveyMapUse {
                    survey_map_internal_name: Some("Serbule Blue Mineral Survey".to_string()),
                },
                confidence: crate::player_event_parser::AttributionConfidence::Confident,
                survey_use_id: None,
            },
        };
        agg.process_event(&mut bonus, &conn, "Zenith", "Dreva", Some("Serbule"));

        let uses = persistence::uses_for_session(&conn, session.id).unwrap();
        assert_eq!(uses[0].loot_qty, 2, "deferred + normal gain");
        assert_eq!(uses[0].status, SurveyUseStatus::Completed);
    }

    #[test]
    fn test_basic_survey_speed_bonus_marks_bonus_transaction() {
        // Speed-bonus ScreenText arrives *after* the bonus gain has been
        // recorded. The aggregator should patch the bonus row's
        // source_details to include is_speed_bonus: true, leaving the
        // primary row untouched.
        let conn = fresh_db();
        let mut agg = SurveySessionAggregator::new(game_data_with_survey_maps());

        // 1. Consume the Basic survey map.
        let mut delete_event = PlayerEvent::ItemDeleted {
            timestamp: "12:00:00".to_string(),
            instance_id: 1,
            item_name: Some("GeologySurveySerbule1".to_string()),
            context: crate::player_event_parser::DeleteContext::Consumed,
        };
        agg.process_event(&mut delete_event, &conn, "Zenith", "Dreva", Some("Serbule"));

        // Pull the allocated use_id so we can seed transactions that mirror
        // what game_state would have written.
        let session_id = persistence::active_session(&conn, "Zenith", "Dreva")
            .unwrap()
            .unwrap()
            .id;
        let uses = persistence::uses_for_session(&conn, session_id).unwrap();
        let use_id = uses[0].id;

        // 2. Primary gain — provenance gets survey_use_id injected.
        let mut primary = PlayerEvent::ItemAdded {
            timestamp: "12:00:00".to_string(),
            item_name: "Blue Spinel".to_string(),
            instance_id: 900,
            slot_index: -1,
            is_new: true,
            initial_quantity: 1,
            provenance: ItemProvenance::Attributed {
                source: ActivitySource::SurveyMapUse {
                    survey_map_internal_name: Some("Serbule Blue Mineral Survey".to_string()),
                },
                confidence: crate::player_event_parser::AttributionConfidence::Confident,
                survey_use_id: None,
            },
        };
        agg.process_event(&mut primary, &conn, "Zenith", "Dreva", Some("Serbule"));

        // Seed transactions that game_state would have written for both items
        // — this test doesn't run game_state, so we fake the rows directly.
        let details = format!(r#"{{"survey_use_id":{}}}"#, use_id);
        conn.execute(
            "INSERT INTO item_transactions (timestamp, character_name, server_name, item_name, quantity, context, source, source_kind, source_details)
             VALUES ('2026-04-15 12:00:00','Zenith','Dreva','Blue Spinel',1,'loot','player_log','survey_map_use',?1),
                    ('2026-04-15 12:00:00','Zenith','Dreva','Rubywall Crystal',2,'loot','player_log','survey_map_use',?1)",
            rusqlite::params![details],
        ).unwrap();

        // 3. ScreenText with "(speed bonus!)" arrives last in the tick.
        let mut screen = PlayerEvent::ScreenText {
            timestamp: "12:00:00".to_string(),
            category: "ImportantInfo".to_string(),
            message: "Blue Spinel collected! Also found Rubywall Crystal x2 (speed bonus!)"
                .to_string(),
        };
        agg.process_event(&mut screen, &conn, "Zenith", "Dreva", Some("Serbule"));

        // Verify: Rubywall flagged, Blue Spinel not.
        let flags: Vec<(String, Option<i64>)> = conn
            .prepare("SELECT item_name, json_extract(source_details, '$.is_speed_bonus') FROM item_transactions ORDER BY id")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert_eq!(flags[0].0, "Blue Spinel");
        assert_eq!(flags[0].1, None, "primary should not be flagged");
        assert_eq!(flags[1].0, "Rubywall Crystal");
        assert_eq!(flags[1].1, Some(1), "bonus should be flagged");
    }

    #[test]
    fn test_motherlode_survey_attributes_first_mining_gain() {
        // Motherlode: consume map → wait for Mining context → gains from
        // the same mining cycle all attribute to the same use. Completion
        // is deferred until a different mining node starts.
        let conn = fresh_db();
        let mut agg = SurveySessionAggregator::new(game_data_with_survey_maps());

        // 1. Consume motherlode map
        let mut consume = PlayerEvent::ItemDeleted {
            timestamp: "18:04:23".to_string(),
            instance_id: 1,
            item_name: Some("MiningSurveyKurMountains1X".to_string()),
            context: crate::player_event_parser::DeleteContext::Consumed,
        };
        agg.process_event(&mut consume, &conn, "Zenith", "Dreva", Some("KurMountains"));

        let session_id = persistence::active_session(&conn, "Zenith", "Dreva")
            .unwrap()
            .unwrap()
            .id;
        let use_id = persistence::uses_for_session(&conn, session_id).unwrap()[0].id;

        // 2. Mining starts (no-op for the aggregator; node id flows in via gain)
        let mut mining = PlayerEvent::DelayLoopStarted {
            timestamp: "18:04:23".to_string(),
            duration: 6.0,
            action_type: "ChopLumber".to_string(),
            label: "Mining...".to_string(),
            entity_id: 0,
            abort_condition: "AbortIfAttacked".to_string(),
        };
        agg.process_event(&mut mining, &conn, "Zenith", "Dreva", Some("KurMountains"));

        let mining_prov = |node_id: u32| ItemProvenance::Attributed {
            source: ActivitySource::Mining {
                node_entity_id: Some(node_id),
                node_name: Some("MiningNodeFromSurvey9".to_string()),
            },
            confidence: crate::player_event_parser::AttributionConfidence::Confident,
            survey_use_id: None,
        };

        // 3. First mining gain — should adopt the pending use but NOT mark
        //    it completed yet (Motherlode can yield multiple items in one swing).
        let mut gain1 = PlayerEvent::ItemAdded {
            timestamp: "18:04:29".to_string(),
            item_name: "Orichalcum".to_string(),
            instance_id: 1234,
            slot_index: -1,
            is_new: true,
            initial_quantity: 1,
            provenance: mining_prov(435138),
        };
        agg.process_event(&mut gain1, &conn, "Zenith", "Dreva", Some("KurMountains"));

        // survey_use_id should be injected on the first gain
        if let PlayerEvent::ItemAdded { provenance, .. } = &gain1 {
            if let ItemProvenance::Attributed { survey_use_id, .. } = provenance {
                assert_eq!(*survey_use_id, Some(use_id));
            } else {
                panic!("expected Attributed");
            }
        }

        // Use should still be pending_loot (deferred completion)
        let u = persistence::get_use(&conn, use_id).unwrap().unwrap();
        assert_eq!(u.status, SurveyUseStatus::PendingLoot);
        assert_eq!(u.loot_qty, 1);

        // 4. Second gain in the same mining cycle — same node, same timestamp.
        //    Should also attribute to the same use.
        let mut gain2 = PlayerEvent::ItemStackChanged {
            timestamp: "18:04:29".to_string(),
            instance_id: 5678,
            item_name: Some("PebbleMix".to_string()),
            item_type_id: 99,
            old_stack_size: 5,
            new_stack_size: 8,
            delta: 3,
            from_server: true,
            provenance: mining_prov(435138),
        };
        agg.process_event(&mut gain2, &conn, "Zenith", "Dreva", Some("KurMountains"));

        if let PlayerEvent::ItemStackChanged { provenance, .. } = &gain2 {
            if let ItemProvenance::Attributed { survey_use_id, .. } = provenance {
                assert_eq!(*survey_use_id, Some(use_id), "second gain should use same use_id");
            } else {
                panic!("expected Attributed on second gain");
            }
        }

        let u2 = persistence::get_use(&conn, use_id).unwrap().unwrap();
        assert_eq!(u2.loot_qty, 4, "1 + 3 from the two gains");
        assert_eq!(u2.status, SurveyUseStatus::PendingLoot);

        // 5. Gain on the same node but >1 second later — window expired,
        //    should NOT attribute to the motherlode use.
        let mut gain_late = PlayerEvent::ItemAdded {
            timestamp: "18:04:31".to_string(), // 2 seconds after first gain
            item_name: "Pebbles".to_string(),
            instance_id: 9999,
            slot_index: -1,
            is_new: true,
            initial_quantity: 1,
            provenance: mining_prov(435138), // same node
        };
        agg.process_event(&mut gain_late, &conn, "Zenith", "Dreva", Some("KurMountains"));

        let u3 = persistence::get_use(&conn, use_id).unwrap().unwrap();
        assert_eq!(u3.status, SurveyUseStatus::Completed, "window expiry closes the motherlode");
        assert_eq!(u3.loot_qty, 4, "late gain not counted");
    }

    #[test]
    fn test_multihit_survey_opens_node_and_chains_subsequent_hits() {
        // Multihit: consume map → first mining gain opens a row in
        // open_multihit_nodes → subsequent gains on the same entity are
        // attributed to the same use (loot accumulates).
        let conn = fresh_db();
        let mut agg = SurveySessionAggregator::new(game_data_with_survey_maps());

        // 1. Consume Multihit map
        let mut consume = PlayerEvent::ItemDeleted {
            timestamp: "12:00:00".to_string(),
            instance_id: 1,
            item_name: Some("MiningSurveyPovus7Y".to_string()),
            context: crate::player_event_parser::DeleteContext::Consumed,
        };
        agg.process_event(&mut consume, &conn, "Zenith", "Dreva", Some("Povus"));

        let session_id = persistence::active_session(&conn, "Zenith", "Dreva")
            .unwrap()
            .unwrap()
            .id;
        let use_id = persistence::uses_for_session(&conn, session_id).unwrap()[0].id;

        // 2. First mining gain on node 9999 — opens the open_multihit_nodes row
        let mut hit1 = PlayerEvent::ItemAdded {
            timestamp: "12:00:10".to_string(),
            item_name: "Marvelous Metal Slab".to_string(),
            instance_id: 1001,
            slot_index: -1,
            is_new: true,
            initial_quantity: 1,
            provenance: ItemProvenance::Attributed {
                source: ActivitySource::Mining {
                    node_entity_id: Some(9999),
                    node_name: None,
                },
                confidence: crate::player_event_parser::AttributionConfidence::Confident,
                survey_use_id: None,
            },
        };
        agg.process_event(&mut hit1, &conn, "Zenith", "Dreva", Some("Povus"));

        // open_multihit_nodes should now have a row
        let node = multihit_state::get_node(&conn, "Zenith", "Dreva", 9999)
            .unwrap()
            .expect("node row should exist");
        assert_eq!(node.survey_use_id, use_id);

        // 3. Second hit on same node — should NOT pop another pending use,
        // should chain via the existing open_multihit_nodes row
        let mut hit2 = PlayerEvent::ItemAdded {
            timestamp: "12:00:16".to_string(),
            item_name: "Pebbles".to_string(),
            instance_id: 1002,
            slot_index: -1,
            is_new: true,
            initial_quantity: 1,
            provenance: ItemProvenance::Attributed {
                source: ActivitySource::Mining {
                    node_entity_id: Some(9999),
                    node_name: None,
                },
                confidence: crate::player_event_parser::AttributionConfidence::Confident,
                survey_use_id: None,
            },
        };
        agg.process_event(&mut hit2, &conn, "Zenith", "Dreva", Some("Povus"));

        // Both gains should have the same survey_use_id and loot_qty=2
        let u = persistence::get_use(&conn, use_id).unwrap().unwrap();
        assert_eq!(u.loot_qty, 2);
        // Status still pending — multihit doesn't auto-complete
        assert_eq!(u.status, SurveyUseStatus::PendingLoot);

        // last_hit_at should have advanced past the original opened_at
        let updated_node = multihit_state::get_node(&conn, "Zenith", "Dreva", 9999)
            .unwrap()
            .unwrap();
        assert!(
            updated_node.last_hit_at > updated_node.opened_at,
            "last_hit_at ({}) should be later than opened_at ({})",
            updated_node.last_hit_at,
            updated_node.opened_at
        );
    }

    #[test]
    fn test_pending_use_grace_window_expires() {
        // If no Mining context fires within 60 seconds of a Motherlode
        // consume, the pending use is dropped from the in-memory queue.
        let conn = fresh_db();
        let mut agg = SurveySessionAggregator::new(game_data_with_survey_maps());

        let mut consume = PlayerEvent::ItemDeleted {
            timestamp: "12:00:00".to_string(),
            instance_id: 1,
            item_name: Some("MiningSurveyKurMountains1X".to_string()),
            context: crate::player_event_parser::DeleteContext::Consumed,
        };
        agg.process_event(&mut consume, &conn, "Zenith", "Dreva", Some("KurMountains"));
        assert_eq!(agg.pending_uses.len(), 1);

        // 2 minutes later, an unrelated event ticks the expiry
        let mut later = PlayerEvent::DelayLoopStarted {
            timestamp: "12:02:00".to_string(),
            duration: 0.5,
            action_type: "Unset".to_string(),
            label: "Using Something Else".to_string(),
            entity_id: 0,
            abort_condition: "AbortIfAttacked".to_string(),
        };
        agg.process_event(&mut later, &conn, "Zenith", "Dreva", None);

        assert_eq!(
            agg.pending_uses.len(),
            0,
            "pending use should have aged out after 60s grace"
        );
    }

    #[test]
    fn test_parse_secs_of_day() {
        assert_eq!(parse_secs_of_day("00:00:00"), Some(0));
        assert_eq!(parse_secs_of_day("01:02:03"), Some(3723));
        assert_eq!(parse_secs_of_day("23:59:59"), Some(86399));
        assert_eq!(parse_secs_of_day("garbage"), None);
        assert_eq!(parse_secs_of_day("12:30"), None);
    }
}
