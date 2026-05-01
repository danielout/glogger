/// Chat Status channel parser — converts [Status] messages into structured events.
///
/// Stateless parser: each message maps to 0 or 1 events.
/// Accumulation and cross-stream correlation are left to subscribing features.
use crate::chat_parser::ChatMessage;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind")]
pub enum ChatStatusEvent {
    /// "X added to inventory" / "X xN added to inventory"
    ItemGained {
        timestamp: String,
        item_name: String,
        quantity: u32,
    },

    /// "You earned N XP in Skill."
    XpGained {
        timestamp: String,
        skill: String,
        amount: u32,
    },

    /// "You earned N XP and reached level L in Skill!"
    LevelUp {
        timestamp: String,
        skill: String,
        level: u32,
        xp: u32,
    },

    /// "You searched the corpse and found N coins."
    CoinsLooted { timestamp: String, amount: u32 },

    /// "You received N Councils." / "You used N councils."
    CouncilsChanged { timestamp: String, amount: i64 },

    /// "The treasure is N meters from here."
    TreasureDistance { timestamp: String, meters: u32 },

    /// "You bury the corpse." / "You botch the autopsy!"
    AnatomyResult { timestamp: String, success: bool },

    /// "Summoned X xN"
    Summoned {
        timestamp: String,
        item_name: String,
        quantity: u32,
    },

    /// "CrudBurst's Hammer of Thumping carefully studied!"
    ItemStudied {
        timestamp: String,
        item_name: String,
    },

    /// "Saved report to C:/.../Reports/Character_Foo.json"
    /// Fired when the player runs /exportcharacter or /outputitems.
    ReportSaved {
        timestamp: String,
        file_path: String,
    },
}

/// Try to parse a Status channel ChatMessage into a structured event.
/// Returns None if the message doesn't match any known pattern.
pub fn parse_status_message(msg: &ChatMessage) -> Option<ChatStatusEvent> {
    // Only process Status channel messages
    if msg.channel.as_deref() != Some("Status") {
        return None;
    }

    let text = msg.message.trim();
    let ts = msg.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();

    // Try each pattern in order of frequency/importance
    try_item_gained(text, &ts)
        .or_else(|| try_xp_gained(text, &ts))
        .or_else(|| try_level_up(text, &ts))
        .or_else(|| try_treasure_distance(text, &ts))
        .or_else(|| try_anatomy_result(text, &ts))
        .or_else(|| try_coins_looted(text, &ts))
        .or_else(|| try_councils_changed(text, &ts))
        .or_else(|| try_summoned(text, &ts))
        .or_else(|| try_item_studied(text, &ts))
        .or_else(|| try_report_saved(text, &ts))
}

/// "X added to inventory." or "X xN added to inventory."
fn try_item_gained(text: &str, ts: &str) -> Option<ChatStatusEvent> {
    let suffix = " added to inventory.";
    if !text.ends_with(suffix) {
        return None;
    }
    let prefix = &text[..text.len() - suffix.len()];

    // Check for "ItemName xN" pattern
    let (item_name, quantity) = parse_name_and_quantity(prefix);

    Some(ChatStatusEvent::ItemGained {
        timestamp: ts.to_string(),
        item_name: item_name.to_string(),
        quantity,
    })
}

/// "You earned N XP in Skill." (but NOT the level-up variant)
fn try_xp_gained(text: &str, ts: &str) -> Option<ChatStatusEvent> {
    // Must start with "You earned " and end with a period (not !)
    if !text.starts_with("You earned ") || !text.ends_with('.') {
        return None;
    }
    // Exclude level-up messages (they end with "!")
    if text.contains("reached level") {
        return None;
    }

    // "You earned 62 XP in Endurance."
    let inner = &text["You earned ".len()..text.len() - 1]; // strip trailing "."
    let xp_pos = inner.find(" XP in ")?;
    let amount: u32 = inner[..xp_pos].parse().ok()?;
    let skill = &inner[xp_pos + " XP in ".len()..];

    Some(ChatStatusEvent::XpGained {
        timestamp: ts.to_string(),
        skill: skill.to_string(),
        amount,
    })
}

/// "You earned N XP and reached level L in Skill!"
fn try_level_up(text: &str, ts: &str) -> Option<ChatStatusEvent> {
    if !text.starts_with("You earned ") || !text.ends_with('!') {
        return None;
    }
    if !text.contains("reached level") {
        return None;
    }

    // "You earned 192 XP and reached level 87 in Cartography!"
    let inner = &text["You earned ".len()..text.len() - 1]; // strip "!"
    let xp_pos = inner.find(" XP and reached level ")?;
    let xp: u32 = inner[..xp_pos].parse().ok()?;

    let after_level = &inner[xp_pos + " XP and reached level ".len()..];
    let level_end = after_level.find(" in ")?;
    let level: u32 = after_level[..level_end].parse().ok()?;
    let skill = &after_level[level_end + " in ".len()..];

    Some(ChatStatusEvent::LevelUp {
        timestamp: ts.to_string(),
        skill: skill.to_string(),
        level,
        xp,
    })
}

/// "The treasure is N meters from here."
fn try_treasure_distance(text: &str, ts: &str) -> Option<ChatStatusEvent> {
    if !text.starts_with("The treasure is ") || !text.ends_with(" meters from here.") {
        return None;
    }
    let inner = &text["The treasure is ".len()..text.len() - " meters from here.".len()];
    let meters: u32 = inner.parse().ok()?;

    Some(ChatStatusEvent::TreasureDistance {
        timestamp: ts.to_string(),
        meters,
    })
}

/// "You bury the corpse." or "You botch the autopsy!"
fn try_anatomy_result(text: &str, ts: &str) -> Option<ChatStatusEvent> {
    if text == "You bury the corpse." {
        return Some(ChatStatusEvent::AnatomyResult {
            timestamp: ts.to_string(),
            success: true,
        });
    }
    if text == "You botch the autopsy!" {
        return Some(ChatStatusEvent::AnatomyResult {
            timestamp: ts.to_string(),
            success: false,
        });
    }
    None
}

/// "You searched the corpse and found N coins."
fn try_coins_looted(text: &str, ts: &str) -> Option<ChatStatusEvent> {
    if !text.starts_with("You searched the corpse and found ") || !text.ends_with(" coins.") {
        return None;
    }
    let inner = &text["You searched the corpse and found ".len()..text.len() - " coins.".len()];
    let amount: u32 = inner.parse().ok()?;

    Some(ChatStatusEvent::CoinsLooted {
        timestamp: ts.to_string(),
        amount,
    })
}

/// "You received N Councils." / "You used N councils."
fn try_councils_changed(text: &str, ts: &str) -> Option<ChatStatusEvent> {
    if text.starts_with("You received ") && text.ends_with(" Councils.") {
        let inner = &text["You received ".len()..text.len() - " Councils.".len()];
        let amount: i64 = inner.replace(',', "").parse().ok()?;
        return Some(ChatStatusEvent::CouncilsChanged {
            timestamp: ts.to_string(),
            amount,
        });
    }
    if text.starts_with("You used ") && text.ends_with(" councils.") {
        let inner = &text["You used ".len()..text.len() - " councils.".len()];
        let amount: i64 = inner.replace(',', "").parse().ok()?;
        return Some(ChatStatusEvent::CouncilsChanged {
            timestamp: ts.to_string(),
            amount: -amount,
        });
    }
    None
}

/// "Summoned X xN"
fn try_summoned(text: &str, ts: &str) -> Option<ChatStatusEvent> {
    if !text.starts_with("Summoned ") {
        return None;
    }
    let rest = &text["Summoned ".len()..];
    let (item_name, quantity) = parse_name_and_quantity(rest);

    Some(ChatStatusEvent::Summoned {
        timestamp: ts.to_string(),
        item_name: item_name.to_string(),
        quantity,
    })
}

/// "CrudBurst's Hammer of Thumping carefully studied!"
/// Hoplology equipment study — fires when a player studies a piece of equipment.
/// The 5-minute cooldown between studies is tracked on the frontend.
fn try_item_studied(text: &str, ts: &str) -> Option<ChatStatusEvent> {
    let suffix = " carefully studied!";
    if !text.ends_with(suffix) {
        return None;
    }
    let item_name = &text[..text.len() - suffix.len()];
    if item_name.is_empty() {
        return None;
    }

    Some(ChatStatusEvent::ItemStudied {
        timestamp: ts.to_string(),
        item_name: item_name.to_string(),
    })
}

/// "Saved report to C:/.../Reports/Character_Foo.json"
fn try_report_saved(text: &str, ts: &str) -> Option<ChatStatusEvent> {
    let prefix = "Saved report to ";
    if !text.starts_with(prefix) {
        return None;
    }
    let file_path = &text[prefix.len()..];
    if file_path.is_empty() {
        return None;
    }

    Some(ChatStatusEvent::ReportSaved {
        timestamp: ts.to_string(),
        file_path: file_path.to_string(),
    })
}

/// Parse "ItemName xN" or "ItemName" → (name, quantity).
/// Returns quantity=1 if no "xN" suffix is found.
fn parse_name_and_quantity(text: &str) -> (&str, u32) {
    // Look for " xN" at the end where N is one or more digits
    if let Some(x_pos) = text.rfind(" x") {
        let after_x = &text[x_pos + 2..];
        if let Ok(qty) = after_x.parse::<u32>() {
            return (&text[..x_pos], qty);
        }
    }
    (text, 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_parser::parse_chat_line;

    fn status_msg(text: &str) -> ChatMessage {
        let line = format!("26-03-26 15:12:48\t[Status] {}", text);
        parse_chat_line(&line).unwrap()
    }

    #[test]
    fn test_item_gained_no_quantity() {
        let msg = status_msg("Tundra Rye Seeds added to inventory.");
        let event = parse_status_message(&msg).unwrap();
        if let ChatStatusEvent::ItemGained {
            item_name,
            quantity,
            ..
        } = event
        {
            assert_eq!(item_name, "Tundra Rye Seeds");
            assert_eq!(quantity, 1);
        } else {
            panic!("Expected ItemGained, got {:?}", event);
        }
    }

    #[test]
    fn test_item_gained_with_quantity() {
        let msg = status_msg("Astounding Metal Slab x26 added to inventory.");
        let event = parse_status_message(&msg).unwrap();
        if let ChatStatusEvent::ItemGained {
            item_name,
            quantity,
            ..
        } = event
        {
            assert_eq!(item_name, "Astounding Metal Slab");
            assert_eq!(quantity, 26);
        } else {
            panic!("Expected ItemGained, got {:?}", event);
        }
    }

    #[test]
    fn test_item_gained_gypsum() {
        let msg = status_msg("Gypsum x9 added to inventory.");
        let event = parse_status_message(&msg).unwrap();
        if let ChatStatusEvent::ItemGained {
            item_name,
            quantity,
            ..
        } = event
        {
            assert_eq!(item_name, "Gypsum");
            assert_eq!(quantity, 9);
        } else {
            panic!("Expected ItemGained, got {:?}", event);
        }
    }

    #[test]
    fn test_xp_gained() {
        let msg = status_msg("You earned 62 XP in Endurance.");
        let event = parse_status_message(&msg).unwrap();
        if let ChatStatusEvent::XpGained { skill, amount, .. } = event {
            assert_eq!(skill, "Endurance");
            assert_eq!(amount, 62);
        } else {
            panic!("Expected XpGained, got {:?}", event);
        }
    }

    #[test]
    fn test_xp_gained_multi_word_skill() {
        let msg = status_msg("You earned 67 XP in Canine Anatomy.");
        let event = parse_status_message(&msg).unwrap();
        if let ChatStatusEvent::XpGained { skill, amount, .. } = event {
            assert_eq!(skill, "Canine Anatomy");
            assert_eq!(amount, 67);
        } else {
            panic!("Expected XpGained, got {:?}", event);
        }
    }

    #[test]
    fn test_level_up() {
        let msg = status_msg("You earned 192 XP and reached level 87 in Cartography!");
        let event = parse_status_message(&msg).unwrap();
        if let ChatStatusEvent::LevelUp {
            skill, level, xp, ..
        } = event
        {
            assert_eq!(skill, "Cartography");
            assert_eq!(level, 87);
            assert_eq!(xp, 192);
        } else {
            panic!("Expected LevelUp, got {:?}", event);
        }
    }

    #[test]
    fn test_treasure_distance() {
        let msg = status_msg("The treasure is 3215 meters from here.");
        let event = parse_status_message(&msg).unwrap();
        if let ChatStatusEvent::TreasureDistance { meters, .. } = event {
            assert_eq!(meters, 3215);
        } else {
            panic!("Expected TreasureDistance, got {:?}", event);
        }
    }

    #[test]
    fn test_anatomy_success() {
        let msg = status_msg("You bury the corpse.");
        let event = parse_status_message(&msg).unwrap();
        if let ChatStatusEvent::AnatomyResult { success, .. } = event {
            assert!(success);
        } else {
            panic!("Expected AnatomyResult, got {:?}", event);
        }
    }

    #[test]
    fn test_anatomy_failure() {
        let msg = status_msg("You botch the autopsy!");
        let event = parse_status_message(&msg).unwrap();
        if let ChatStatusEvent::AnatomyResult { success, .. } = event {
            assert!(!success);
        } else {
            panic!("Expected AnatomyResult, got {:?}", event);
        }
    }

    #[test]
    fn test_summoned() {
        let msg = status_msg("Summoned Nice Phlogiston x5");
        let event = parse_status_message(&msg).unwrap();
        if let ChatStatusEvent::Summoned {
            item_name,
            quantity,
            ..
        } = event
        {
            assert_eq!(item_name, "Nice Phlogiston");
            assert_eq!(quantity, 5);
        } else {
            panic!("Expected Summoned, got {:?}", event);
        }
    }

    #[test]
    fn test_councils_received() {
        let msg = status_msg("You received 500 Councils.");
        let event = parse_status_message(&msg).unwrap();
        if let ChatStatusEvent::CouncilsChanged { amount, .. } = event {
            assert_eq!(amount, 500);
        } else {
            panic!("Expected CouncilsChanged, got {:?}", event);
        }
    }

    #[test]
    fn test_councils_used() {
        let msg = status_msg("You used 200 councils.");
        let event = parse_status_message(&msg).unwrap();
        if let ChatStatusEvent::CouncilsChanged { amount, .. } = event {
            assert_eq!(amount, -200);
        } else {
            panic!("Expected CouncilsChanged, got {:?}", event);
        }
    }

    #[test]
    fn test_coins_looted() {
        let msg = status_msg("You searched the corpse and found 42 coins.");
        let event = parse_status_message(&msg).unwrap();
        if let ChatStatusEvent::CoinsLooted { amount, .. } = event {
            assert_eq!(amount, 42);
        } else {
            panic!("Expected CoinsLooted, got {:?}", event);
        }
    }

    #[test]
    fn test_item_studied() {
        let msg = status_msg("Moldy Ancient Shoes carefully studied!");
        let event = parse_status_message(&msg).unwrap();
        if let ChatStatusEvent::ItemStudied { item_name, .. } = event {
            assert_eq!(item_name, "Moldy Ancient Shoes");
        } else {
            panic!("Expected ItemStudied, got {:?}", event);
        }
    }

    #[test]
    fn test_item_studied_long_name() {
        let msg = status_msg("CrudBurst's Hammer of Thumping of Hammering carefully studied!");
        let event = parse_status_message(&msg).unwrap();
        if let ChatStatusEvent::ItemStudied { item_name, .. } = event {
            assert_eq!(item_name, "CrudBurst's Hammer of Thumping of Hammering");
        } else {
            panic!("Expected ItemStudied, got {:?}", event);
        }
    }

    #[test]
    fn test_report_saved_character() {
        let msg = status_msg("Saved report to C:/Users/TestUser/AppData/LocalLow/Elder Game/Project Gorgon/Reports/Character_TestPlayer_Dreva.json");
        let event = parse_status_message(&msg).unwrap();
        if let ChatStatusEvent::ReportSaved { file_path, .. } = event {
            assert_eq!(file_path, "C:/Users/TestUser/AppData/LocalLow/Elder Game/Project Gorgon/Reports/Character_TestPlayer_Dreva.json");
        } else {
            panic!("Expected ReportSaved, got {:?}", event);
        }
    }

    #[test]
    fn test_report_saved_inventory() {
        let msg = status_msg("Saved report to C:/Users/TestUser/AppData/LocalLow/Elder Game/Project Gorgon/Reports/TestPlayer_Dreva_items_2026-04-20-01-53-45Z.json");
        let event = parse_status_message(&msg).unwrap();
        if let ChatStatusEvent::ReportSaved { file_path, .. } = event {
            assert!(file_path.contains("items_"));
        } else {
            panic!("Expected ReportSaved, got {:?}", event);
        }
    }

    #[test]
    fn test_book_saved_not_report() {
        // "Saved book to ..." should NOT match ReportSaved
        let msg = status_msg("Saved book to C:/Users/TestUser/AppData/LocalLow/Elder Game/Project Gorgon/Books/HelpScreen_260419_185352.txt");
        assert!(parse_status_message(&msg).is_none());
    }

    #[test]
    fn test_non_status_ignored() {
        let line = "26-03-09 05:01:46\t[Global] Player: hello";
        let msg = parse_chat_line(&line).unwrap();
        assert!(parse_status_message(&msg).is_none());
    }

    #[test]
    fn test_unrecognized_status_returns_none() {
        let msg = status_msg("You have 3 friends online.");
        assert!(parse_status_message(&msg).is_none());
    }

    #[test]
    fn test_joined_chat_room_returns_none() {
        let msg = status_msg("Joined chat room \"Trade\". There are 280 other users here.");
        assert!(parse_status_message(&msg).is_none());
    }
}
