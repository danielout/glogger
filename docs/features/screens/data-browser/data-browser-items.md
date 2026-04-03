# Data Browser — Items

## Overview

Browse and search all game items with advanced filtering by keywords, equipment slot, and level range.

## Search & Filters

- **Text search** — by item name (debounced 250ms)
- **Advanced filters** (collapsible):
  - Keyword filter with autocomplete dropdown
  - Equipment slot filter
  - Level range filter (min/max)

## Detail View

- Icon (pixelated rendering)
- ID, icon ID, value in copper, stack size
- **Equipment** — slot, skill requirements
- **Crafting** — TSys profile, craft points, target level
- **Food description** — styled callout for food items
- **Bestow** — ability, quest, recipes, title ID granted by the item
- **Usage count**
- **Sources** — items/entities that drop or bestow this item
- **Keywords** — color-coded badges (Lint_* in bronze, others in blue)
- **Effect descriptions** — green text list
- **Raw JSON**

## Item Tooltip

The shared `ItemTooltip` component (used everywhere items are referenced via `ItemInline`) shows:

- Icon, name, vendor price, buy-used price (2×), market price (if set), effective value
- Description, keywords, effect descriptions
- **Sold by** — comma-separated list of NPC vendors (via `NpcInline`) that sell or barter this item. Loaded on hover via `get_vendors_for_item` using the `vendors_for_item` reverse index. Only appears for items with Vendor/Barter entries in CDN sources data.
- Max stack size, owned count
- Market value editor (set/edit/remove)
