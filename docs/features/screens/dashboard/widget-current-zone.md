# Widget: Current Zone

**ID:** `current-zone` | **Default size:** Medium | **Component:** `CurrentZone.vue`

Shows the player's current area and lists all friendly NPCs located there:
- Area name displayed via `AreaInline`
- NPC list loaded from CDN data via `getNpcsInArea()`, each rendered with `NpcInline`
- Favor rank badge next to each NPC (color-coded by tier)
- Updates reactively when the player transitions to a new zone

**Data source:** `gameStateStore.world.area` (persistent, database), `gameDataStore.getNpcsInArea()` (CDN), `gameStateStore.favorByNpc` (persistent, database).
