# Widget: Words of Power

**Size:** Medium (2 columns)

## Purpose

Tracks discovered words of power, the one-use spoken buffs crafted via the Calligraphy skill. Words decay in usefulness over time (other players can discover and use the same word), so the widget emphasizes age — a 2-hour-old word should be preferred over a 3-day-old one.

## Features

- **Auto-capture:** Words are detected from `ProcessBook` events with title "You discovered a word of power!" and persisted to SQLite automatically.
- **Grouped by power name:** Words are grouped under their power name (e.g., "Unnatural Gravity") with newest groups first.
- **Age counter:** Each word shows a live-updating age (e.g., "2h 15m ago", "3d 6h ago") with color coding:
  - Green: < 1 hour
  - Gold: 1-6 hours
  - Orange: 6-24 hours
  - Red: > 24 hours
- **Click to copy:** Clicking a word copies it to the clipboard.
- **Search:** Filter words by word text or power name.
- **Manual add:** Add words manually for words discovered outside of log tailing.
- **Remove:** Hover a word to reveal the delete button.

## Data Flow

1. Player crafts a word of power in-game
2. `ProcessBook("You discovered a word of power!", ...)` appears in Player.log
3. `PlayerEventParser` emits a `BookOpened` event
4. `DataIngestCoordinator` detects the title match, extracts the word from `<sel>` tags and power name from `<size=125%>` tags
5. Word is persisted to `words_of_power` table
6. Coordinator emits `game-state-updated` with domain `"words_of_power"`
7. Widget reloads from DB

## Database

Table: `words_of_power` (migration v37)

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PK | Auto-increment |
| character_name | TEXT | Character who discovered the word |
| server_name | TEXT | Server name |
| word | TEXT | The word itself (e.g., "TOAEOACHROF") |
| power_name | TEXT | Power name (e.g., "Unnatural Gravity") |
| description | TEXT | Effect description text |
| discovered_at | TEXT | Timestamp of discovery |
| source | TEXT | "auto" (from log) or "manual" (user-added) |

## Files

- `src/components/Dashboard/widgets/WordsOfPowerWidget.vue` — widget component
- `src-tauri/src/db/words_of_power_commands.rs` — Tauri commands and DB helper
- `src-tauri/src/db/migrations.rs` — migration v37
- `src-tauri/src/coordinator.rs` — `ingest_word_of_power()` method
