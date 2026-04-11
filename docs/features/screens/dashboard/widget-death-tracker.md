# Widget: Death Tracker

**ID:** `death-tracker` | **Default size:** Medium | **Component:** `widgets/DeathTrackerWidget.vue`

Combined death and resuscitation tracker showing:
- **Recent Deaths** — last 5 deaths with killer name (via EnemyInline) and timestamp
- **Last Rezzed By** — who most recently resuscitated the active character, with timestamp
- **Top Rezzers** — top 5 players who have rezzed the active character, ranked by count
- **You Rezzed** — top 5 players the active character has rezzed, ranked by count
- **Summary footer** — total deaths, rezzes received, and rezzes given

## Data Sources

### Deaths
Loaded from existing `character_deaths` table via `deathStore`. Deaths are detected from `[Combat]` channel `(FATALITY!)` messages by `chat_combat_parser.rs` and persisted by the coordinator.

### Resuscitations
Loaded from `character_resuscitations` table via `resuscitateStore` and `get_character_resuscitations` Tauri command.

Resuscitate events are parsed from the `[Action Emotes]` channel by `chat_resuscitate_parser.rs`. Two patterns are detected:
- `"CasterName resuscitates TargetName"` — successful rez
- `"CasterName futilely attempts to resuscitate TargetName"` — failed attempt

All nearby resuscitate events are recorded (not just those involving the active character), enabling both "who rezzed me" and "who did I rez" tracking.

### Live Events
- `character-death` — emitted by coordinator on player death, handled by `deathStore.handleDeathEvent()`
- `character-resuscitated` — emitted by coordinator on any resuscitate event, handled by `resuscitateStore.handleResuscitateEvent()`

Both stores are initialized lazily on widget mount and updated in real-time via Tauri event listeners registered in `startupStore.ts`.

## Database

**Table:** `character_resuscitations` (migration v22)

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PK | Auto-increment |
| character_name | TEXT | Active character when event occurred |
| server_name | TEXT | Active server |
| occurred_at | TEXT | UTC timestamp |
| caster_name | TEXT | Who cast the rez |
| target_name | TEXT | Who was rezzed |
| success | INTEGER | 1 = successful, 0 = failed attempt |
| area | TEXT | Current area (nullable) |

Indexed on `(character_name, server_name)` and `occurred_at`.

## Files

- `src-tauri/src/chat_resuscitate_parser.rs` — parser for [Action Emotes] resuscitate lines
- `src-tauri/src/db/resuscitate_commands.rs` — Tauri query command
- `src/stores/resuscitateStore.ts` — Pinia store with computed summaries
- `src/components/Dashboard/widgets/DeathTrackerWidget.vue` — widget component

## Future Enhancements

- Resuscitate cooldown timer (start countdown when the active character casts Resuscitate successfully)
- Link deaths to the rez that followed them (time-correlation between death and subsequent rez)
- Click-to-navigate from deaths list to the full Deaths screen
