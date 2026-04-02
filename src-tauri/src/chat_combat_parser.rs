/// Chat Combat channel parser — detects player deaths and incoming damage
/// from [Combat] messages.
///
/// Stateless parser: each message maps to 0 or 1 events.
use crate::chat_parser::ChatMessage;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind")]
pub enum ChatCombatEvent {
    /// An enemy killed the player character.
    /// Detected via "(FATALITY!)" suffix where the target matches the active character.
    PlayerDeath {
        timestamp: String,
        killer_name: String,
        killer_entity_id: String,
        killing_ability: String,
        health_damage: u32,
        armor_damage: u32,
    },
    /// An enemy dealt damage to the player (non-fatal).
    DamageOnPlayer {
        timestamp: String,
        attacker_name: String,
        attacker_entity_id: String,
        ability_name: String,
        health_damage: u32,
        armor_damage: u32,
        is_crit: bool,
    },
}

/// Try to parse a Combat channel ChatMessage into a combat event.
/// Returns None if the message is not a Combat message or not relevant to the player.
pub fn parse_combat_message(
    msg: &ChatMessage,
    player_name: &str,
) -> Option<ChatCombatEvent> {
    if msg.channel.as_deref() != Some("Combat") {
        return None;
    }

    let text = msg.message.trim();
    let ts = msg.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();

    // Try FATALITY first (most specific)
    if text.ends_with("(FATALITY!)") {
        if let Some(event) = try_player_death(text, &ts, player_name) {
            return Some(event);
        }
    }

    // Try non-fatal damage on player
    try_damage_on_player(text, &ts, player_name)
}

/// Parse: "EnemyName #ID: AbilityName on PlayerName! Dmg: N health, N armor. (FATALITY!)"
fn try_player_death(text: &str, ts: &str, player_name: &str) -> Option<ChatCombatEvent> {
    if !text.ends_with("(FATALITY!)") {
        return None;
    }

    let parsed = parse_attack_line(text, player_name)?;
    let dmg_text = parsed.dmg_text?;

    // Strip trailing ". (FATALITY!)" or " (FATALITY!)"
    let dmg_text = dmg_text
        .strip_suffix("(FATALITY!)")
        .unwrap_or(dmg_text)
        .trim()
        .trim_end_matches('.');
    let dmg_text = dmg_text.trim();

    let (health_damage, armor_damage) = parse_damage(dmg_text)?;

    Some(ChatCombatEvent::PlayerDeath {
        timestamp: ts.to_string(),
        killer_name: parsed.attacker_name.to_string(),
        killer_entity_id: parsed.entity_id.to_string(),
        killing_ability: parsed.ability_name.to_string(),
        health_damage,
        armor_damage,
    })
}

/// Parse non-fatal damage: "EnemyName #ID: AbilityName on PlayerName! Dmg: N health, N armor"
fn try_damage_on_player(text: &str, ts: &str, player_name: &str) -> Option<ChatCombatEvent> {
    let parsed = parse_attack_line(text, player_name)?;

    // Must have damage text — lines ending in "(EVADED!)" won't have "Dmg:"
    let dmg_text = parsed.dmg_text?;
    let dmg_text = dmg_text.trim().trim_end_matches('.');
    let dmg_text = dmg_text.trim();

    let (health_damage, armor_damage) = parse_damage(dmg_text)?;

    // Skip zero-damage hits (Dmg: none) — not useful in the damage log
    if health_damage == 0 && armor_damage == 0 {
        return None;
    }

    Some(ChatCombatEvent::DamageOnPlayer {
        timestamp: ts.to_string(),
        attacker_name: parsed.attacker_name.to_string(),
        attacker_entity_id: parsed.entity_id.to_string(),
        ability_name: parsed.ability_name.to_string(),
        health_damage,
        armor_damage,
        is_crit: parsed.is_crit,
    })
}

/// Common fields extracted from an attack line.
struct ParsedAttackLine<'a> {
    attacker_name: &'a str,
    entity_id: &'a str,
    ability_name: &'a str,
    is_crit: bool,
    /// Text after "Dmg: " if present. None for EVADED or lines without damage.
    dmg_text: Option<&'a str>,
}

/// Parse the common structure of an attack line:
/// "ActorName #EntityID: AbilityName on TargetName! Dmg: ..."
/// "ActorName #EntityID: AbilityName on TargetName (CRIT!) Dmg: ..."
/// "ActorName #EntityID: AbilityName on TargetName (EVADED!)"
///
/// Returns None if the target doesn't match player_name or the format is wrong.
fn parse_attack_line<'a>(text: &'a str, player_name: &str) -> Option<ParsedAttackLine<'a>> {
    let hash_pos = find_entity_id_separator(text)?;
    let attacker_name = &text[..hash_pos];

    let after_hash = &text[hash_pos + 2..];
    let colon_pos = after_hash.find(": ")?;
    let entity_id = &after_hash[..colon_pos];

    if !entity_id.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let after_colon = &after_hash[colon_pos + 2..];
    let on_pos = after_colon.find(" on ")?;
    let ability_name = &after_colon[..on_pos];

    let after_on = &after_colon[on_pos + 4..];

    let target_end = after_on.find('!')?;
    let target_name_part = &after_on[..target_end];

    let is_crit = target_name_part.ends_with(" (CRIT");
    let target_name = target_name_part
        .strip_suffix(" (CRIT")
        .unwrap_or(target_name_part)
        .trim();

    if target_name != player_name {
        return None;
    }

    let dmg_marker = "Dmg: ";
    let dmg_text = after_on.find(dmg_marker).map(|pos| &after_on[pos + dmg_marker.len()..]);

    Some(ParsedAttackLine {
        attacker_name,
        entity_id,
        ability_name,
        is_crit,
        dmg_text,
    })
}

/// Find the position of " #" that separates entity name from entity ID.
/// Must be followed by digits. Returns the position of the space before "#".
fn find_entity_id_separator(text: &str) -> Option<usize> {
    let mut search_from = 0;
    while let Some(pos) = text[search_from..].find(" #") {
        let abs_pos = search_from + pos;
        let after = &text[abs_pos + 2..];
        if after.starts_with(|c: char| c.is_ascii_digit()) {
            return Some(abs_pos);
        }
        search_from = abs_pos + 2;
    }
    None
}

/// Parse damage text like "N health, N armor" or "N health" or " none"
fn parse_damage(text: &str) -> Option<(u32, u32)> {
    let text = text.trim();

    if text == "none" {
        return Some((0, 0));
    }

    let parts: Vec<&str> = text.split(", ").collect();

    let health = parse_damage_component(parts.first()?)?;
    let armor = if parts.len() > 1 {
        parse_damage_component(parts.get(1)?)?
    } else {
        0
    };

    Some((health, armor))
}

/// Parse a single damage component like "N health" or "N armor"
fn parse_damage_component(text: &str) -> Option<u32> {
    let text = text.trim();
    let space_pos = text.find(' ')?;
    text[..space_pos].parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_parser::parse_chat_line;

    fn combat_msg(text: &str) -> ChatMessage {
        let line = format!("26-04-01 05:30:53\t[Combat] {}", text);
        parse_chat_line(&line).unwrap()
    }

    // ── PlayerDeath tests ───────────────────────────────────────────────────

    #[test]
    fn test_player_death_basic() {
        let msg = combat_msg(
            "Demon Scout #7059135: Demon Bolt on Zenith! Dmg: 179 health, 178 armor. (FATALITY!)",
        );
        let event = parse_combat_message(&msg, "Zenith").unwrap();
        let ChatCombatEvent::PlayerDeath {
            killer_name,
            killer_entity_id,
            killing_ability,
            health_damage,
            armor_damage,
            ..
        } = event
        else {
            panic!("Expected PlayerDeath");
        };
        assert_eq!(killer_name, "Demon Scout");
        assert_eq!(killer_entity_id, "7059135");
        assert_eq!(killing_ability, "Demon Bolt");
        assert_eq!(health_damage, 179);
        assert_eq!(armor_damage, 178);
    }

    #[test]
    fn test_player_death_kuvou() {
        let msg = combat_msg(
            "Kuvou #7681191: Bash on Zenith! Dmg: 108 health, 11 armor. (FATALITY!)",
        );
        let event = parse_combat_message(&msg, "Zenith").unwrap();
        let ChatCombatEvent::PlayerDeath {
            killer_name,
            killing_ability,
            health_damage,
            armor_damage,
            ..
        } = event
        else {
            panic!("Expected PlayerDeath");
        };
        assert_eq!(killer_name, "Kuvou");
        assert_eq!(killing_ability, "Bash");
        assert_eq!(health_damage, 108);
        assert_eq!(armor_damage, 11);
    }

    #[test]
    fn test_player_death_behemoth() {
        let msg = combat_msg(
            "Behemoth #6323710: Infernal Slam on Zenith! Dmg: 201 health, 201 armor. (FATALITY!)",
        );
        let event = parse_combat_message(&msg, "Zenith").unwrap();
        let ChatCombatEvent::PlayerDeath {
            killer_name,
            killing_ability,
            ..
        } = event
        else {
            panic!("Expected PlayerDeath");
        };
        assert_eq!(killer_name, "Behemoth");
        assert_eq!(killing_ability, "Infernal Slam");
    }

    #[test]
    fn test_player_death_long_enemy_name() {
        let msg = combat_msg(
            "Demonic Equine Analyst #8004000: Demon Bolt on Zenith! Dmg: 174 health, 173 armor. (FATALITY!)",
        );
        let event = parse_combat_message(&msg, "Zenith").unwrap();
        let ChatCombatEvent::PlayerDeath {
            killer_name,
            killer_entity_id,
            ..
        } = event
        else {
            panic!("Expected PlayerDeath");
        };
        assert_eq!(killer_name, "Demonic Equine Analyst");
        assert_eq!(killer_entity_id, "8004000");
    }

    #[test]
    fn test_mob_fatality_ignored() {
        let msg = combat_msg(
            "Zenith: Rib Shatter 9 on Orcish Ranger #7442047! Dmg: 3790 health, 1741 armor. (FATALITY!)",
        );
        assert!(parse_combat_message(&msg, "Zenith").is_none());
    }

    #[test]
    fn test_non_combat_channel_ignored() {
        let line = "26-04-01 05:30:52\t[Status] You earned 5 XP in Dying.";
        let msg = parse_chat_line(&line).unwrap();
        assert!(parse_combat_message(&msg, "Zenith").is_none());
    }

    #[test]
    fn test_different_player_fatality_ignored() {
        let msg = combat_msg(
            "Demon Scout #7059135: Demon Bolt on SomeOtherPlayer! Dmg: 179 health, 178 armor. (FATALITY!)",
        );
        assert!(parse_combat_message(&msg, "Zenith").is_none());
    }

    #[test]
    fn test_health_only_fatality() {
        let msg = combat_msg(
            "Demon Scout #7059135: Tracking Stare on Zenith! Dmg: 144 health. (FATALITY!)",
        );
        let event = parse_combat_message(&msg, "Zenith").unwrap();
        let ChatCombatEvent::PlayerDeath {
            health_damage,
            armor_damage,
            ..
        } = event
        else {
            panic!("Expected PlayerDeath");
        };
        assert_eq!(health_damage, 144);
        assert_eq!(armor_damage, 0);
    }

    #[test]
    fn test_recovered_message_ignored() {
        let msg = combat_msg("Zenith: Recovered: 21 health");
        assert!(parse_combat_message(&msg, "Zenith").is_none());
    }

    #[test]
    fn test_self_buff_ignored() {
        let msg = combat_msg("Zenith: Psi Health Wave 7 on Zenith!");
        assert!(parse_combat_message(&msg, "Zenith").is_none());
    }

    // ── DamageOnPlayer tests ────────────────────────────────────────────────

    #[test]
    fn test_damage_on_player_basic() {
        let msg = combat_msg(
            "Demon Scout #6937326: Demon Bolt on Zenith! Dmg: 171 health, 171 armor",
        );
        let event = parse_combat_message(&msg, "Zenith").unwrap();
        let ChatCombatEvent::DamageOnPlayer {
            attacker_name,
            attacker_entity_id,
            ability_name,
            health_damage,
            armor_damage,
            is_crit,
            ..
        } = event
        else {
            panic!("Expected DamageOnPlayer, got {:?}", event);
        };
        assert_eq!(attacker_name, "Demon Scout");
        assert_eq!(attacker_entity_id, "6937326");
        assert_eq!(ability_name, "Demon Bolt");
        assert_eq!(health_damage, 171);
        assert_eq!(armor_damage, 171);
        assert!(!is_crit);
    }

    #[test]
    fn test_damage_on_player_health_only() {
        let msg = combat_msg(
            "Demon Scout #6937326: Tracking Stare on Zenith! Dmg: 144 health",
        );
        let event = parse_combat_message(&msg, "Zenith").unwrap();
        let ChatCombatEvent::DamageOnPlayer {
            health_damage,
            armor_damage,
            ..
        } = event
        else {
            panic!("Expected DamageOnPlayer");
        };
        assert_eq!(health_damage, 144);
        assert_eq!(armor_damage, 0);
    }

    #[test]
    fn test_damage_on_player_crit() {
        let msg = combat_msg(
            "Brain Demon #7463620: Bite on Zenith (CRIT!) Dmg: 205 health, 205 armor",
        );
        let event = parse_combat_message(&msg, "Zenith").unwrap();
        let ChatCombatEvent::DamageOnPlayer {
            attacker_name,
            ability_name,
            health_damage,
            armor_damage,
            is_crit,
            ..
        } = event
        else {
            panic!("Expected DamageOnPlayer");
        };
        assert_eq!(attacker_name, "Brain Demon");
        assert_eq!(ability_name, "Bite");
        assert_eq!(health_damage, 205);
        assert_eq!(armor_damage, 205);
        assert!(is_crit);
    }

    #[test]
    fn test_evaded_attack_ignored() {
        let msg = combat_msg(
            "Demon Scout #6937326: Demon Bolt on Zenith (EVADED!)",
        );
        assert!(parse_combat_message(&msg, "Zenith").is_none());
    }

    #[test]
    fn test_none_damage_ignored() {
        let msg = combat_msg(
            "Ranalon Guardian #7681185: Ranalon Guardian Stab on Zenith! Dmg:  none",
        );
        assert!(parse_combat_message(&msg, "Zenith").is_none());
    }

    #[test]
    fn test_damage_on_different_player_ignored() {
        let msg = combat_msg(
            "Demon Scout #6937326: Demon Bolt on OtherPlayer! Dmg: 171 health, 171 armor",
        );
        assert!(parse_combat_message(&msg, "Zenith").is_none());
    }

    #[test]
    fn test_player_attack_on_mob_ignored() {
        let msg = combat_msg(
            "Zenith: Rib Shatter 9 on Orcish Ranger #7442047! Dmg: 3790 health, 1741 armor",
        );
        assert!(parse_combat_message(&msg, "Zenith").is_none());
    }

    #[test]
    fn test_indirect_damage_ignored() {
        let msg = combat_msg(
            "Demonic Swarmer #7537641: Suffered indirect dmg: -2 health",
        );
        assert!(parse_combat_message(&msg, "Zenith").is_none());
    }
}
