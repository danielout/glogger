# Statehelm Reputation Tracker

Tracks weekly gift progress for Statehelm NPCs. Each NPC accepts up to 5 gifts per week (weeks reset Monday 00:00 UTC). Shows NPC services (training, storage, vendor) and gift preferences at a glance.

## How It Works

Gift events are detected from `ProcessDeltaFavor` log lines where `is_gift` is `True`. These are already parsed by `PlayerEventParser` as `FavorChanged` events with the `is_gift` flag. When a gift is detected:

1. The favor delta is applied to `game_state_favor` (existing behavior)
2. A separate row is inserted into `game_state_gift_log` with the NPC key, timestamp, and favor delta
3. The frontend composable watches the favor activity feed for real-time updates

## NPC Selection

Statehelm NPCs are identified dynamically from CDN data:
- `area_friendly_name` or `area_name` contains "Statehelm"
- NPC has at least one entry in `preferences` (meaning they accept gifts)

This is fully data-driven — new NPCs added to Statehelm in future CDN updates appear automatically.

## Weekly Reset

The week boundary is calculated as Monday 00:00 UTC. Gift counts are derived by filtering `game_state_gift_log` entries with `gifted_at >= weekStart`. No data is deleted on reset — old gift logs are retained for history.

## Manual Adjustment

Users can manually increment/decrement gift counts per NPC using +/- buttons on each card. This covers cases where glogger missed a gift event (e.g., app wasn't running) or incorrectly tracked one. Manual additions insert a row with `favor_delta = 0.0`; removals delete the most recent gift log entry for that NPC within the current week.

## NPC Card Info

Each card displays:
- **NPC name** (clickable via `NpcInline`)
- **Favor tier** badge (from `game_state_favor`)
- **Gift progress** — dot indicators and count (X / 5)
- **Training** — skills the NPC trains (via `SkillInline`)
- **Storage** — whether they offer storage and at which tiers space increases
- **Vendor** — buying caps per favor tier and item types
- **Gift preferences** — what the NPC likes/loves/hates with pref values

## Filters

- **Hide above tier** — dropdown to hide NPCs at or above a chosen favor tier (e.g., hide Soul Mates)
- **Trainers / Vendors / Storage** — checkbox filters (additive: show NPCs matching any checked service)
- **Hide maxed gifts** — hide NPCs that already received 5 gifts this week

## Data Sources

| Data | Source |
|------|--------|
| Gift events | `game_state_gift_log` (DB), populated from `FavorChanged` events where `is_gift = true` |
| NPC list & preferences | CDN via `gameDataStore.npcsByKey` |
| NPC services | CDN `services` array, parsed via `parseServices()` |
| Favor tiers | `game_state_favor` (DB) via `gameStateStore.favorByNpc` |

## Key Files

- Gift log persistence: `src-tauri/src/game_state.rs` (in `FavorChanged` handler)
- Database migration: `src-tauri/src/db/migrations.rs` (v17)
- Tauri commands: `src-tauri/src/db/game_state_commands.rs` (`get_gift_log`, `add_manual_gift`, `remove_last_gift`)
- Frontend composable: `src/composables/useStatehelmTracker.ts`
- UI: `src/components/Character/StatehelmView.vue`
