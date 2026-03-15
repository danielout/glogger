# Player Log Pattern Registration Guide

This guide explains how to add custom pattern matchers to the PlayerLogWatcher without modifying core code.

## Overview

The PlayerLogWatcher uses a pattern registration system that allows modules to register custom line matchers. This makes the watcher extensible - new game events can be detected without editing the core watcher implementation.

## Pattern Matcher Signature

```rust
pub type PatternMatcher = fn(&str, &mut PlayerLogWatcher) -> Option<LogEvent>;
```

A pattern matcher is a function that:
- Takes a log line string and mutable access to the watcher
- Returns `Some(LogEvent)` if the pattern matches
- Returns `None` if the pattern doesn't match

## Core Patterns

Three patterns are registered by default:

1. **Character Login**: `"Logged in as character [CharacterName]"`
2. **Chat Log Path**: `"Logging chat to C:/Users/.../ChatLogs/Chat-26-03-06.log"`
3. **Area Transition**: `"LOADING LEVEL AreaCasino"`

See [`log_watchers.rs:217-273`](../../src-tauri/src/log_watchers.rs#L217-L273) for implementations.

## Adding a Custom Pattern

### Step 1: Define the Pattern Matcher Function

Create a function matching the `PatternMatcher` signature:

```rust
fn match_xp_gain(line: &str, _watcher: &mut PlayerLogWatcher) -> Option<LogEvent> {
    // Example: "You gain 100 experience in Sword."
    if let Some(start) = line.find("You gain ") {
        if let Some(mid) = line.find(" experience in ") {
            // Extract amount
            let amount_str = &line[start + 9..mid];
            if let Ok(amount) = amount_str.parse::<u32>() {
                // Extract skill name
                let skill_start = mid + 15;
                let skill_end = line.len() - 1; // Remove trailing period
                let skill = line[skill_start..skill_end].to_string();

                return Some(LogEvent::XpGained {
                    skill,
                    amount,
                    timestamp: chrono::Local::now().naive_local(),
                });
            }
        }
    }
    None
}
```

### Step 2: Register the Pattern

In your module that creates the PlayerLogWatcher:

```rust
let mut watcher = PlayerLogWatcher::new(file_path);
watcher.register_pattern(match_xp_gain);
```

Or when creating in the coordinator:

```rust
// In coordinator.rs or similar
impl DataIngestCoordinator {
    pub fn start_player_log_tailing(&mut self) -> Result<(), String> {
        // ... existing code ...

        let mut watcher = PlayerLogWatcher::new(player_log_path);

        // Register additional patterns
        watcher.register_pattern(match_xp_gain);
        watcher.register_pattern(match_survey_complete);
        watcher.register_pattern(match_item_looted);

        watcher.start()?;
        self.player_watcher = Some(watcher);

        Ok(())
    }
}
```

## Pattern Execution Order

Patterns are tried in registration order:
1. Core patterns (character login, chat log path, area transition)
2. Custom patterns (in order of registration)

The first pattern that returns `Some(LogEvent)` wins - no further patterns are tried for that line.

## Accessing Watcher State

Pattern matchers have mutable access to the watcher, allowing them to:

### Read State

```rust
fn match_with_context(line: &str, watcher: &mut PlayerLogWatcher) -> Option<LogEvent> {
    // Check if we have an active character
    if let Some(character) = watcher.get_active_character() {
        // Pattern only applies when logged in
        if line.contains("special event") {
            return Some(LogEvent::Unparsed {
                line: format!("{}: {}", character, line),
                timestamp: chrono::Local::now().naive_local(),
            });
        }
    }
    None
}
```

### Update State

```rust
fn match_character_login(line: &str, watcher: &mut PlayerLogWatcher) -> Option<LogEvent> {
    if let Some(start) = line.find("Logged in as character [") {
        if let Some(end) = line[start..].find(']') {
            let name_start = start + "Logged in as character [".len();
            let name_end = start + end;
            let character_name = line[name_start..name_end].to_string();

            // Update watcher's active character
            watcher.active_character = Some(character_name.clone());

            return Some(LogEvent::CharacterLogin {
                character_name,
                timestamp: chrono::Local::now().naive_local(),
                area: None,
            });
        }
    }
    None
}
```

**Note:** Only update watcher state fields if necessary. Most patterns should just return events.

## Best Practices

### 1. Fail Fast

Return `None` as early as possible if the pattern doesn't match:

```rust
fn match_survey_complete(line: &str, _watcher: &mut PlayerLogWatcher) -> Option<LogEvent> {
    // Quick check first
    if !line.contains("Survey complete") {
        return None;
    }

    // More expensive parsing only if quick check passes
    // ... detailed parsing ...
}
```

### 2. Use String Slices

Avoid allocating strings until you're sure the pattern matches:

```rust
// ❌ Bad - allocates even if pattern doesn't match
let parts: Vec<String> = line.split(':').map(|s| s.to_string()).collect();
if parts.len() != 3 {
    return None;
}

// ✅ Good - only allocates if pattern matches
if let Some((skill, amount)) = line.split_once(" experience in ") {
    return Some(LogEvent::XpGained {
        skill: skill.to_string(),
        amount: amount.parse().ok()?,
        timestamp: chrono::Local::now().naive_local(),
    });
}
None
```

### 3. Don't Panic

Use `?` operator or `unwrap_or` to handle parsing errors gracefully:

```rust
fn match_xp_gain(line: &str, _watcher: &mut PlayerLogWatcher) -> Option<LogEvent> {
    let amount_str = extract_amount(line)?;  // Returns None on failure
    let amount = amount_str.parse::<u32>().ok()?;  // Returns None on parse error

    Some(LogEvent::XpGained {
        skill: "Unknown".to_string(),
        amount,
        timestamp: chrono::Local::now().naive_local(),
    })
}
```

### 4. Document Patterns

Add comments showing the expected log line format:

```rust
/// Match item looted pattern
///
/// Example: "You receive Rough Animal Skin x5."
/// Example: "You receive Red Crystal."
fn match_item_looted(line: &str, _watcher: &mut PlayerLogWatcher) -> Option<LogEvent> {
    // Implementation...
}
```

## Example: Survey Module

Here's a complete example of a survey parsing module:

```rust
// src-tauri/src/survey_patterns.rs

use crate::log_watchers::{LogEvent, PlayerLogWatcher, PatternMatcher};
use chrono::Local;

/// Register all survey-related patterns
pub fn register_survey_patterns(watcher: &mut PlayerLogWatcher) {
    watcher.register_pattern(match_survey_complete);
    watcher.register_pattern(match_survey_quality);
}

/// Match survey completion
///
/// Example: "Survey complete! You have completed Eltibule Green Mineral Survey."
fn match_survey_complete(line: &str, _watcher: &mut PlayerLogWatcher) -> Option<LogEvent> {
    if !line.contains("Survey complete!") {
        return None;
    }

    // Extract survey name
    if let Some(start) = line.find("You have completed ") {
        let name_start = start + 19;
        let name_end = line.len() - 1; // Remove trailing period
        let survey_type = line[name_start..name_end].to_string();

        return Some(LogEvent::SurveyCompleted {
            survey_type,
            quality: None,
            timestamp: Local::now().naive_local(),
        });
    }

    None
}

/// Match survey quality result
///
/// Example: "You have found a quality-98 survey!"
fn match_survey_quality(line: &str, _watcher: &mut PlayerLogWatcher) -> Option<LogEvent> {
    if let Some(start) = line.find("You have found a quality-") {
        if let Some(end) = line[start..].find(" survey!") {
            let quality_start = start + 25;
            let quality_end = start + end;
            let quality_str = &line[quality_start..quality_end];

            if let Ok(quality) = quality_str.parse::<u32>() {
                return Some(LogEvent::SurveyCompleted {
                    survey_type: "Unknown".to_string(),
                    quality: Some(quality),
                    timestamp: Local::now().naive_local(),
                });
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_survey_complete() {
        let mut watcher = PlayerLogWatcher::new(PathBuf::from("test.log"));
        register_survey_patterns(&mut watcher);

        let event = watcher.parse_line("Survey complete! You have completed Eltibule Green Mineral Survey.");
        assert!(event.is_some());

        if let Some(LogEvent::SurveyCompleted { survey_type, quality, .. }) = event {
            assert_eq!(survey_type, "Eltibule Green Mineral Survey");
            assert_eq!(quality, None);
        } else {
            panic!("Expected SurveyCompleted event");
        }
    }

    #[test]
    fn test_survey_quality() {
        let mut watcher = PlayerLogWatcher::new(PathBuf::from("test.log"));
        register_survey_patterns(&mut watcher);

        let event = watcher.parse_line("You have found a quality-98 survey!");
        assert!(event.is_some());

        if let Some(LogEvent::SurveyCompleted { quality, .. }) = event {
            assert_eq!(quality, Some(98));
        } else {
            panic!("Expected SurveyCompleted event");
        }
    }
}
```

Then in `coordinator.rs`:

```rust
use crate::survey_patterns;

impl DataIngestCoordinator {
    pub fn start_player_log_tailing(&mut self) -> Result<(), String> {
        // ... existing code ...

        let mut watcher = PlayerLogWatcher::new(player_log_path);

        // Register survey patterns
        survey_patterns::register_survey_patterns(&mut watcher);

        watcher.start()?;
        self.player_watcher = Some(watcher);

        Ok(())
    }
}
```

## Testing Custom Patterns

Always write tests for your custom patterns:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_custom_pattern() {
        let mut watcher = PlayerLogWatcher::new(PathBuf::from("test.log"));
        watcher.register_pattern(match_custom_event);

        // Test matching
        let event = watcher.parse_line("expected input");
        assert!(event.is_some());

        // Test non-matching
        let no_event = watcher.parse_line("unrelated line");
        assert!(no_event.is_none());

        // Test edge cases
        let edge = watcher.parse_line("edge case input");
        assert!(matches!(edge, Some(LogEvent::YourVariant { .. })));
    }
}
```

Run tests with:
```bash
cargo test pattern_name
```

## Performance Considerations

### Pattern Order Matters

Place frequently-matching patterns first:

```rust
// ✅ Good - common patterns first
watcher.register_pattern(match_chat_log_path);     // Very frequent
watcher.register_pattern(match_area_transition);   // Somewhat frequent
watcher.register_pattern(match_rare_event);        // Rare

// ❌ Bad - rare pattern checked for every line
watcher.register_pattern(match_rare_event);
watcher.register_pattern(match_chat_log_path);
```

### Avoid Regex Unless Necessary

String operations are faster than regex:

```rust
// ✅ Good - simple string operations
if line.contains("You gain ") && line.contains(" experience in ") {
    // ...
}

// ❌ Slower - regex overhead
let re = Regex::new(r"You gain (\d+) experience in (.+)").unwrap();
if let Some(caps) = re.captures(line) {
    // ...
}
```

Only use regex for complex patterns that can't be handled with string operations.

## Troubleshooting

### Pattern Not Matching

1. **Check pattern order** - Is another pattern matching first?
2. **Print the line** - Use `eprintln!("Line: {:?}", line);` to see exact content
3. **Check whitespace** - Log files may have unexpected spaces/tabs
4. **Test in isolation** - Write a unit test with exact log line

### Pattern Registered But Not Called

```rust
// Make sure watcher is created correctly
let mut watcher = PlayerLogWatcher::new(path);  // Note: mut

// Register before starting
watcher.register_pattern(my_pattern);

// Start watcher
watcher.start()?;
```

### Borrow Checker Issues

If you need to access other data while parsing:

```rust
// Create closure that captures needed data
let skill_list = vec!["Sword", "Archery"];
let matcher = move |line: &str, _watcher: &mut PlayerLogWatcher| {
    // Can use skill_list here
    if skill_list.iter().any(|s| line.contains(s)) {
        // ...
    }
    None
};

watcher.register_pattern(matcher);
```

## Reference

- [log_watchers.rs](../../src-tauri/src/log_watchers.rs) - Core implementation
- [coordinator.rs](../../src-tauri/src/coordinator.rs) - Watcher usage
- [Data Architecture](../reference/data-architecture.md) - Overall system design
