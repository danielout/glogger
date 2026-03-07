/// Events the survey parser can produce from a single line (or line transition)
#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "kind")]
pub enum SurveyEvent {
    /// A new survey run started (map crafted/consumed)
    SessionStart {
        timestamp: String,
        map_name: String,
    },
    /// Survey item used — got directions, not at the spot yet
    Located {
        timestamp: String,
        survey_name: String,
        direction_hint: String,
    },
    /// Survey item used and successfully mined
    Completed {
        timestamp: String,
        survey_name: String,
        loot_text: String,
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
            if let Some(hint) = parse_map_fx_hint(line) {
                // Got directions — this was a locate
                events.push(SurveyEvent::Located {
                    timestamp,
                    survey_name,
                    direction_hint: hint,
                });
                return events; // line consumed
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
                events.push(SurveyEvent::Completed {
                    timestamp,
                    survey_name,
                    loot_text: loot,
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
                events.push(SurveyEvent::Completed {
                    timestamp,
                    survey_name,
                    loot_text: loot,
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