// Pure parsing logic — no Tauri dependencies here

#[derive(Debug, serde::Serialize, Clone)]
pub struct SkillUpdate {
    pub skill_type: String,
    pub level: u32,
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
    let level: u32 = extract_field(line, "raw=")?.parse().ok()?;
    let xp: u32 = extract_field(line, "xp=")?.parse().ok()?;
    let tnl: u32 = extract_field(line, "tnl=")?.parse().ok()?;

    Some(SkillUpdate {
        skill_type,
        level,
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

/// Convert a Player.log time-only timestamp ("HH:MM:SS") to a full UTC datetime string.
///
/// Player.log timestamps are local time with no date. We combine with today's UTC date
/// and apply the timezone offset to produce a UTC datetime.
///
/// `tz_offset_seconds`: offset from UTC in seconds (e.g., -25200 for UTC-7).
/// The log time is local, so UTC = local_time - offset.
pub fn to_utc_datetime(time_str: &str, tz_offset_seconds: i32) -> String {
    use chrono::{Duration, NaiveTime, Utc};

    let today = Utc::now().date_naive();

    // Parse the HH:MM:SS time string
    if let Ok(local_time) = NaiveTime::parse_from_str(time_str, "%H:%M:%S") {
        let local_dt = today.and_time(local_time);
        // UTC = local - offset (offset is seconds east of UTC)
        let utc_dt = local_dt - Duration::seconds(tz_offset_seconds as i64);
        utc_dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        // Fallback: use current UTC time if parsing fails
        Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }
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
