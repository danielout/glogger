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
- **Favorite Gift Items** — badge list
- **Raw JSON**

## Data Loading

NPCs are preloaded on startup into `npcsByKey` and `npcsByDisplayName` maps in `gameDataStore`, using synchronous resolution rather than async queries.
