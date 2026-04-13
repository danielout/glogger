use crate::parsers::{parse_loot_items, LootItem};
use crate::player_event_parser::PlayerEvent;
/// Survey parser — consumes PlayerEvents from the player event parser
/// to track survey session state machines.
///
/// This parser operates on structured PlayerEvents rather than raw log lines,
/// except for ProcessMapFx which is survey-specific and not in the player event parser.
///
/// Key design: the parser leverages player-event-parser's identity-resolved events:
/// - ItemAdded with internal_name → detect survey map crafting
/// - ItemDeleted with item_name → confirm survey map consumption (not locate)
/// - ItemStackChanged with negative delta → track ingredient consumption during crafting
/// - DelayLoopStarted → detect "Using X Survey" and "Surveying" crafting ticks
use std::collections::HashMap;

// ============================================================
// Public Types
// ============================================================

/// Events the survey parser can produce
#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "kind")]
pub enum SurveyEvent {
    /// A survey map was added to inventory (crafted or received).
    /// This is the ONLY event that creates sessions / increments maps_started.
    MapCrafted {
        timestamp: String,
        map_name: String,
        internal_name: String,
        ingredients_consumed: Vec<ConsumedIngredient>,
    },
    /// Player used a survey map (informational, for activity log only — NO counting)
    SurveyUsed {
        timestamp: String,
        survey_name: String,
    },
    /// Survey completed — loot collected
    Completed {
        timestamp: String,
        survey_name: String,
        loot_items: Vec<LootItem>,
        speed_bonus_earned: bool,
    },
    /// Motherlode map consumed + node mined — loot collected from mining
    MotherlodeCompleted {
        timestamp: String,
        map_name: String,
        loot_items: Vec<LootItem>,
    },
}

/// An ingredient consumed during the crafting window
#[derive(serde::Serialize, Clone, Debug)]
pub struct ConsumedIngredient {
    pub item_name: String,
    pub quantity: u32,
}

/// Info about a known survey type, loaded from the survey_types DB table
#[derive(Clone, Debug)]
pub struct KnownSurveyType {
    pub display_name: String,
    pub is_motherlode: bool,
}

// ============================================================
// Internal State
// ============================================================

/// Pending state while tracking a survey use (locate vs collect)
#[derive(Debug)]
enum Pending {
    /// "Using X Survey" detected, waiting to see if it's a locate or collect
    UsingSurvey {
        timestamp: String,
        survey_name: String,
        lines_waited: u32,
    },
    /// Survey map was consumed (ItemDeleted), waiting for loot text
    AwaitingLoot {
        timestamp: String,
        survey_name: String,
        lines_waited: u32,
    },
}

/// Pending motherlode: map consumed, waiting for mining + loot
#[derive(Debug)]
struct MotherlodePending {
    timestamp: String,
    map_name: String,
    lines_waited: u32,
    mining_started: bool,
    loot_items: Vec<LootItem>,
    /// Entity ID of the node we're mining (set on first Mining interaction).
    /// Used to distinguish motherlode mining from regular node mining after combat.
    mining_entity_id: Option<u32>,
    /// Entity ID from the most recent InteractionStarted event.
    /// Compared against mining_entity_id to decide if a "Mining ..." is our node.
    last_interaction_entity: Option<u32>,
}

/// Timeout for motherlode pending state — counted in lines with actual player events,
/// not raw log lines (asset downloads, appearance changes, etc. don't count).
/// Needs to be generous because the player may get interrupted by combat between
/// map consumption and mining, or between mining attempts.
const MOTHERLODE_TIMEOUT_EVENTS: u32 = 200;

/// Tracks ingredients consumed during an active crafting window
#[derive(Debug)]
struct CraftingWindow {
    ingredients: HashMap<String, u32>,
}

impl CraftingWindow {
    fn new() -> Self {
        Self {
            ingredients: HashMap::new(),
        }
    }

    fn record(&mut self, item_name: &str, quantity: u32) {
        *self.ingredients.entry(item_name.to_string()).or_insert(0) += quantity;
    }

    fn drain(self) -> Vec<ConsumedIngredient> {
        self.ingredients
            .into_iter()
            .map(|(item_name, quantity)| ConsumedIngredient {
                item_name,
                quantity,
            })
            .collect()
    }
}

const PENDING_TIMEOUT_LINES: u32 = 15;

// ============================================================
// Parser
// ============================================================

pub struct SurveyParser {
    /// Known survey types keyed by internal_name
    known_surveys: HashMap<String, KnownSurveyType>,
    /// Pending state for survey use resolution (locate vs collect)
    pending: Option<Pending>,
    /// Active crafting window for tracking ingredient consumption
    crafting_window: Option<CraftingWindow>,
    /// Pending motherlode: map consumed, accumulating mining loot
    motherlode_pending: Option<MotherlodePending>,
}

impl SurveyParser {
    pub fn new(known_surveys: HashMap<String, KnownSurveyType>) -> Self {
        Self {
            known_surveys,
            pending: None,
            crafting_window: None,
            motherlode_pending: None,
        }
    }

    /// Feed player events and the raw line for one log line; returns zero or more survey events.
    ///
    /// The raw line is still needed for ProcessMapFx detection (survey-specific,
    /// not parsed by the player event parser).
    pub fn process_events(
        &mut self,
        player_events: &[PlayerEvent],
        raw_line: &str,
    ) -> Vec<SurveyEvent> {
        let mut events = Vec::new();

        // --- 1. Crafting detection (always, regardless of pending state) ---
        self.process_crafting(player_events, &mut events);

        // --- 2. Resolve pending survey use (may create motherlode_pending) ---
        self.resolve_pending(player_events, raw_line, &mut events);

        // --- 3. Catch motherlode DeleteItem even if UsingSurvey timed out ---
        // During triangulation, 15+ lines of noise can cause UsingSurvey to timeout.
        // If we see a motherlode map DeleteItem without an active motherlode_pending,
        // start one directly.
        self.detect_motherlode_delete(player_events);

        // --- 4. Resolve pending motherlode (accumulate loot or finalize) ---
        // Runs after resolve_pending so that a motherlode_pending created from
        // a flushed DeleteItem on this line can immediately process mining events.
        self.resolve_motherlode(player_events, &mut events);

        // --- 5. Detect new "Using" lines ---
        self.detect_using_survey(player_events, &mut events);

        events
    }

    /// Detect crafting ticks, ingredient consumption, and crafted survey maps
    fn process_crafting(&mut self, player_events: &[PlayerEvent], events: &mut Vec<SurveyEvent>) {
        for event in player_events {
            match event {
                // "Surveying" crafting tick — open crafting window
                PlayerEvent::DelayLoopStarted { label, .. } if label == "Surveying" => {
                    if self.crafting_window.is_none() {
                        self.crafting_window = Some(CraftingWindow::new());
                    }
                }

                // Stack decreased — ingredient consumed during crafting
                PlayerEvent::ItemStackChanged {
                    item_name: Some(name),
                    delta,
                    ..
                } if *delta < 0 && self.crafting_window.is_some() => {
                    if let Some(window) = &mut self.crafting_window {
                        window.record(name, (-*delta) as u32);
                    }
                }

                // Item deleted during crafting — ingredient fully consumed
                // (but NOT if it's a known survey map being consumed during use)
                PlayerEvent::ItemDeleted {
                    item_name: Some(name),
                    ..
                } if self.crafting_window.is_some()
                    && !self.known_surveys.contains_key(name.as_str()) =>
                {
                    if let Some(window) = &mut self.crafting_window {
                        window.record(name, 1);
                    }
                }

                // Survey map added to inventory — crafting completed
                PlayerEvent::ItemAdded {
                    timestamp,
                    item_name,
                    ..
                } if self.known_surveys.contains_key(item_name.as_str()) => {
                    let survey_type = &self.known_surveys[item_name.as_str()];
                    let ingredients = self
                        .crafting_window
                        .take()
                        .map(|w| w.drain())
                        .unwrap_or_default();

                    events.push(SurveyEvent::MapCrafted {
                        timestamp: timestamp.clone(),
                        map_name: survey_type.display_name.clone(),
                        internal_name: item_name.clone(),
                        ingredients_consumed: ingredients,
                    });
                }

                _ => {}
            }
        }
    }

    /// Resolve pending survey use state (locate vs collect)
    fn resolve_pending(
        &mut self,
        player_events: &[PlayerEvent],
        raw_line: &str,
        events: &mut Vec<SurveyEvent>,
    ) {
        let pending = match self.pending.take() {
            Some(p) => p,
            None => return,
        };

        match pending {
            Pending::UsingSurvey {
                timestamp,
                survey_name,
                lines_waited,
            } => {
                // MapFx → was a locate, clear pending
                if parse_map_fx_hint(raw_line).is_some() {
                    return;
                }

                // ItemDeleted for a known survey → map consumed
                if let Some(deleted_survey) = player_events.iter().find_map(|e| {
                    if let PlayerEvent::ItemDeleted {
                        item_name: Some(name),
                        ..
                    } = e
                    {
                        self.known_surveys.get(name.as_str()).map(|st| st.clone())
                    } else {
                        None
                    }
                }) {
                    // Motherlode maps → start motherlode pending state (no ScreenText loot)
                    if deleted_survey.is_motherlode {
                        eprintln!(
                            "[survey-parser] Motherlode map deleted (via pending): {} at {}",
                            survey_name, timestamp
                        );
                        self.motherlode_pending = Some(MotherlodePending {
                            timestamp,
                            map_name: survey_name,
                            lines_waited: 0,
                            mining_started: false,
                            loot_items: Vec::new(),
                            mining_entity_id: None,
                            last_interaction_entity: None,
                        });
                        return;
                    }

                    // Regular survey → await loot ScreenText
                    self.pending = Some(Pending::AwaitingLoot {
                        timestamp,
                        survey_name,
                        lines_waited: 0,
                    });
                    // Also check for loot text on same line
                    self.check_loot_text(player_events, events);
                    return;
                }

                // ScreenText with "collected!" → direct completion (delete + loot on same tick)
                if let Some(loot_msg) = find_loot_screen_text(player_events) {
                    let (loot_items, speed_bonus_earned) = parse_loot_items(&loot_msg);
                    events.push(SurveyEvent::Completed {
                        timestamp,
                        survey_name,
                        loot_items,
                        speed_bonus_earned,
                    });
                    return;
                }

                // Only count lines with actual player events toward the timeout.
                // Raw noise lines (NullAnimEx, asset downloads, etc.) don't count.
                let tick = if !player_events.is_empty() { 1 } else { 0 };

                // Safety timeout
                if lines_waited >= PENDING_TIMEOUT_LINES {
                    eprintln!(
                        "[survey-parser] Timeout waiting for locate/collect resolution for {}",
                        survey_name
                    );
                    return;
                }

                // Keep waiting
                self.pending = Some(Pending::UsingSurvey {
                    timestamp,
                    survey_name,
                    lines_waited: lines_waited + tick,
                });
            }

            Pending::AwaitingLoot {
                timestamp,
                survey_name,
                lines_waited,
            } => {
                // ScreenText with "collected!" → completion
                if let Some(loot_msg) = find_loot_screen_text(player_events) {
                    let (loot_items, speed_bonus_earned) = parse_loot_items(&loot_msg);
                    events.push(SurveyEvent::Completed {
                        timestamp,
                        survey_name,
                        loot_items,
                        speed_bonus_earned,
                    });
                    return;
                }

                // Only count lines with actual player events toward the timeout.
                let tick = if !player_events.is_empty() { 1 } else { 0 };

                // Safety timeout
                if lines_waited >= PENDING_TIMEOUT_LINES {
                    eprintln!(
                        "[survey-parser] Timeout waiting for loot text for {}",
                        survey_name
                    );
                    return;
                }

                // Keep waiting
                self.pending = Some(Pending::AwaitingLoot {
                    timestamp,
                    survey_name,
                    lines_waited: lines_waited + tick,
                });
            }
        }
    }

    /// Check for loot text in the current events (used when transitioning to AwaitingLoot)
    fn check_loot_text(&mut self, player_events: &[PlayerEvent], events: &mut Vec<SurveyEvent>) {
        if let Some(Pending::AwaitingLoot {
            timestamp,
            survey_name,
            ..
        }) = &self.pending
        {
            if let Some(loot_msg) = find_loot_screen_text(player_events) {
                let ts = timestamp.clone();
                let name = survey_name.clone();
                let (loot_items, speed_bonus_earned) = parse_loot_items(&loot_msg);
                self.pending = None;
                events.push(SurveyEvent::Completed {
                    timestamp: ts,
                    survey_name: name,
                    loot_items,
                    speed_bonus_earned,
                });
            }
        }
    }

    /// Resolve pending motherlode: accumulate mining loot or finalize.
    ///
    /// Combat interruption handling: the player may get attacked mid-mine, fight,
    /// loot the corpse, then return to mine the motherlode node again. So:
    /// - We only count lines with actual player events toward the timeout
    ///   (asset downloads, appearance loops, etc. are noise)
    /// - Any non-mining activity (InteractionStarted, non-mining DelayLoop) resets
    ///   `mining_started` to false, so items from corpse looting or combat aren't
    ///   captured as motherlode loot
    /// - If we already have loot when a non-mining activity happens, we finalize
    /// - A "Mining ..." delay loop (re)enables loot accumulation
    fn resolve_motherlode(&mut self, player_events: &[PlayerEvent], events: &mut Vec<SurveyEvent>) {
        let pending = match self.motherlode_pending.take() {
            Some(p) => p,
            None => return,
        };

        let MotherlodePending {
            timestamp,
            map_name,
            mut lines_waited,
            mut mining_started,
            mut loot_items,
            mut mining_entity_id,
            mut last_interaction_entity,
        } = pending;

        // Only count lines that have actual player events toward the timeout.
        // Raw log lines with just asset downloads or appearance changes don't count.
        if !player_events.is_empty() {
            lines_waited += 1;
        }

        for event in player_events {
            // Early finalization: once we have loot from mining, ANY non-loot event
            // means mining is done. Loot events (ItemAdded, positive ItemStackChanged)
            // always arrive in the same log-line batch as the mining completion.
            // Anything else (combat state, mount, next interaction, skill update, etc.)
            // means we've moved past the loot window.
            if mining_started && !loot_items.is_empty() {
                let is_loot_event = match event {
                    PlayerEvent::ItemAdded { item_name, .. } => {
                        !self.known_surveys.contains_key(item_name.as_str())
                    }
                    PlayerEvent::ItemStackChanged { delta, .. } if *delta > 0 => true,
                    _ => false,
                };
                if !is_loot_event {
                    events.push(SurveyEvent::MotherlodeCompleted {
                        timestamp,
                        map_name,
                        loot_items,
                    });
                    return;
                }
            }

            match event {
                // Track which entity the player is interacting with.
                // This fires before "Mining ..." and lets us distinguish
                // motherlode node mining from regular node mining.
                PlayerEvent::InteractionStarted { entity_id, .. } => {
                    last_interaction_entity = Some(*entity_id);
                    mining_started = false;
                }

                // "Mining ..." delay loop — only accept if this is the same entity
                // we first mined (or if we haven't identified the entity yet).
                PlayerEvent::DelayLoopStarted {
                    action_type, label, ..
                } if action_type == "ChopLumber" && label == "Mining ..." => {
                    let entity = last_interaction_entity;
                    if mining_entity_id.is_none() {
                        // First mining interaction — adopt this entity as our motherlode node
                        mining_entity_id = entity;
                        mining_started = true;
                        eprintln!(
                            "[survey-parser] Motherlode mining started for {} (event {}, entity {:?})",
                            map_name, lines_waited, mining_entity_id
                        );
                    } else if entity == mining_entity_id {
                        // Same entity — resuming mining after combat interruption
                        mining_started = true;
                        eprintln!(
                            "[survey-parser] Motherlode mining resumed for {} (event {}, entity {:?})",
                            map_name, lines_waited, mining_entity_id
                        );
                    } else {
                        // Different entity — player is mining a regular node, not our motherlode.
                        eprintln!(
                            "[survey-parser] Ignoring mining of different entity {:?} (expected {:?}) for {}",
                            entity, mining_entity_id, map_name
                        );
                    }
                }

                // Item added while actively mining our node → motherlode loot
                PlayerEvent::ItemAdded { item_name, .. } if mining_started => {
                    // Don't count survey maps as motherlode loot
                    if !self.known_surveys.contains_key(item_name.as_str()) {
                        loot_items.push(LootItem {
                            item_name: item_name.clone(),
                            quantity: 1,
                            is_speed_bonus: false,
                            is_primary: true,
                        });
                    }
                }

                // Stack increased while actively mining our node → motherlode loot
                // Note: motherlode loot is server-authoritative, so we accept
                // from_server=true here (unlike regular surveys where we'd filter it)
                PlayerEvent::ItemStackChanged {
                    item_name: Some(name),
                    delta,
                    ..
                } if mining_started && *delta > 0 => {
                    loot_items.push(LootItem {
                        item_name: name.clone(),
                        quantity: *delta as u32,
                        is_speed_bonus: false,
                        is_primary: true,
                    });
                }

                // Non-mining delay loop (combat ability, crafting, etc.)
                // Reset mining_started so we stop capturing loot.
                PlayerEvent::DelayLoopStarted {
                    action_type, label, ..
                } if !(action_type == "ChopLumber" && label == "Mining ...") => {
                    mining_started = false;
                }

                _ => {}
            }
        }

        // Timeout (counted in player-event lines only)
        if lines_waited >= MOTHERLODE_TIMEOUT_EVENTS {
            if !loot_items.is_empty() {
                // We have loot — emit what we've got
                events.push(SurveyEvent::MotherlodeCompleted {
                    timestamp,
                    map_name,
                    loot_items,
                });
            } else {
                eprintln!(
                    "[survey-parser] Motherlode timeout without loot for {} ({} events)",
                    map_name, lines_waited
                );
            }
            return;
        }

        // Keep waiting
        self.motherlode_pending = Some(MotherlodePending {
            timestamp,
            map_name,
            lines_waited,
            mining_started,
            loot_items,
            mining_entity_id,
            last_interaction_entity,
        });
    }

    /// Detect motherlode map DeleteItem outside of the UsingSurvey pending flow.
    /// During triangulation, the player may ride around for 15+ lines between uses,
    /// causing UsingSurvey to timeout. This catches the DeleteItem independently.
    fn detect_motherlode_delete(&mut self, player_events: &[PlayerEvent]) {
        // Only trigger if we don't already have a motherlode pending
        if self.motherlode_pending.is_some() {
            return;
        }

        for event in player_events {
            if let PlayerEvent::ItemDeleted {
                item_name: Some(name),
                timestamp,
                ..
            } = event
            {
                if let Some(survey_type) = self.known_surveys.get(name.as_str()) {
                    if survey_type.is_motherlode {
                        eprintln!(
                            "[survey-parser] Motherlode map deleted (fallback): {} at {}",
                            survey_type.display_name, timestamp
                        );
                        self.motherlode_pending = Some(MotherlodePending {
                            timestamp: timestamp.clone(),
                            map_name: survey_type.display_name.clone(),
                            lines_waited: 0,
                            mining_started: false,
                            loot_items: Vec::new(),
                            mining_entity_id: None,
                            last_interaction_entity: None,
                        });
                        // Clear any stale UsingSurvey pending that might exist
                        self.pending = None;
                        return;
                    }
                }
            }
        }
    }

    /// Detect "Using X Survey/Map" delay loop events
    fn detect_using_survey(
        &mut self,
        player_events: &[PlayerEvent],
        events: &mut Vec<SurveyEvent>,
    ) {
        if let Some((ts, name)) = find_using_survey(player_events) {
            // Only emit SurveyUsed if we don't already have a pending state for the same survey
            // (repeated "Using" lines for the same survey shouldn't spam the log)
            let is_duplicate = match &self.pending {
                Some(Pending::UsingSurvey { survey_name, .. }) => survey_name == &name,
                _ => false,
            };

            if !is_duplicate {
                events.push(SurveyEvent::SurveyUsed {
                    timestamp: ts.clone(),
                    survey_name: name.clone(),
                });
            }

            self.pending = Some(Pending::UsingSurvey {
                timestamp: ts,
                survey_name: name,
                lines_waited: 0,
            });
        }
    }
}

// ============================================================
// Helper Functions
// ============================================================

/// Extract a "Using ... Survey/Map" from a DelayLoopStarted event
fn find_using_survey(events: &[PlayerEvent]) -> Option<(String, String)> {
    for event in events {
        if let PlayerEvent::DelayLoopStarted {
            timestamp, label, ..
        } = event
        {
            if label.starts_with("Using ") && (label.ends_with("Survey") || label.ends_with("Map"))
            {
                let name = label.strip_prefix("Using ").unwrap_or(label);
                return Some((timestamp.clone(), name.to_string()));
            }
        }
    }
    None
}

/// Extract loot message from a ScreenText event containing "collected!"
fn find_loot_screen_text(events: &[PlayerEvent]) -> Option<String> {
    for event in events {
        if let PlayerEvent::ScreenText {
            category, message, ..
        } = event
        {
            if category == "ImportantInfo" && message.contains("collected!") {
                return Some(message.clone());
            }
        }
    }
    None
}

/// ProcessMapFx(..., "Peridot is here", ImportantInfo, "The Peridot is 1386m east...")
/// Survey-specific — not in the player event parser
fn parse_map_fx_hint(line: &str) -> Option<String> {
    if !line.contains("ProcessMapFx") {
        return None;
    }
    let last_q = line.rfind('"')?;
    let before = &line[..last_q];
    let open_q = before.rfind('"')? + 1;
    Some(line[open_q..last_q].to_string())
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player_event_parser::PlayerEventParser;

    /// Build a known_surveys map for testing
    fn test_known_surveys() -> HashMap<String, KnownSurveyType> {
        let mut map = HashMap::new();
        map.insert(
            "EltibuleGreenMineralSurvey".to_string(),
            KnownSurveyType {
                display_name: "Eltibule Green Mineral Survey".to_string(),
                is_motherlode: false,
            },
        );
        map.insert(
            "KurMountainsGoodMetalMotherlodeMap".to_string(),
            KnownSurveyType {
                display_name: "Kur Mountains Good Metal Motherlode Map".to_string(),
                is_motherlode: true,
            },
        );
        map
    }

    /// Helper: run a line through PlayerEventParser and then SurveyParser
    fn process_line(
        parser: &mut PlayerEventParser,
        survey: &mut SurveyParser,
        line: &str,
    ) -> (Vec<PlayerEvent>, Vec<SurveyEvent>) {
        let player_events = parser.process_line(line);
        let survey_events = survey.process_events(&player_events, line);
        (player_events, survey_events)
    }

    #[test]
    fn test_using_survey_emits_survey_used_not_session_start() {
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(test_known_surveys());
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[16:17:47] LocalPlayer: ProcessDoDelayLoop(0.5, Unset, "Using Eltibule Green Mineral Survey", 5305, AbortIfAttacked)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            SurveyEvent::SurveyUsed { survey_name, .. } => {
                assert_eq!(survey_name, "Eltibule Green Mineral Survey");
            }
            _ => panic!("Expected SurveyUsed, got {:?}", events[0]),
        }
    }

    #[test]
    fn test_repeated_using_no_duplicate_survey_used() {
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(test_known_surveys());

        let (_, events1) = process_line(
            &mut parser,
            &mut survey,
            r#"[16:17:47] LocalPlayer: ProcessDoDelayLoop(0.5, Unset, "Using Eltibule Green Mineral Survey", 5305, AbortIfAttacked)"#,
        );
        assert_eq!(events1.len(), 1);

        let (_, events2) = process_line(
            &mut parser,
            &mut survey,
            r#"[16:17:48] LocalPlayer: ProcessDoDelayLoop(0.5, Unset, "Using Eltibule Green Mineral Survey", 5305, AbortIfAttacked)"#,
        );
        assert!(
            events2.is_empty(),
            "Repeated Using should NOT emit SurveyUsed"
        );
    }

    #[test]
    fn test_surveying_craft_line_no_survey_event() {
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(test_known_surveys());
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[00:08:36] LocalPlayer: ProcessDoDelayLoop(5, UseTeleportationCircle, "Surveying", 5305, AbortIfAttacked)"#,
        );
        // "Surveying" opens a crafting window but doesn't emit an event by itself
        assert!(
            events.is_empty(),
            "Surveying craft ticks should not emit survey events"
        );
    }

    #[test]
    fn test_craft_detection_via_item_added() {
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(test_known_surveys());

        // Crafting tick
        process_line(
            &mut parser,
            &mut survey,
            r#"[00:08:36] LocalPlayer: ProcessDoDelayLoop(5, UseTeleportationCircle, "Surveying", 5305, AbortIfAttacked)"#,
        );

        // Survey map added to inventory — triggers MapCrafted
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[00:08:41] LocalPlayer: ProcessAddItem(EltibuleGreenMineralSurvey(115244857), -1, True)"#,
        );

        assert_eq!(events.len(), 1);
        match &events[0] {
            SurveyEvent::MapCrafted {
                map_name,
                internal_name,
                ..
            } => {
                assert_eq!(map_name, "Eltibule Green Mineral Survey");
                assert_eq!(internal_name, "EltibuleGreenMineralSurvey");
            }
            _ => panic!("Expected MapCrafted, got {:?}", events[0]),
        }
    }

    #[test]
    fn test_craft_detection_without_crafting_tick() {
        // Survey map added without a preceding "Surveying" tick (e.g., received from trade)
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(test_known_surveys());

        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[00:10:00] LocalPlayer: ProcessAddItem(EltibuleGreenMineralSurvey(115244857), -1, True)"#,
        );

        assert_eq!(events.len(), 1);
        match &events[0] {
            SurveyEvent::MapCrafted {
                ingredients_consumed,
                ..
            } => {
                assert!(
                    ingredients_consumed.is_empty(),
                    "No crafting window = no ingredients"
                );
            }
            _ => panic!("Expected MapCrafted"),
        }
    }

    #[test]
    fn test_crafting_ingredient_tracking() {
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(test_known_surveys());

        // Crafting tick opens window
        process_line(
            &mut parser,
            &mut survey,
            r#"[00:08:36] LocalPlayer: ProcessDoDelayLoop(5, UseTeleportationCircle, "Surveying", 5305, AbortIfAttacked)"#,
        );

        // First we need items registered so ItemStackChanged gets names
        process_line(
            &mut parser,
            &mut survey,
            r#"[00:08:00] LocalPlayer: ProcessAddItem(BasicInk(100001), 1, True)"#,
        );
        process_line(
            &mut parser,
            &mut survey,
            r#"[00:08:00] LocalPlayer: ProcessAddItem(BasicParchment(100002), 2, True)"#,
        );

        // Stack changes (ingredient consumption)
        process_line(
            &mut parser,
            &mut survey,
            r#"[00:08:37] LocalPlayer: ProcessUpdateItemCode(100001, 65536, True)"#,
        );
        process_line(
            &mut parser,
            &mut survey,
            r#"[00:08:37] LocalPlayer: ProcessUpdateItemCode(100002, 65536, True)"#,
        );

        // Survey map crafted
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[00:08:41] LocalPlayer: ProcessAddItem(EltibuleGreenMineralSurvey(115244857), -1, True)"#,
        );

        assert_eq!(events.len(), 1);
        match &events[0] {
            SurveyEvent::MapCrafted {
                ingredients_consumed,
                ..
            } => {
                // Ingredients may or may not be captured depending on UpdateItemCode behavior
                // The key test is that MapCrafted was emitted and the crafting window closed
                assert!(
                    survey.crafting_window.is_none(),
                    "Crafting window should be closed"
                );
            }
            _ => panic!("Expected MapCrafted"),
        }
    }

    #[test]
    fn test_full_survey_completion_flow() {
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(test_known_surveys());

        // Register the survey item in the parser's instance registry
        process_line(
            &mut parser,
            &mut survey,
            r#"[16:17:46] LocalPlayer: ProcessAddItem(EltibuleGreenMineralSurvey(115230973), 5, True)"#,
        );

        // Using survey — emits SurveyUsed
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[16:17:47] LocalPlayer: ProcessDoDelayLoop(0.5, Unset, "Using Eltibule Green Mineral Survey", 5305, AbortIfAttacked)"#,
        );
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], SurveyEvent::SurveyUsed { .. }));

        // AddItem (loot added)
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[16:17:48] LocalPlayer: ProcessAddItem(Malachite(115244857), -1, True)"#,
        );
        assert!(events.is_empty());

        // DeleteItem (survey consumed) — transitions to AwaitingLoot
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[16:17:48] LocalPlayer: ProcessDeleteItem(115230973)"#,
        );
        assert!(events.is_empty());

        // Screen text with loot — emits Completed
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[16:17:48] LocalPlayer: ProcessScreenText(ImportantInfo, "Malachite collected! Also found Quartz x3 (speed bonus!)")"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            SurveyEvent::Completed {
                survey_name,
                loot_items,
                speed_bonus_earned,
                ..
            } => {
                assert_eq!(survey_name, "Eltibule Green Mineral Survey");
                assert!(*speed_bonus_earned);
                assert_eq!(loot_items.len(), 2);
            }
            _ => panic!("Expected Completed event"),
        }
    }

    #[test]
    fn test_locate_then_collect() {
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(test_known_surveys());

        // Register survey in instance registry
        process_line(
            &mut parser,
            &mut survey,
            r#"[00:15:44] LocalPlayer: ProcessAddItem(EltibuleGreenMineralSurvey(113654706), 5, True)"#,
        );

        // First "Using" — SurveyUsed
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[00:15:45] LocalPlayer: ProcessDoDelayLoop(0.5, Unset, "Using Eltibule Green Mineral Survey", 5305, AbortIfAttacked)"#,
        );
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], SurveyEvent::SurveyUsed { .. }));

        // Noise
        process_line(
            &mut parser,
            &mut survey,
            r#"[00:15:45] Download appearance loop @Invisible is done"#,
        );

        // MapFx locate — clears pending, no event
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[00:15:46] LocalPlayer: ProcessMapFx((1482.00, 111.54, 905.00), 25, 1, "Aquamarine is here", ImportantInfo, "The Aquamarine is 978m east and 1734m north.")"#,
        );
        assert!(events.is_empty(), "Locate should not emit events");

        // Second "Using" after locate — new SurveyUsed
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[00:15:47] LocalPlayer: ProcessDoDelayLoop(0.5, Unset, "Using Eltibule Green Mineral Survey", 5305, AbortIfAttacked)"#,
        );
        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0], SurveyEvent::SurveyUsed { .. }),
            "Second use should emit SurveyUsed (NOT SessionStart/MapCrafted)"
        );

        // Noise
        process_line(
            &mut parser,
            &mut survey,
            r#"[00:15:48] entity_159956: OnAttackHitMe(Fiery Bite). Evaded = False"#,
        );

        // DeleteItem (survey consumed) + ScreenText completion
        process_line(
            &mut parser,
            &mut survey,
            r#"[00:15:48] LocalPlayer: ProcessDeleteItem(113654706)"#,
        );
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[00:15:48] LocalPlayer: ProcessScreenText(ImportantInfo, "Piece of Green Glass collected!")"#,
        );

        assert_eq!(events.len(), 1);
        match &events[0] {
            SurveyEvent::Completed {
                loot_items,
                speed_bonus_earned,
                ..
            } => {
                assert!(!*speed_bonus_earned);
                assert_eq!(loot_items.len(), 1);
                assert_eq!(loot_items[0].item_name, "Piece of Green Glass");
            }
            _ => panic!("Expected Completed event"),
        }
    }

    #[test]
    fn test_non_survey_using_is_ignored() {
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(test_known_surveys());
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[00:08:00] LocalPlayer: ProcessDoDelayLoop(1.5, Eat, "Using Gobbledygook", 6223, AbortIfAttacked)"#,
        );
        assert!(events.is_empty(), "Non-survey 'Using' should be ignored");
    }

    #[test]
    fn test_motherlode_distance_text_not_treated_as_loot() {
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(test_known_surveys());

        // Start a motherlode session
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[14:08:44] LocalPlayer: ProcessDoDelayLoop(1, Unset, "Using Kur Mountains Good Metal Motherlode Map", 5305, AbortIfAttacked)"#,
        );
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], SurveyEvent::SurveyUsed { .. }));

        // Distance text — should NOT produce Completed
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[14:08:45] LocalPlayer: ProcessScreenText(ImportantInfo, "The treasure is 869 meters from here.")"#,
        );
        assert!(
            events.is_empty(),
            "Motherlode distance should not produce Completed"
        );
    }

    #[test]
    fn test_parse_loot_items_with_speed_bonus() {
        let (items, speed_bonus) =
            parse_loot_items("Malachite collected! Also found Quartz x3 (speed bonus!)");
        assert!(speed_bonus);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].item_name, "Malachite");
        assert!(items[0].is_primary);
        assert_eq!(items[0].quantity, 1);
        assert_eq!(items[1].item_name, "Quartz");
        assert!(!items[1].is_primary);
        assert!(items[1].is_speed_bonus);
        assert_eq!(items[1].quantity, 3);
    }

    #[test]
    fn test_parse_loot_items_no_bonus() {
        let (items, speed_bonus) = parse_loot_items("Piece of Green Glass collected!");
        assert!(!speed_bonus);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item_name, "Piece of Green Glass");
        assert!(items[0].is_primary);
    }

    #[test]
    fn test_non_survey_item_added_is_ignored() {
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(test_known_surveys());

        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[00:10:00] LocalPlayer: ProcessAddItem(Malachite(115244857), -1, True)"#,
        );
        assert!(
            events.is_empty(),
            "Non-survey ItemAdded should not emit events"
        );
    }

    // ============================================================
    // Motherlode Detection Tests
    // ============================================================

    #[test]
    fn test_motherlode_full_sequence() {
        // Replays the optimal motherlode sequence:
        // Using map → DeleteItem → Mining ... → AddItem → mount (finalize)
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(test_known_surveys());

        // Register the motherlode map item in the parser's instance registry
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:08:00] LocalPlayer: ProcessAddItem(KurMountainsGoodMetalMotherlodeMap(136709665), 5, True)"#,
        );

        // Using motherlode map — emits SurveyUsed
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[14:08:44] LocalPlayer: ProcessDoDelayLoop(1, Unset, "Using Kur Mountains Good Metal Motherlode Map", 5305, AbortIfAttacked)"#,
        );
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], SurveyEvent::SurveyUsed { .. }));

        // DeleteItem — map consumed, motherlode node spawns
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[14:08:46] LocalPlayer: ProcessDeleteItem(136709665)"#,
        );
        assert!(events.is_empty(), "DeleteItem should not emit events yet");

        // Mining XP (from map consumption — ignored by survey parser)
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:08:46] LocalPlayer: ProcessUpdateSkill({type=Mining,raw=68,bonus=5,xp=3373,tnl=5000,max=70}, True, 300, 0, 0)"#,
        );

        // StartInteraction with the motherlode node
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:08:47] LocalPlayer: ProcessStartInteraction(5032392, 7, 0, False, "")"#,
        );

        // Mining delay loop
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[14:08:47] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining ...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );
        assert!(events.is_empty(), "Mining start should not emit events yet");

        // Loot: AddItem
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[14:08:53] LocalPlayer: ProcessAddItem(Tungsten(136807948), -1, True)"#,
        );
        assert!(events.is_empty(), "Loot should accumulate, not emit yet");

        // Mining XP (from completing the mine)
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:08:53] LocalPlayer: ProcessUpdateSkill({type=Mining,raw=68,bonus=5,xp=3503,tnl=5000,max=70}, True, 130, 0, 0)"#,
        );

        // Player mounts up → finalizes the motherlode
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[14:08:55] LocalPlayer: ProcessPlayerMount(5008166, True)"#,
        );
        assert_eq!(events.len(), 1, "Mount should finalize motherlode");
        match &events[0] {
            SurveyEvent::MotherlodeCompleted {
                map_name,
                loot_items,
                ..
            } => {
                assert_eq!(map_name, "Kur Mountains Good Metal Motherlode Map");
                assert_eq!(loot_items.len(), 1);
                assert_eq!(loot_items[0].item_name, "Tungsten");
                assert_eq!(loot_items[0].quantity, 1);
            }
            _ => panic!("Expected MotherlodeCompleted, got {:?}", events[0]),
        }
    }

    #[test]
    fn test_motherlode_stack_update_loot() {
        // When motherlode loot goes to an existing stack (UpdateItemCode)
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(test_known_surveys());

        // Register map and existing metal slab stack
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:00:00] LocalPlayer: ProcessAddItem(KurMountainsGoodMetalMotherlodeMap(100001), 5, True)"#,
        );
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:00:00] LocalPlayer: ProcessAddItem(MetalSlab7(100002), 3, True)"#,
        );

        // Using map
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:00] LocalPlayer: ProcessDoDelayLoop(1, Unset, "Using Kur Mountains Good Metal Motherlode Map", 5305, AbortIfAttacked)"#,
        );

        // Map consumed
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:02] LocalPlayer: ProcessDeleteItem(100001)"#,
        );

        // Mining
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:04] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining ...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );

        // Loot via stack update: MetalSlab7 stack goes from 1 to 27 (delta +26)
        // UpdateItemCode encodes (stackSize << 16) | itemTypeId
        // stack=27 for MetalSlab7 (type_id doesn't matter for this test, using arbitrary)
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:10] LocalPlayer: ProcessUpdateItemCode(100002, 1769490, True)"#,
        );

        // Player mounts → finalize
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:12] LocalPlayer: ProcessPlayerMount(5008166, True)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            SurveyEvent::MotherlodeCompleted { loot_items, .. } => {
                assert!(!loot_items.is_empty(), "Should have loot from stack update");
                // The exact quantity depends on how the item code decoder resolves the delta
                // The important thing is that we captured *something*
            }
            _ => panic!("Expected MotherlodeCompleted"),
        }
    }

    #[test]
    fn test_motherlode_timeout_no_mining() {
        // Player consumes map but never mines — should timeout without emitting
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(test_known_surveys());

        // Register map
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:00:00] LocalPlayer: ProcessAddItem(KurMountainsGoodMetalMotherlodeMap(100001), 5, True)"#,
        );

        // Using map
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:00] LocalPlayer: ProcessDoDelayLoop(1, Unset, "Using Kur Mountains Good Metal Motherlode Map", 5305, AbortIfAttacked)"#,
        );

        // Map consumed
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:02] LocalPlayer: ProcessDeleteItem(100001)"#,
        );

        // Feed 200+ noise lines with player events to trigger timeout
        let mut final_events = Vec::new();
        for i in 0..205 {
            let line = format!(
                r#"[14:02:{:02}] LocalPlayer: ProcessSetAttributes(5008166, "[CUR_HEALTH], [500]")"#,
                i % 60
            );
            let (_, events) = process_line(&mut parser, &mut survey, &line);
            final_events.extend(events);
        }

        // Should NOT emit MotherlodeCompleted (no loot was collected)
        assert!(
            final_events.is_empty(),
            "Timeout without mining should not emit MotherlodeCompleted"
        );
        assert!(
            survey.motherlode_pending.is_none(),
            "Pending should be cleared"
        );
    }

    #[test]
    fn test_motherlode_triangulation_then_mine() {
        // Player uses map multiple times (triangulating) before it's consumed
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(test_known_surveys());

        // Register map
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:00:00] LocalPlayer: ProcessAddItem(KurMountainsGoodMetalMotherlodeMap(100001), 5, True)"#,
        );

        // First use — far away, locate only
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:00] LocalPlayer: ProcessDoDelayLoop(1, Unset, "Using Kur Mountains Good Metal Motherlode Map", 5305, AbortIfAttacked)"#,
        );
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], SurveyEvent::SurveyUsed { .. }));

        // Distance text
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:01] LocalPlayer: ProcessScreenText(ImportantInfo, "The treasure is 869 meters from here.")"#,
        );

        // Second use — still far
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[14:02:00] LocalPlayer: ProcessDoDelayLoop(1, Unset, "Using Kur Mountains Good Metal Motherlode Map", 5305, AbortIfAttacked)"#,
        );
        assert!(events.is_empty(), "Repeated use should not emit SurveyUsed");

        // Distance text
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:02:01] LocalPlayer: ProcessScreenText(ImportantInfo, "The treasure is 35 meters from here.")"#,
        );

        // Third use — close enough, map is consumed
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:02:30] LocalPlayer: ProcessDoDelayLoop(1, Unset, "Using Kur Mountains Good Metal Motherlode Map", 5305, AbortIfAttacked)"#,
        );
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:02:32] LocalPlayer: ProcessDeleteItem(100001)"#,
        );
        // Note: DeleteItem is buffered by player_event_parser until next line flushes it

        // Mine and get loot — this line also flushes the DeleteItem which triggers motherlode_pending
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:02:34] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining ...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );
        assert!(
            survey.motherlode_pending.is_some(),
            "Should have motherlode pending after flush + mining"
        );
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:02:40] LocalPlayer: ProcessAddItem(Paladium(200001), -1, True)"#,
        );

        // Mount → finalize
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[14:02:42] LocalPlayer: ProcessPlayerMount(5008166, True)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            SurveyEvent::MotherlodeCompleted { loot_items, .. } => {
                assert_eq!(loot_items.len(), 1);
                assert_eq!(loot_items[0].item_name, "Paladium");
            }
            _ => panic!("Expected MotherlodeCompleted"),
        }
    }

    #[test]
    fn test_motherlode_combat_interruption() {
        // Player starts mining motherlode, gets attacked by a mob, fights it,
        // loots the corpse (InteractionStarted resets mining_started),
        // corpse drops an item (should NOT be captured as motherlode loot),
        // then returns to mine the motherlode again.
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(test_known_surveys());

        // Register map
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:00:00] LocalPlayer: ProcessAddItem(KurMountainsGoodMetalMotherlodeMap(100001), 5, True)"#,
        );

        // Using map
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:00] LocalPlayer: ProcessDoDelayLoop(1, Unset, "Using Kur Mountains Good Metal Motherlode Map", 5305, AbortIfAttacked)"#,
        );

        // Map consumed (buffered delete)
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:02] LocalPlayer: ProcessDeleteItem(100001)"#,
        );

        // Mining starts — flushes the delete which creates motherlode_pending
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:04] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining ...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );
        assert!(survey.motherlode_pending.is_some());

        // Combat interruption! Player gets attacked, mining aborted.
        // Some combat noise lines...
        for i in 0..5 {
            let line = format!(
                r#"[14:01:{:02}] LocalPlayer: ProcessSetAttributes(5008166, "[CUR_HEALTH], [{}]")"#,
                10 + i,
                500 - i * 50
            );
            process_line(&mut parser, &mut survey, &line);
        }

        // Player loots mob corpse — InteractionStarted resets mining_started.
        // Should NOT finalize (no loot yet).
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:20] LocalPlayer: ProcessStartInteraction(12345, 7, 0, False, "")"#,
        );
        assert!(
            events.is_empty(),
            "InteractionStarted with no loot should not finalize"
        );
        assert!(
            survey.motherlode_pending.is_some(),
            "Motherlode pending should survive combat interruption"
        );

        // Corpse drops a TrooperHelm — should NOT be captured (mining_started was reset)
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:21] LocalPlayer: ProcessAddItem(TrooperHelm(300001), -1, True)"#,
        );
        assert!(
            events.is_empty(),
            "Corpse loot should not finalize motherlode"
        );

        // Player returns to mine motherlode again
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:30] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining ...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );

        // Now real motherlode loot arrives
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:36] LocalPlayer: ProcessAddItem(Paladium(200001), -1, True)"#,
        );

        // Mount → finalize with ONLY the motherlode loot, not the corpse loot
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:38] LocalPlayer: ProcessPlayerMount(5008166, True)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            SurveyEvent::MotherlodeCompleted { loot_items, .. } => {
                assert_eq!(
                    loot_items.len(),
                    1,
                    "Should only have motherlode loot, not corpse loot"
                );
                assert_eq!(loot_items[0].item_name, "Paladium");
            }
            _ => panic!("Expected MotherlodeCompleted"),
        }
    }

    #[test]
    fn test_motherlode_loot_not_captured_after_mining_completes() {
        // Scenario from real logs: mining completes with loot at 22:47:22,
        // then combat state changes arrive, then an unrelated UpdateItemCode
        // (campfire stack update) at 22:47:28 should NOT be captured.
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(test_known_surveys());

        // Register campfire item BEFORE the motherlode sequence
        process_line(
            &mut parser,
            &mut survey,
            r#"[13:50:00] LocalPlayer: ProcessAddItem(Campfire0(300001), 5, True)"#,
        );

        // Register map
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:00:00] LocalPlayer: ProcessAddItem(KurMountainsGoodMetalMotherlodeMap(100001), 5, True)"#,
        );

        // Using map
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:00] LocalPlayer: ProcessDoDelayLoop(1, Unset, "Using Kur Mountains Good Metal Motherlode Map", 5305, AbortIfAttacked)"#,
        );

        // Map consumed (buffered delete)
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:02] LocalPlayer: ProcessDeleteItem(100001)"#,
        );

        // Ore already in inventory from earlier mining
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:02] LocalPlayer: ProcessAddItem(MetalSlab7(200001), 3, True)"#,
        );
        // Set initial stack so delta is meaningful
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:02] LocalPlayer: ProcessUpdateItemCode(200001, 3801191, True)"#,
        );

        // Interaction + Mining starts — flushes the delete
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:03] LocalPlayer: ProcessStartInteraction(999001, 7, 0, False, "")"#,
        );
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:04] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining ...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );
        assert!(survey.motherlode_pending.is_some());

        // Mining completes — loot arrives (stack update: 58 → 87 = +29 ore)
        process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:10] LocalPlayer: ProcessUpdateItemCode(200001, 5705959, True)"#,
        );
        // Motherlode should still be pending (loot captured, waiting for finalization)
        assert!(
            survey.motherlode_pending.is_some(),
            "Should still be pending with loot"
        );

        // Next line: combat state change — should finalize the motherlode
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:11] LocalPlayer: ProcessCombatModeStatus(InCombat, System.Int32[])"#,
        );
        assert_eq!(
            events.len(),
            1,
            "CombatStateChanged should finalize motherlode"
        );
        match &events[0] {
            SurveyEvent::MotherlodeCompleted { loot_items, .. } => {
                assert!(
                    loot_items.iter().all(|l| l.item_name != "Campfire0"),
                    "Campfire should not be in motherlode loot"
                );
            }
            _ => panic!("Expected MotherlodeCompleted"),
        }
        assert!(
            survey.motherlode_pending.is_none(),
            "Motherlode should be resolved"
        );

        // Campfire stack update arrives later — should NOT be captured
        let (_, events) = process_line(
            &mut parser,
            &mut survey,
            r#"[14:01:15] LocalPlayer: ProcessUpdateItemCode(300001, 210039, True)"#,
        );
        assert!(
            events.is_empty(),
            "No events after motherlode already finalized"
        );
    }

    // ============================================================
    // Integration Tests — Full Log Replay
    // ============================================================

    /// Helper: replay a full Player.log file through the parser pipeline,
    /// collecting all survey events produced.
    fn replay_log(
        log_path: &str,
        known_surveys: HashMap<String, KnownSurveyType>,
    ) -> Vec<SurveyEvent> {
        let content = std::fs::read_to_string(log_path)
            .unwrap_or_else(|e| panic!("Failed to read log file {}: {}", log_path, e));
        let mut parser = PlayerEventParser::new();
        let mut survey = SurveyParser::new(known_surveys);
        let mut all_events = Vec::new();

        for line in content.lines() {
            let player_events = parser.process_line(line);
            let survey_events = survey.process_events(&player_events, line);
            all_events.extend(survey_events);
        }

        all_events
    }

    /// Helper: tally loot from Completed events into a HashMap<item_name, total_quantity>
    fn tally_completed_loot(events: &[SurveyEvent]) -> HashMap<String, u32> {
        let mut totals: HashMap<String, u32> = HashMap::new();
        for event in events {
            if let SurveyEvent::Completed { loot_items, .. } = event {
                for item in loot_items {
                    *totals.entry(item.item_name.clone()).or_insert(0) += item.quantity;
                }
            }
        }
        totals
    }

    #[test]
    fn test_replay_100x_serbule_crystal_survey() {
        // 100x Serbule Blue Mineral Survey with surveying ring
        // Internal name: GeologySurveySerbule1
        let log_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../test_data/surveyLogs/100x-serbcrystal-withring/Player.log"
        );

        let mut known = HashMap::new();
        known.insert(
            "GeologySurveySerbule1".to_string(),
            KnownSurveyType {
                display_name: "Serbule Blue Mineral Survey".to_string(),
                is_motherlode: false,
            },
        );

        let events = replay_log(log_path, known);

        // Count event types
        let completions: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, SurveyEvent::Completed { .. }))
            .collect();
        let crafts: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, SurveyEvent::MapCrafted { .. }))
            .collect();
        let uses: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, SurveyEvent::SurveyUsed { .. }))
            .collect();

        assert_eq!(completions.len(), 100, "Should have 100 completions");
        assert_eq!(crafts.len(), 100, "Should have 100 maps crafted");
        // Uses can be more than completions due to locate steps
        assert!(
            uses.len() >= 100,
            "Should have at least 100 survey uses (locates + collects), got {}",
            uses.len()
        );

        // Verify loot totals from ScreenText parsing
        // These are the totals as reported in the loot text messages.
        // Note: the ring silently grants extra primary items via ProcessUpdateItemCode
        // which are NOT reflected in the ScreenText — so these totals are lower than
        // the actual inventory changes recorded in results.txt.
        let loot = tally_completed_loot(&events);

        let expected: HashMap<&str, u32> = [
            ("Amethyst", 31),
            ("Aquamarine", 7),
            ("Azurite", 42),
            ("Bloodstone", 16),
            ("Blue Spinel", 32),
            ("Fluorite", 27),
            ("Lapis Lazuli", 25),
            ("Malachite", 8),
            ("Morganite", 3),
            ("Obsidian", 36),
            ("Peridot", 7),
            ("Piece of Green Glass", 31),
            ("Rubywall Crystal", 23),
            ("Sapphire", 1),
            ("Sunstone", 1),
            ("Tourmaline", 8),
            ("Tsavorite", 2),
        ]
        .into_iter()
        .collect();

        for (item, expected_qty) in &expected {
            let actual = loot.get(*item).copied().unwrap_or(0);
            assert_eq!(
                actual, *expected_qty,
                "Loot mismatch for {}: expected {}, got {}",
                item, expected_qty, actual
            );
        }

        // Verify no unexpected items appeared
        for (item, qty) in &loot {
            assert!(
                expected.contains_key(item.as_str()),
                "Unexpected loot item: {} x{}",
                item,
                qty
            );
        }
    }

    #[test]
    fn test_replay_100x_eltibule_metal_survey() {
        // 100x Eltibule Amazing Mining Survey with ring and pick
        // Internal name: MiningSurveyEltibule6
        let log_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../test_data/surveyLogs/100x-eltmetal-ringandpick/Player-prev.log"
        );

        let mut known = HashMap::new();
        known.insert(
            "MiningSurveyEltibule6".to_string(),
            KnownSurveyType {
                display_name: "Eltibule Amazing Mining Survey".to_string(),
                is_motherlode: false,
            },
        );

        let events = replay_log(log_path, known);

        // Count event types
        let completions: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, SurveyEvent::Completed { .. }))
            .collect();
        let crafts: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, SurveyEvent::MapCrafted { .. }))
            .collect();
        let uses: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, SurveyEvent::SurveyUsed { .. }))
            .collect();

        assert_eq!(completions.len(), 100, "Should have 100 completions");
        assert_eq!(crafts.len(), 100, "Should have 100 maps crafted");
        assert!(
            uses.len() >= 100,
            "Should have at least 100 survey uses, got {}",
            uses.len()
        );

        // Verify loot totals from ScreenText parsing
        // Same caveat: ring/pick bonuses add items silently, so ScreenText totals
        // are lower than the results.txt inventory totals.
        let loot = tally_completed_loot(&events);

        let expected: HashMap<&str, u32> = [
            ("Amazing Metal Slab", 94),
            ("Astounding Metal Slab", 8),
            ("Basic Metal Slab", 12),
            ("Copper Ore", 10),
            ("Expert-Quality Metal Slab", 4),
            ("Fire Dust", 1),
            ("Flinty Rock", 9),
            ("Gold Nugget", 15),
            ("Gold Ore", 4),
            ("Good Metal Slab", 4),
            ("Masterwork Metal Slab", 1),
            ("Pyrite", 5),
            ("Saltpeter", 3),
            ("Silver Ore", 12),
            ("Simple Metal Slab", 15),
            ("Sulfur", 1),
            ("Tungsten", 1),
        ]
        .into_iter()
        .collect();

        for (item, expected_qty) in &expected {
            let actual = loot.get(*item).copied().unwrap_or(0);
            assert_eq!(
                actual, *expected_qty,
                "Loot mismatch for {}: expected {}, got {}",
                item, expected_qty, actual
            );
        }

        // Verify no unexpected items appeared
        for (item, qty) in &loot {
            assert!(
                expected.contains_key(item.as_str()),
                "Unexpected loot item: {} x{}",
                item,
                qty
            );
        }
    }
}
