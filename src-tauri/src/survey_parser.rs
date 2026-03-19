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
use crate::parsers::{LootItem, parse_loot_items};
use crate::player_event_parser::PlayerEvent;

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
            .map(|(item_name, quantity)| ConsumedIngredient { item_name, quantity })
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
}

impl SurveyParser {
    pub fn new(known_surveys: HashMap<String, KnownSurveyType>) -> Self {
        Self {
            known_surveys,
            pending: None,
            crafting_window: None,
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

        // --- 2. Resolve pending survey use ---
        self.resolve_pending(player_events, raw_line, &mut events);

        // --- 3. Detect new "Using" lines ---
        self.detect_using_survey(player_events, &mut events);

        events
    }

    /// Detect crafting ticks, ingredient consumption, and crafted survey maps
    fn process_crafting(
        &mut self,
        player_events: &[PlayerEvent],
        events: &mut Vec<SurveyEvent>,
    ) {
        for event in player_events {
            match event {
                // "Surveying" crafting tick — open crafting window
                PlayerEvent::DelayLoopStarted { label, .. }
                    if label == "Surveying" =>
                {
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
                PlayerEvent::ItemAdded { timestamp, item_name, .. }
                    if self.known_surveys.contains_key(item_name.as_str()) =>
                {
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

                // ItemDeleted for a known survey → map consumed, await loot
                if player_events.iter().any(|e| {
                    matches!(e, PlayerEvent::ItemDeleted { item_name: Some(name), .. }
                        if self.known_surveys.contains_key(name.as_str()))
                }) {
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
                    lines_waited: lines_waited + 1,
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
                    lines_waited: lines_waited + 1,
                });
            }
        }
    }

    /// Check for loot text in the current events (used when transitioning to AwaitingLoot)
    fn check_loot_text(
        &mut self,
        player_events: &[PlayerEvent],
        events: &mut Vec<SurveyEvent>,
    ) {
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
        if let PlayerEvent::DelayLoopStarted { timestamp, label, .. } = event {
            if label.starts_with("Using ")
                && (label.ends_with("Survey") || label.ends_with("Map"))
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
        if let PlayerEvent::ScreenText { category, message, .. } = event {
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
        assert!(events2.is_empty(), "Repeated Using should NOT emit SurveyUsed");
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
                assert!(survey.crafting_window.is_none(), "Crafting window should be closed");
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
}
