# Crafting — Dynamic Items

## Overview

Some recipes accept wildcard ingredient slots (e.g. "Any Crystal", "Cheap Meat") instead of specific items. The Dynamic Items tab lets users configure which concrete items are allowed for each wildcard keyword. These preferences feed into material availability checking across projects.

## How It Works

- **Left pane** lists all keywords used as wildcard ingredient slots, with enabled/total item counts
- **Center pane** shows all items matching the selected keyword, each with a toggle
- Default: all items are enabled; users disable items they don't want to use
- Preferences persist across sessions via `viewPreferences`

## Integration with Projects

When a project's materials are resolved, dynamic ingredients are no longer skipped during availability checking:

1. Each wildcard keyword is resolved to its set of enabled concrete items
2. Inventory and storage are queried for all enabled items
3. Quantities are aggregated into a single material row (shown with a ◆ indicator)
4. Shortfalls reflect only items the user has enabled

## Data Storage

Preferences are stored in `viewPreferences.dynamicItems` as a map of keyword → disabled item IDs. The disabled-list model means:
- New items added by CDN updates are automatically enabled
- Only exclusions are stored, keeping data small

## Components

- `DynamicItemsTab.vue` — PaneLayout left/center tab component
- Preferences managed via `craftingStore.setDynamicItemDisabled()` / `setAllDynamicItems()`
- Backend: `get_recipe_ingredient_keywords` returns keyword + player-facing description pairs
