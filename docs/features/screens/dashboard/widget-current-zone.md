# Widget: Zone NPCs

**ID:** `zone-npcs` | **Default size:** Large | **Component:** `widgets/ZoneNpcsWidget.vue` | **Config:** `widgets/ZoneNpcsWidgetConfig.vue`

Shows the player's current area and lists all friendly NPCs located there in a compact two-line format:

- **Line 1:** NPC name (`NpcInline`), favor rank badge (color-coded), trained skills (diamond-prefixed)
- **Line 2:** Storage status (used/total), vendor gold (available/max), gold reset timer if applicable

Updates reactively when the player transitions to a new zone.

### Configuration

Gear icon opens a config popover with:
- **NPC type filters:** Checkboxes for Storage NPCs, Shop NPCs, Trainers
- **Favor range:** Min/max favor rank dropdowns
- **Options:** Show giftable only, push NPCs without services to bottom

Config is persisted in localStorage under `zoneNpcsWidget.config`.

**Data sources:** `gameStateStore.world.area` (persistent, database), `gameDataStore.getNpcsInArea()` (CDN), `gameStateStore.favorByNpc` (persistent, database), `gameStateStore.vendorByNpc`, `gameStateStore.storageByVault`, `gameStateStore.storageVaultsByKey`.

### History

This widget replaces the previous `CurrentZone.vue` (simple NPC list) and the old card-based `ZoneNpcsWidget.vue` (detail view), merging both into a single compact configurable widget.
