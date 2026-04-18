// Pure parsing logic — no Tauri dependencies here

#[derive(Debug, serde::Serialize, Clone)]
pub struct SkillUpdate {
    pub skill_type: String,
    pub level: u32,
    pub bonus: u32,
    pub xp: u32,
    pub tnl: u32,
    pub timestamp: String, // e.g. "00:08:37"
}

pub fn parse_skill_update(line: &str) -> Option<SkillUpdate> {
    if !line.contains("ProcessUpdateSkill") {
        return None;
    }

    let timestamp = parse_timestamp(line)?;
    let skill_type = extract_field(line, "type=")?;
    let raw: u32 = extract_field(line, "raw=")?.parse().ok()?;
    let bonus: u32 = extract_field(line, "bonus=")?.parse().ok()?;
    let xp: u32 = extract_field(line, "xp=")?.parse().ok()?;
    let tnl: u32 = extract_field(line, "tnl=")?.parse().ok()?;

    Some(SkillUpdate {
        skill_type,
        level: raw + bonus,
        bonus,
        xp,
        tnl,
        timestamp,
    })
}

// Parses "[HH:MM:SS] " from the start of a line, returns "HH:MM:SS"
pub fn parse_timestamp(line: &str) -> Option<String> {
    let line = line.trim_start();
    if !line.starts_with('[') {
        return None;
    }
    let end = line.find(']')?;
    Some(line[1..end].to_string())
}

/// Convert a Player.log `HH:MM:SS` timestamp to a full UTC datetime string, using an
/// explicit UTC base date when provided. Live tailing passes `None` (today's UTC date);
/// replay and old-log reparse pass the date derived from the log's file / chat timestamps.
pub fn to_utc_datetime_with_base(time_str: &str, base_date: Option<chrono::NaiveDate>) -> String {
    use chrono::{NaiveTime, Utc};

    let date = base_date.unwrap_or_else(|| Utc::now().date_naive());

    if let Ok(utc_time) = NaiveTime::parse_from_str(time_str, "%H:%M:%S") {
        let utc_dt = date.and_time(utc_time);
        utc_dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

/// Convert a Chat.log local-time NaiveDateTime to UTC by subtracting the timezone offset.
///
/// Chat.log timestamps are in the player's local timezone. The offset (seconds east of UTC)
/// is parsed from the chat login line's "Timezone Offset" field.
pub fn chat_local_to_utc(local_dt: chrono::NaiveDateTime, tz_offset_seconds: i32) -> chrono::NaiveDateTime {
    use chrono::Duration;
    local_dt - Duration::seconds(tz_offset_seconds as i64)
}

// Extracts the value after `key` up to the next comma or `}`
pub fn extract_field(line: &str, key: &str) -> Option<String> {
    let start = line.find(key)? + key.len();
    let rest = &line[start..];
    let end = rest.find(|c| c == ',' || c == '}').unwrap_or(rest.len());
    Some(rest[..end].to_string())
}

// ============================================================
// Loot Text Parsing (reusable across features)
// ============================================================
//
// Used by the survey aggregator to extract speed-bonus item markers from
// ProcessScreenText lines like:
//   "Blue Spinel collected! Also found Rubywall Crystal x2 (speed bonus!)"

/// Individual loot item parsed from screen text
#[derive(Debug, serde::Serialize, Clone)]
pub struct LootItem {
    pub item_name: String,
    pub quantity: u32,
    pub is_speed_bonus: bool,
    pub is_primary: bool,
}

/// Parse loot text into individual items.
/// Example: "Tsavorite collected! Also found Moss Agate x2, Onyx (speed bonus!)"
/// Returns (Vec<LootItem>, speed_bonus_earned)
pub fn parse_loot_items(loot_text: &str) -> (Vec<LootItem>, bool) {
    let mut items = Vec::new();
    let speed_bonus_earned = loot_text.contains("(speed bonus!)");

    // Primary item: everything before " collected!"
    // May include quantity for ring bonus, e.g. "Lapis Lazuli x2 collected!"
    if let Some(collected_idx) = loot_text.find(" collected!") {
        let primary_text = loot_text[..collected_idx].trim();
        if let Some((name, qty)) = parse_item_with_quantity(primary_text) {
            items.push(LootItem {
                item_name: name,
                quantity: qty,
                is_speed_bonus: false,
                is_primary: true,
            });
        }
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
pub fn parse_item_with_quantity(text: &str) -> Option<(String, u32)> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_loot_simple() {
        let (items, bonus) = parse_loot_items("Fluorite collected!");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item_name, "Fluorite");
        assert_eq!(items[0].quantity, 1);
        assert!(items[0].is_primary);
        assert!(!bonus);
    }

    #[test]
    fn test_parse_loot_with_speed_bonus() {
        let (items, bonus) =
            parse_loot_items("Blue Spinel collected! Also found Rubywall Crystal x2 (speed bonus!)");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].item_name, "Blue Spinel");
        assert_eq!(items[0].quantity, 1);
        assert!(items[0].is_primary);
        assert_eq!(items[1].item_name, "Rubywall Crystal");
        assert_eq!(items[1].quantity, 2);
        assert!(items[1].is_speed_bonus);
        assert!(bonus);
    }

    #[test]
    fn test_parse_loot_primary_with_quantity() {
        // Ring bonus can grant extra primary items: "Lapis Lazuli x2 collected!"
        let (items, bonus) =
            parse_loot_items("Lapis Lazuli x2 collected! Also found Azurite (speed bonus!)");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].item_name, "Lapis Lazuli");
        assert_eq!(items[0].quantity, 2);
        assert!(items[0].is_primary);
        assert_eq!(items[1].item_name, "Azurite");
        assert_eq!(items[1].quantity, 1);
        assert!(items[1].is_speed_bonus);
        assert!(bonus);
    }

    #[test]
    fn test_parse_loot_primary_quantity_no_bonus() {
        let (items, bonus) = parse_loot_items("Obsidian x2 collected!");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item_name, "Obsidian");
        assert_eq!(items[0].quantity, 2);
        assert!(items[0].is_primary);
        assert!(!bonus);
    }
}
