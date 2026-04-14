use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;

// ── Entry splitting ────────────────────────────────────────────────────────

/// Matches a timestamp prefix like "Sat Mar 28 15:39 - "
static ENTRY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)([A-Z][a-z]{2} [A-Z][a-z]{2} \d{1,2} \d{1,2}:\d{2}) - ").unwrap()
});

// ── Per-action regexes ─────────────────────────────────────────────────────

static BOUGHT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?P<player>\S+) bought (?P<item>.+?) ?x?(?P<quantity>\d+)? at a cost of (?P<price_unit>\d+) per (?P<quantity_unit>\d+) = (?P<price_total>\d+)$").unwrap()
});

static ADDED_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?P<player>\S+) added (?P<item>.+?) ?x?(?P<quantity>\d+)? to shop$").unwrap()
});

static REMOVED_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?P<player>\S+) removed (?P<item>.+?) ?x?(?P<quantity>\d+)? from shop$").unwrap()
});

static CONFIGURED_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?P<player>\S+) configured (?P<item>.+?) ?x?(?P<quantity>\d+)? to cost (?P<price_unit>\d+) per (?P<quantity_unit>\d+)\.?\s*(?P<rest>.*)$").unwrap()
});

static VISIBLE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?P<player>\S+) made (?P<item>.+?) ?x?(?P<quantity>\d+)? visible in shop at a cost of (?P<price_unit>\d+) per (?P<quantity_unit>\d+)\.?\s*(?P<rest>.*)$").unwrap()
});

static COLLECTED_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?P<player>\S+) collected (?P<price_total>\d+) Councils from customer purchases$").unwrap()
});

// ── Data structures ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ShopLogEntry {
    pub entry_index: i64,
    pub timestamp: String,
    pub event_at: Option<String>,
    pub action: String,
    pub player: String,
    pub item: Option<String>,
    pub quantity: i64,
    pub price_unit: Option<f64>,
    pub price_total: Option<i64>,
    pub raw_message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShopLog {
    pub log_timestamp: String,
    pub title: String,
    pub entries: Vec<ShopLogEntry>,
    pub owner: Option<String>,
}

// ── Owner action check ─────────────────────────────────────────────────────

fn is_owner_action(action: &str) -> bool {
    matches!(action, "added" | "removed" | "configured" | "visible" | "collected")
}

// ── Parsing ────────────────────────────────────────────────────────────────

fn parse_entry(entry_index: i64, timestamp: &str, message: &str) -> ShopLogEntry {
    let message = message.trim();

    // Try bought
    if let Some(caps) = BOUGHT_RE.captures(message) {
        let quantity: i64 = caps.name("quantity")
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(1);
        let price_unit: i64 = caps["price_unit"].parse().unwrap_or(0);
        let quantity_unit: i64 = caps["quantity_unit"].parse().unwrap_or(1);
        let price_total: i64 = caps["price_total"].parse().unwrap_or(0);

        return ShopLogEntry {
            entry_index,
            timestamp: timestamp.to_string(),
            event_at: None,
            action: "bought".to_string(),
            player: caps["player"].to_string(),
            item: Some(caps["item"].to_string()),
            quantity,
            price_unit: Some(price_unit as f64 / quantity_unit as f64),
            price_total: Some(price_total),
            raw_message: message.to_string(),
        };
    }

    // Try added
    if let Some(caps) = ADDED_RE.captures(message) {
        let quantity: i64 = caps.name("quantity")
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(1);
        return ShopLogEntry {
            entry_index,
            timestamp: timestamp.to_string(),
            event_at: None,
            action: "added".to_string(),
            player: caps["player"].to_string(),
            item: Some(caps["item"].to_string()),
            quantity,
            price_unit: None,
            price_total: None,
            raw_message: message.to_string(),
        };
    }

    // Try removed
    if let Some(caps) = REMOVED_RE.captures(message) {
        let quantity: i64 = caps.name("quantity")
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(1);
        return ShopLogEntry {
            entry_index,
            timestamp: timestamp.to_string(),
            event_at: None,
            action: "removed".to_string(),
            player: caps["player"].to_string(),
            item: Some(caps["item"].to_string()),
            quantity,
            price_unit: None,
            price_total: None,
            raw_message: message.to_string(),
        };
    }

    // Try configured
    if let Some(caps) = CONFIGURED_RE.captures(message) {
        let quantity: i64 = caps.name("quantity")
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(1);
        let price_unit: i64 = caps["price_unit"].parse().unwrap_or(0);
        let quantity_unit: i64 = caps["quantity_unit"].parse().unwrap_or(1);
        let effective_price = price_unit as f64 / quantity_unit as f64;

        return ShopLogEntry {
            entry_index,
            timestamp: timestamp.to_string(),
            event_at: None,
            action: "configured".to_string(),
            player: caps["player"].to_string(),
            item: Some(caps["item"].to_string()),
            quantity,
            price_unit: Some(effective_price),
            price_total: None,
            raw_message: message.to_string(),
        };
    }

    // Try visible (made visible)
    if let Some(caps) = VISIBLE_RE.captures(message) {
        let quantity: i64 = caps.name("quantity")
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(1);
        let price_unit: i64 = caps["price_unit"].parse().unwrap_or(0);
        let quantity_unit: i64 = caps["quantity_unit"].parse().unwrap_or(1);
        let effective_price = price_unit as f64 / quantity_unit as f64;

        return ShopLogEntry {
            entry_index,
            timestamp: timestamp.to_string(),
            event_at: None,
            action: "visible".to_string(),
            player: caps["player"].to_string(),
            item: Some(caps["item"].to_string()),
            quantity,
            price_unit: Some(effective_price),
            price_total: None,
            raw_message: message.to_string(),
        };
    }

    // Try collected
    if let Some(caps) = COLLECTED_RE.captures(message) {
        let price_total: i64 = caps["price_total"].parse().unwrap_or(0);
        return ShopLogEntry {
            entry_index,
            timestamp: timestamp.to_string(),
            event_at: None,
            action: "collected".to_string(),
            player: caps["player"].to_string(),
            item: None,
            quantity: 1,
            price_unit: None,
            price_total: Some(price_total),
            raw_message: message.to_string(),
        };
    }

    // Unknown
    ShopLogEntry {
        entry_index,
        timestamp: timestamp.to_string(),
        event_at: None,
        action: "unknown".to_string(),
        player: String::new(),
        item: None,
        quantity: 1,
        price_unit: None,
        price_total: None,
        raw_message: message.to_string(),
    }
}

/// Parse the content of a BookOpened event with book_type "PlayerShopLog".
///
/// The content contains newline-separated entries, each prefixed with a
/// timestamp like "Sat Mar 28 15:39 - message text".
pub fn parse_shop_log(
    title: &str,
    content: &str,
    log_timestamp: &str,
    base_year: i32,
) -> ShopLog {
    // Unescape the content (it comes from ProcessBook with escaped newlines)
    let content = content.replace("\\n", "\n");

    let mut entries = Vec::new();

    // Find all timestamp matches and split content into (timestamp, message) pairs
    let matches: Vec<_> = ENTRY_RE.find_iter(&content).collect();

    // Collect raw (timestamp, message) pairs in content order (newest first)
    let mut raw_entries: Vec<(&str, &str)> = Vec::new();
    for (i, m) in matches.iter().enumerate() {
        let timestamp_str = &content[m.start()..m.end()];
        let timestamp = timestamp_str.trim_end_matches(" - ").trim();

        let msg_start = m.end();
        let msg_end = if i + 1 < matches.len() {
            matches[i + 1].start()
        } else {
            content.len()
        };

        let message = content[msg_start..msg_end].trim();
        if !message.is_empty() {
            raw_entries.push((timestamp, message));
        }
    }

    // Reverse so oldest entry gets index 0. This ensures stable indices
    // when new entries are prepended to the log on subsequent opens.
    raw_entries.reverse();
    for (i, (timestamp, message)) in raw_entries.iter().enumerate() {
        entries.push(parse_entry(i as i64, timestamp, message));
    }

    // Resolve real ISO timestamps from the year-less game format. Entries are
    // already oldest-first, so the monotonic resolver walks forward correctly.
    let timestamps: Vec<&str> = entries.iter().map(|e| e.timestamp.as_str()).collect();
    let resolved = crate::stall_year_resolver::resolve_timestamps_oldest_first(&timestamps, base_year);
    for (entry, event_at) in entries.iter_mut().zip(resolved) {
        entry.event_at = event_at;
    }

    // Detect owner from first owner action
    let owner = entries.iter()
        .find(|e| is_owner_action(&e.action))
        .map(|e| e.player.clone());

    ShopLog {
        log_timestamp: log_timestamp.to_string(),
        title: title.to_string(),
        entries,
        owner,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bought() {
        let entry = parse_entry(0, "Sat Mar 28 15:09", "MrBonq bought Quality Reins at a cost of 4500 per 1 = 4500");
        assert_eq!(entry.action, "bought");
        assert_eq!(entry.player, "MrBonq");
        assert_eq!(entry.item, Some("Quality Reins".to_string()));
        assert_eq!(entry.quantity, 1);
        assert_eq!(entry.price_unit, Some(4500.0));
        assert_eq!(entry.price_total, Some(4500));
    }

    #[test]
    fn test_parse_bought_with_quantity() {
        let entry = parse_entry(0, "Tue Apr 7 16:25", "Zangariel bought Orcish Spell Pouch x12 at a cost of 450 per 1 = 5400");
        assert_eq!(entry.action, "bought");
        assert_eq!(entry.player, "Zangariel");
        assert_eq!(entry.item, Some("Orcish Spell Pouch".to_string()));
        assert_eq!(entry.quantity, 12);
        assert_eq!(entry.price_unit, Some(450.0));
        assert_eq!(entry.price_total, Some(5400));
    }

    #[test]
    fn test_parse_added() {
        let entry = parse_entry(0, "Sat Mar 28 15:39", "Deradon added Quality Mystic Saddlebag to shop");
        assert_eq!(entry.action, "added");
        assert_eq!(entry.player, "Deradon");
        assert_eq!(entry.item, Some("Quality Mystic Saddlebag".to_string()));
    }

    #[test]
    fn test_parse_added_with_quantity() {
        let entry = parse_entry(0, "Sat Mar 28 15:39", "Deradon added Barley Seeds x36 to shop");
        assert_eq!(entry.action, "added");
        assert_eq!(entry.item, Some("Barley Seeds".to_string()));
        assert_eq!(entry.quantity, 36);
    }

    #[test]
    fn test_parse_removed() {
        let entry = parse_entry(0, "Sat Mar 28 15:39", "Deradon removed Decent Horseshoes from shop");
        assert_eq!(entry.action, "removed");
        assert_eq!(entry.player, "Deradon");
        assert_eq!(entry.item, Some("Decent Horseshoes".to_string()));
        assert_eq!(entry.quantity, 1);
    }

    #[test]
    fn test_parse_removed_with_quantity() {
        let entry = parse_entry(0, "Sat Mar 28 15:39", "Deradon removed Barley Seeds x36 from shop");
        assert_eq!(entry.action, "removed");
        assert_eq!(entry.item, Some("Barley Seeds".to_string()));
        assert_eq!(entry.quantity, 36);
    }

    #[test]
    fn test_parse_configured() {
        let entry = parse_entry(0, "Sat Mar 28 13:30", "Deradon configured Decent Horseshoes to cost 3500 per 1");
        assert_eq!(entry.action, "configured");
        assert_eq!(entry.player, "Deradon");
        assert_eq!(entry.item, Some("Decent Horseshoes".to_string()));
        assert_eq!(entry.price_unit, Some(3500.0));
    }

    #[test]
    fn test_parse_configured_with_bulk_and_restriction() {
        let entry = parse_entry(0, "Sat Mar 28 13:30", "Deradon configured Barley Seedsx36 to cost 3000 per 2. Item can only be purchased by Wogan.");
        assert_eq!(entry.action, "configured");
        assert_eq!(entry.item, Some("Barley Seeds".to_string()));
        assert_eq!(entry.quantity, 36);
        assert_eq!(entry.price_unit, Some(1500.0)); // 3000 / 2
    }

    #[test]
    fn test_parse_visible() {
        let entry = parse_entry(0, "Sat Mar 28 15:38", "Deradon made Nice Saddle visible in shop at a cost of 4000 per 1");
        assert_eq!(entry.action, "visible");
        assert_eq!(entry.player, "Deradon");
        assert_eq!(entry.item, Some("Nice Saddle".to_string()));
        assert_eq!(entry.price_unit, Some(4000.0));
    }

    #[test]
    fn test_parse_visible_with_space_before_quantity() {
        let entry = parse_entry(0, "Sat Mar 28 13:30", "Deradon made Aquamarine x49 visible in shop at a cost of 750 per 1");
        assert_eq!(entry.action, "visible");
        assert_eq!(entry.item, Some("Aquamarine".to_string()));
        assert_eq!(entry.quantity, 49);
        assert_eq!(entry.price_unit, Some(750.0));
    }

    #[test]
    fn test_parse_configured_with_space_before_quantity() {
        let entry = parse_entry(0, "Sat Mar 28 13:30", "Deradon configured Aquamarine x49 to cost 750 per 1");
        assert_eq!(entry.action, "configured");
        assert_eq!(entry.item, Some("Aquamarine".to_string()));
        assert_eq!(entry.quantity, 49);
        assert_eq!(entry.price_unit, Some(750.0));
    }

    #[test]
    fn test_parse_visible_with_bulk_and_restriction() {
        let entry = parse_entry(0, "Sat Mar 28 13:30", "Deradon made Barley Seedsx36 visible in shop at a cost of 3000 per 2. Item can only be purchased by Wogan.");
        assert_eq!(entry.action, "visible");
        assert_eq!(entry.item, Some("Barley Seeds".to_string()));
        assert_eq!(entry.quantity, 36);
        assert_eq!(entry.price_unit, Some(1500.0)); // 3000 / 2
    }

    #[test]
    fn test_parse_collected() {
        let entry = parse_entry(0, "Sat Mar 28 14:13", "Deradon collected 30500 Councils from customer purchases");
        assert_eq!(entry.action, "collected");
        assert_eq!(entry.player, "Deradon");
        assert_eq!(entry.item, None);
        assert_eq!(entry.price_total, Some(30500));
    }

    #[test]
    fn test_parse_unknown() {
        let entry = parse_entry(0, "Sat Mar 28 05:21", "Deradon paid 14100 Councils to hire Gluzzibab for another 24 hours. Paid hours remaining = 35");
        assert_eq!(entry.action, "unknown");
        assert!(entry.raw_message.contains("paid 14100"));
    }

    #[test]
    fn test_parse_shop_log_full() {
        let content = "Sat Mar 28 15:09 - MrBonq bought Quality Reins at a cost of 4500 per 1 = 4500\n\nSat Mar 28 15:08 - AlestiarWolf bought Quality Mystic Saddlebag at a cost of 6000 per 1 = 6000\n\nSat Mar 28 14:13 - Deradon collected 30500 Councils from customer purchases\n\n";
        let log = parse_shop_log("Today's Shop Logs", content, "19:40:40", 2026);

        assert_eq!(log.title, "Today's Shop Logs");
        assert_eq!(log.log_timestamp, "19:40:40");
        assert_eq!(log.entries.len(), 3);
        assert_eq!(log.owner, Some("Deradon".to_string()));

        // Entries are reversed: oldest (14:13) first, newest (15:09) last
        assert_eq!(log.entries[0].action, "collected");
        assert_eq!(log.entries[0].player, "Deradon");
        assert_eq!(log.entries[0].entry_index, 0);
        assert_eq!(log.entries[1].action, "bought");
        assert_eq!(log.entries[1].player, "AlestiarWolf");
        assert_eq!(log.entries[1].entry_index, 1);
        assert_eq!(log.entries[2].action, "bought");
        assert_eq!(log.entries[2].player, "MrBonq");
        assert_eq!(log.entries[2].entry_index, 2);
    }

    #[test]
    fn test_parse_shop_log_with_escaped_newlines() {
        let content = r"Sat Mar 28 15:09 - MrBonq bought Quality Reins at a cost of 4500 per 1 = 4500\n\nSat Mar 28 14:13 - Deradon collected 30500 Councils from customer purchases\n\n";
        let log = parse_shop_log("Yesterday's Shop Logs", content, "13:26:04", 2026);

        assert_eq!(log.entries.len(), 2);
        // Reversed: oldest first
        assert_eq!(log.entries[0].action, "collected");
        assert_eq!(log.entries[1].action, "bought");
    }

    #[test]
    fn test_duplicate_entries_get_different_indices() {
        // Two identical purchases in the same minute should get different entry_index values
        let content = "Sat Mar 28 15:09 - Kork bought Nice Saddle at a cost of 4000 per 1 = 4000\n\nSat Mar 28 15:09 - Kork bought Nice Saddle at a cost of 4000 per 1 = 4000\n\n";
        let log = parse_shop_log("Today's Shop Logs", content, "19:40:40", 2026);

        assert_eq!(log.entries.len(), 2);
        assert_eq!(log.entries[0].raw_message, log.entries[1].raw_message);
        assert_eq!(log.entries[0].timestamp, log.entries[1].timestamp);
        assert_ne!(log.entries[0].entry_index, log.entries[1].entry_index);
        assert_eq!(log.entries[0].entry_index, 0);
        assert_eq!(log.entries[1].entry_index, 1);
    }

    #[test]
    fn test_entry_indices_stable_across_reopens() {
        // First open: two entries (newest first in content)
        let content1 = "Sat Mar 28 15:09 - MrBonq bought Quality Reins at a cost of 4500 per 1 = 4500\n\nSat Mar 28 14:13 - Deradon collected 30500 Councils from customer purchases\n\n";
        let log1 = parse_shop_log("Today's Shop Logs", content1, "16:00:00", 2026);

        // Second open: new entry prepended, same old entries follow
        let content2 = "Sat Mar 28 16:30 - Kork bought Nice Saddle at a cost of 4000 per 1 = 4000\n\nSat Mar 28 15:09 - MrBonq bought Quality Reins at a cost of 4500 per 1 = 4500\n\nSat Mar 28 14:13 - Deradon collected 30500 Councils from customer purchases\n\n";
        let log2 = parse_shop_log("Today's Shop Logs", content2, "17:00:00", 2026);

        // Old entries should keep the same indices across both opens
        assert_eq!(log1.entries[0].entry_index, log2.entries[0].entry_index); // collected = index 0
        assert_eq!(log1.entries[0].raw_message, log2.entries[0].raw_message);
        assert_eq!(log1.entries[1].entry_index, log2.entries[1].entry_index); // MrBonq = index 1
        assert_eq!(log1.entries[1].raw_message, log2.entries[1].raw_message);
        // New entry gets the highest index
        assert_eq!(log2.entries[2].entry_index, 2); // Kork = index 2
    }
}
