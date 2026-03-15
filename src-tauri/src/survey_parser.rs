/// Individual loot item
#[derive(serde::Serialize, Clone, Debug)]
pub struct LootItem {
    pub item_name: String,
    pub quantity: u32,
    pub is_speed_bonus: bool,
    pub is_primary: bool,
}

/// Events the survey parser can produce from a single line (or line transition)
#[derive(serde::Serialize, Clone, Debug)]

#[serde(tag = "kind")]
pub enum SurveyEvent {
    /// A new survey run started (map crafted/consumed)
    SessionStart {
        timestamp: String,
        map_name: String,
    },
    /// Survey item used and successfully mined
    Completed {
        timestamp: String,
        survey_name: String,
        loot_items: Vec<LootItem>,
        speed_bonus_earned: bool,
    },
}

/// Pending state while we wait to see whether a "Using ... Survey" resolves
/// to a locate or a completion
#[derive(Debug)]
enum Pending {
    UsingSurvey {
        timestamp: String,
        survey_name: String,
    },
}

pub struct SurveyParser {
    pending: Option<Pending>,
}

impl SurveyParser {
    pub fn new() -> Self {
        Self { pending: None }
    }

    /// Feed one log line; returns zero or more events
    pub fn process_line(&mut self, line: &str) -> Vec<SurveyEvent> {
        let mut events = Vec::new();

        // --- Check if we can resolve a pending state ---
        if let Some(Pending::UsingSurvey { timestamp, survey_name }) = self.pending.take() {
            if parse_map_fx_hint(line).is_some() {
                // Got directions — this was a locate (we don't track these anymore)
                // Just consume the line and don't generate an event
                return events;
            } else if line.contains("ProcessDeleteItem") {
                // Survey item deleted — this was a completion; loot text comes shortly after
                // Store back as a different pending to wait for ProcessScreenText
                self.pending = Some(Pending::UsingSurvey {
                    timestamp,
                    survey_name,
                });
                // Fall through so we also check this line for other events below
            } else if let Some(loot) = parse_screen_text_loot(line) {
                // Got loot text — survey completed
                let (loot_items, speed_bonus_earned) = parse_loot_items(&loot);
                events.push(SurveyEvent::Completed {
                    timestamp,
                    survey_name,
                    loot_items,
                    speed_bonus_earned,
                });
                return events;
            } else {
                // Unrelated line — put pending back and continue
                self.pending = Some(Pending::UsingSurvey { timestamp, survey_name });
            }
        }

        // --- Detect new events ---

        // Survey session start: ProcessDoDelayLoop with "Surveying" action
        if let Some(event) = parse_session_start(line) {
            events.push(event);
            return events;
        }

        // Survey being used: ProcessDoDelayLoop with "Using ... Survey"
        if let Some((ts, name)) = parse_using_survey(line) {
            self.pending = Some(Pending::UsingSurvey {
                timestamp: ts,
                survey_name: name,
            });
            return events;
        }

        // Loot line while pending completion
        if let Some(loot) = parse_screen_text_loot(line) {
            // This can arrive after ProcessDeleteItem sets pending back
            if let Some(Pending::UsingSurvey { timestamp, survey_name }) = self.pending.take() {
                let (loot_items, speed_bonus_earned) = parse_loot_items(&loot);
                events.push(SurveyEvent::Completed {
                    timestamp,
                    survey_name,
                    loot_items,
                    speed_bonus_earned,
                });
            }
        }

        events
    }
}

/// [HH:MM:SS] LocalPlayer: ProcessDoDelayLoop(5, UseTeleportationCircle, "Surveying", ...)
fn parse_session_start(line: &str) -> Option<SurveyEvent> {
    if !line.contains("ProcessDoDelayLoop") || !line.contains("\"Surveying\"") {
        return None;
    }
    let ts = crate::parsers::parse_timestamp(line)?;
    // Extract map name from the 4th argument (the numeric zone ID isn't useful;
    // we'll just label it by timestamp for now — map name isn't in this line)
    Some(SurveyEvent::SessionStart {
        timestamp: ts,
        map_name: "Survey Run".to_string(),
    })
}

/// [HH:MM:SS] LocalPlayer: ProcessDoDelayLoop(0.5, Unset, "Using Eltibule Green Mineral Survey", ...)
fn parse_using_survey(line: &str) -> Option<(String, String)> {
    if !line.contains("ProcessDoDelayLoop") {
        return None;
    }
    // Extract the quoted string — it should start with "Using " and end with "Survey"
    let q_start = line.find('"')? + 1;
    let rest = &line[q_start..];
    let q_end = rest.find('"')?;
    let label = &rest[..q_end];
    if !label.starts_with("Using ") || !label.ends_with("Survey") {
        return None;
    }
    let ts = crate::parsers::parse_timestamp(line)?;
    Some((ts, label.to_string()))
}

/// ProcessMapFx(..., "Peridot is here", ImportantInfo, "The Peridot is 1386m east...")
fn parse_map_fx_hint(line: &str) -> Option<String> {
    if !line.contains("ProcessMapFx") {
        return None;
    }
    // Grab the last quoted string (the direction text)
    let last_q = line.rfind('"')?;
    let before = &line[..last_q];
    let open_q = before.rfind('"')? + 1;
    Some(line[open_q..last_q].to_string())
}

/// ProcessScreenText(ImportantInfo, "Tsavorite collected! Also found ...")
fn parse_screen_text_loot(line: &str) -> Option<String> {
    if !line.contains("ProcessScreenText") || !line.contains("ImportantInfo") {
        return None;
    }
    let q_start = line.rfind('"').map(|i| {
        let before = &line[..i];
        before.rfind('"').map(|j| j + 1)
    })??;
    let q_end = line.rfind('"')?;
    if q_start >= q_end {
        return None;
    }
    Some(line[q_start..q_end].to_string())
}

/// Parse loot text into individual items
/// Example: "Tsavorite collected! Also found Moss Agate x2, Onyx (speed bonus!)"
/// Returns (Vec<LootItem>, speed_bonus_earned)
fn parse_loot_items(loot_text: &str) -> (Vec<LootItem>, bool) {
    let mut items = Vec::new();
    let speed_bonus_earned = loot_text.contains("(speed bonus!)");

    // Primary item: everything before " collected!"
    if let Some(collected_idx) = loot_text.find(" collected!") {
        let primary_name = loot_text[..collected_idx].trim();
        items.push(LootItem {
            item_name: primary_name.to_string(),
            quantity: 1,
            is_speed_bonus: false,
            is_primary: true,
        });
    }

    // Secondary items: after "Also found "
    if let Some(also_idx) = loot_text.find("Also found ") {
        // Strip trailing "(speed bonus!)" or similar parentheticals
        let mut secondary = &loot_text[also_idx + "Also found ".len()..];

        // Check if these are speed bonus items
        let is_bonus = if let Some(bonus_idx) = secondary.find(" (speed bonus!)") {
            secondary = &secondary[..bonus_idx];
            true
        } else {
            // Remove any other parentheticals
            if let Some(paren_idx) = secondary.find(" (") {
                secondary = &secondary[..paren_idx];
            }
            false
        };

        // Split on commas and parse each item
        for part in secondary.split(',') {
            let piece = part.trim();
            if piece.is_empty() {
                continue;
            }

            // Match "Item Name x3" or just "Item Name"
            if let Some((name, qty)) = parse_item_with_quantity(piece) {
                items.push(LootItem {
                    item_name: name,
                    quantity: qty,
                    is_speed_bonus: is_bonus,
                    is_primary: false,
                });
            }
        }
    }

    (items, speed_bonus_earned)
}

/// Parse "Item Name x3" or "Item Name" into (name, quantity)
fn parse_item_with_quantity(text: &str) -> Option<(String, u32)> {
    // Match "Item Name x3"
    if let Some(x_pos) = text.rfind(" x") {
        let name = text[..x_pos].trim();
        let qty_str = &text[x_pos + 2..];
        if let Ok(qty) = qty_str.parse::<u32>() {
            return Some((name.to_string(), qty));
        }
    }
    // Just "Item Name"
    Some((text.to_string(), 1))
}