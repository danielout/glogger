# Widget: Favor Changes

**ID:** `favor-changes` | **Default size:** Medium | **Component:** `widgets/FavorChangesWidget.vue`

Activity feed (purple dot) showing NPC favor deltas:
- Player.log: `FavorChanged` events (gifts, quest rewards, etc.)
- NPC names rendered via `NpcInline` for tooltips and navigation
- Shows signed running total

**Data source:** `gameStateStore.favorChanges`. Session-only, in-memory.
