/// Chat [Action Emotes] parser — detects resuscitate events from action emote messages.
///
/// Parses two patterns from the [Action Emotes] channel:
/// - `"CasterName resuscitates TargetName"` (successful rez)
/// - `"CasterName futilely attempts to resuscitate TargetName"` (failed rez)
///
/// Note: [Action Emotes] messages have no colon separator, so the chat parser
/// treats them as system messages with `sender: None`. The caster and target
/// names are extracted from the full message text.
///
/// Stateless parser: each message maps to 0 or 1 events.
use crate::chat_parser::ChatMessage;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind")]
pub enum ChatResuscitateEvent {
    /// Someone successfully resuscitated a player.
    Resuscitated {
        timestamp: String,
        caster_name: String,
        target_name: String,
    },
    /// Someone attempted but failed to resuscitate a player.
    ResuscitateFailed {
        timestamp: String,
        caster_name: String,
        target_name: String,
    },
}

/// Try to parse an [Action Emotes] ChatMessage into a resuscitate event.
/// Returns None if the message is not an Action Emotes message or not a resuscitate action.
pub fn parse_resuscitate_message(msg: &ChatMessage) -> Option<ChatResuscitateEvent> {
    if msg.channel.as_deref() != Some("Action Emotes") {
        return None;
    }

    let ts = msg.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
    let text = msg.message.trim();

    // Try "CasterName futilely attempts to resuscitate TargetName" first (more specific)
    if let Some((caster, target)) = split_on_marker(text, " futilely attempts to resuscitate ") {
        return Some(ChatResuscitateEvent::ResuscitateFailed {
            timestamp: ts,
            caster_name: caster.to_string(),
            target_name: target.to_string(),
        });
    }

    // Try "CasterName resuscitates TargetName"
    if let Some((caster, target)) = split_on_marker(text, " resuscitates ") {
        return Some(ChatResuscitateEvent::Resuscitated {
            timestamp: ts,
            caster_name: caster.to_string(),
            target_name: target.to_string(),
        });
    }

    None
}

/// Split text on a marker string, returning (before, after) if both are non-empty.
fn split_on_marker<'a>(text: &'a str, marker: &str) -> Option<(&'a str, &'a str)> {
    let pos = text.find(marker)?;
    let before = text[..pos].trim();
    let after = text[pos + marker.len()..].trim();
    if before.is_empty() || after.is_empty() {
        return None;
    }
    Some((before, after))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_parser::parse_chat_line;

    fn action_emote_msg(text: &str) -> ChatMessage {
        // Action emotes have no colon, so the parser treats the whole text as message
        let line = format!("26-04-01 10:25:54\t[Action Emotes] {}", text);
        parse_chat_line(&line).unwrap()
    }

    #[test]
    fn test_successful_resuscitate() {
        let msg = action_emote_msg("Tzunamy resuscitates Mellow Yellow");
        let event = parse_resuscitate_message(&msg).unwrap();
        match event {
            ChatResuscitateEvent::Resuscitated {
                caster_name,
                target_name,
                ..
            } => {
                assert_eq!(caster_name, "Tzunamy");
                assert_eq!(target_name, "Mellow Yellow");
            }
            _ => panic!("Expected Resuscitated"),
        }
    }

    #[test]
    fn test_failed_resuscitate() {
        let msg = action_emote_msg("Delthea futilely attempts to resuscitate Nougat");
        let event = parse_resuscitate_message(&msg).unwrap();
        match event {
            ChatResuscitateEvent::ResuscitateFailed {
                caster_name,
                target_name,
                ..
            } => {
                assert_eq!(caster_name, "Delthea");
                assert_eq!(target_name, "Nougat");
            }
            _ => panic!("Expected ResuscitateFailed"),
        }
    }

    #[test]
    fn test_non_action_emotes_channel_ignored() {
        let line = "26-04-01 05:30:52\t[Status] You earned 5 XP in Dying.";
        let msg = parse_chat_line(&line).unwrap();
        assert!(parse_resuscitate_message(&msg).is_none());
    }

    #[test]
    fn test_non_resuscitate_action_ignored() {
        let msg = action_emote_msg("Delthea dances with Nougat");
        assert!(parse_resuscitate_message(&msg).is_none());
    }

    #[test]
    fn test_combat_channel_ignored() {
        let line = "26-04-01 05:30:52\t[Combat] Something resuscitates Zenith";
        let msg = parse_chat_line(&line).unwrap();
        assert!(parse_resuscitate_message(&msg).is_none());
    }

    #[test]
    fn test_multiple_failed_then_success() {
        let fail1 = action_emote_msg("Blood futilely attempts to resuscitate Flesh");
        let fail2 = action_emote_msg("Blood futilely attempts to resuscitate Flesh");
        let success = action_emote_msg("Blood resuscitates Flesh");

        assert!(matches!(
            parse_resuscitate_message(&fail1),
            Some(ChatResuscitateEvent::ResuscitateFailed { .. })
        ));
        assert!(matches!(
            parse_resuscitate_message(&fail2),
            Some(ChatResuscitateEvent::ResuscitateFailed { .. })
        ));
        assert!(matches!(
            parse_resuscitate_message(&success),
            Some(ChatResuscitateEvent::Resuscitated { .. })
        ));
    }

    #[test]
    fn test_target_with_spaces() {
        let msg = action_emote_msg("Vandoodle resuscitates Vanbee Melee");
        let event = parse_resuscitate_message(&msg).unwrap();
        match event {
            ChatResuscitateEvent::Resuscitated { target_name, .. } => {
                assert_eq!(target_name, "Vanbee Melee");
            }
            _ => panic!("Expected Resuscitated"),
        }
    }

    #[test]
    fn test_target_with_unicode() {
        let msg = action_emote_msg("Hebrewrage resuscitates Bär");
        let event = parse_resuscitate_message(&msg).unwrap();
        match event {
            ChatResuscitateEvent::Resuscitated {
                caster_name,
                target_name,
                ..
            } => {
                assert_eq!(caster_name, "Hebrewrage");
                assert_eq!(target_name, "Bär");
            }
            _ => panic!("Expected Resuscitated"),
        }
    }

    #[test]
    fn test_failed_target_with_spaces() {
        let msg = action_emote_msg(
            "Vandoodle futilely attempts to resuscitate Vanbee Melee",
        );
        let event = parse_resuscitate_message(&msg).unwrap();
        match event {
            ChatResuscitateEvent::ResuscitateFailed {
                caster_name,
                target_name,
                ..
            } => {
                assert_eq!(caster_name, "Vandoodle");
                assert_eq!(target_name, "Vanbee Melee");
            }
            _ => panic!("Expected ResuscitateFailed"),
        }
    }

}
