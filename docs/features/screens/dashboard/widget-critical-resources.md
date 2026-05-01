# Widget: Critical Resources

**ID:** `critical-resources` | **Default size:** Medium | **Component:** `widgets/CriticalResourcesWidget.vue`

Displays quantities of tracked valuable items at a glance:
- Each row shows an `ItemInline` (hoverable, clickable) with the owned count (gold text, right-aligned)
- Items with 0 count render dimmed
- Default tracked items: Diamond, Amethyst, Aquamarine, Eternal Green, Salt, Fire Dust

Uses `gameStateStore.ownedItemCounts[itemName]` which merges persisted inventory (database) with live inventory (session events) for accurate counts.

## Configuration

Gear icon opens a config popover to customize tracked items:
- **Item search** — autocomplete item picker to add new items to the tracked list
- **Remove** — click to remove an item from the list
- Persisted via `useViewPrefs`
- Defaults to 6 items: Diamond, Amethyst, Aquamarine, Eternal Green, Salt, Fire Dust

**Data source:** `gameStateStore.ownedItemCounts`. Persistent + live merged.
