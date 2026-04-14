# Data Browser — NPCs

## Overview

Browse NPCs with area filtering, descriptions, training info, and favor gift preferences.

## Search & Filters

- **Area filter dropdown** — populated from preloaded NPC data
- **Text search** — across name and description (computed, no debounce)

## Detail View

- Name, CDN key, area name, friendly area name
- **Description**
- **Trains Skills** — badge list of skills the NPC can train
- **Favor Preferences** — sorted by preference amount (descending), color-coded by desire:
  - Love (pink), Like (cyan), Dislike (red), Hate (dark red)
  - Shows item name or keywords with preference value (+X)
- **Gift Favor Tiers** — badges showing at which favor levels the NPC gives gifts
- **Associated Quests** — quests linked to this NPC (from precomputed `questsByNpc` index via `getQuestsForNpc`)
- **Vendor Inventory** — items sold by this NPC, loaded async via `get_npc_vendor_items`. Shows item name (via `ItemInline`) and estimated vendor buy price (`value × 1.5`). Data sourced from CDN `sources_items.json` Vendor/Barter entries, indexed at CDN load time in `vendor_items_by_npc`.
- **Raw JSON**

## Data Loading

NPCs are preloaded on startup into `npcsByKey` and `npcsByDisplayName` maps in `gameDataStore`, using synchronous resolution rather than async queries. Quests are also preloaded and indexed by NPC key (via `FavorNpc` field) into a reactive `questsByNpc` computed map, making quest lookups synchronous. Vendor inventory is loaded on-demand when an NPC is selected.
