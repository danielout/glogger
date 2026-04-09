# Widget: Statehelm Gifting

**ID:** `statehelm-summary` | **Default size:** Medium | **Component:** `widgets/StatehelmSummaryWidget.vue`

Lightweight summary of weekly Statehelm gift progress:
- Header: total gifts given / max possible + weekly reset countdown
- Progress bar (gold fill)
- Summary counts: NPCs maxed vs. remaining
- Up to 5 NPCs still needing gifts, sorted by fewest gifts first, each showing:
  - `NpcInline` (hoverable)
  - Visual gift dots: filled (gold) for given, empty (dim) for remaining

Weekly reset is Monday 00:00 UTC. Uses the same `useStatehelmTracker` composable as the full Statehelm tab — calls `loadGiftLog()` on mount.

**Data source:** `useStatehelmTracker` composable (gift log from database, NPC data from CDN, favor from game state).
