# Widget: Milking Timers

**Size:** Small (1 column)

## Purpose

Tracks cow milking activity across three categories: NPC cow cooldown timers, player-to-player milking leaderboards, and self-milking (cow players milking themselves).

## Features

### Timers Tab (NPC Cows)
- **Cooldown tracking:** Shows 1-hour countdown timers for each NPC cow milked, with "Ready!" when cooldown expires.
- **Grouped by zone:** Cows are organized under their zone, sorted alphabetically.
- **Current zone first:** The player's current zone is highlighted and floated to the top.
- **Cooldown backfill:** If the player tries to milk a cow that's still on cooldown, the error message is detected and a timer is backfilled (set to 59 minutes ago since exact time is unknown).

### Players Milked Tab
- **Leaderboard:** Ranked list of other players milked by this character, sorted by count descending.

### Milked By Tab
- **Leaderboard:** Ranked list of players who have milked this character, sorted by count descending.

### Self-Milked Counter
- **Inline counter:** A single line above the tabs showing the total self-milk count. Only visible when count > 0.

## Detection & Data Flow

### NPC Cow Milking
1. Player interacts with an NPC cow (e.g., `Cow_Homer`)
2. `PlayerEventParser` emits `InteractionStarted` with an `npc_name` starting with `Cow_`
3. Coordinator stores `pending_cow_interaction`
4. Chat status `[Status] Bottle of Milk added to inventory.` fires
5. Coordinator matches the milk gain to the pending cow interaction
6. Cow name and zone are persisted to `milking_timers` table (UPSERT)

### NPC Cow Cooldown Backfill
1. Player tries to milk a cow still on cooldown
2. `ProcessScreenText(ErrorMessage, "You've already milked Homer in the past hour.")` is detected
3. Coordinator records a backfill entry with timestamp = now - 59 minutes

### Player-to-Player Milking (Milking Another Player)
1. Player milks another player in cow form
2. `ProcessScreenText(GeneralInfo, "You obtain fresh milk from PlayerName.")` is detected
3. Recorded as direction `milked` in `player_milking_log`

### Player-to-Player Milking (Being Milked)
1. Another player milks this character
2. `ProcessScreenText(ImportantInfo, "PlayerName has milked you.")` is detected
3. Recorded as direction `milked_by` in `player_milking_log`

### Self-Milking (Cow Player Milks Themselves)
1. A player in cow form uses Collect Milk on themselves
2. Combat chat `[Combat] PlayerName: Collect Milk on PlayerName!` is detected (no entity ID, matching active character as both source and target)
3. Recorded as direction `self_milked` in `player_milking_log`
4. Note: self-milking does NOT produce the `ProcessScreenText(GeneralInfo, "You obtain fresh milk from ...")` message that milking other players does

## Database

### Table: `milking_timers` (migration v35)

| Column | Type | Description |
|--------|------|-------------|
| character_name | TEXT | PK part — character who milked |
| server_name | TEXT | PK part — server |
| cow_name | TEXT | PK part — display name (e.g., "Homer") |
| zone | TEXT | PK part — area where cow was milked |
| last_milked_at | TEXT | RFC3339 timestamp of last milk |

### Table: `player_milking_log` (migration v36, updated v38)

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PK | Auto-increment |
| character_name | TEXT | Character who performed/received the milking |
| server_name | TEXT | Server name |
| other_player | TEXT | The other player involved (or self for self_milked) |
| direction | TEXT | `milked`, `milked_by`, or `self_milked` |
| milked_at | TEXT | RFC3339 timestamp |

## Files

- `src/components/Dashboard/widgets/MilkingTimersWidget.vue` — widget component (3 tabs + self-milked counter)
- `src-tauri/src/coordinator.rs` — detection logic for all milking types
- `src-tauri/src/db/migrations.rs` — migrations v35 (milking_timers), v36 (player_milking_log), v38 (self_milked direction)
- `src-tauri/src/db/game_state_commands.rs` — `get_milking_timers` and `get_player_milking_leaderboard` Tauri commands
