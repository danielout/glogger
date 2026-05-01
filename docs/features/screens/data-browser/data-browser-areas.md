# Data Browser — Areas

## Overview

Browse all game areas/zones. The area list comes from the CDN `areas.json`, enriched with NPC counts, monster counts, and storage vault info from other data sources.

## Search & Filter

- Text search by friendly name, short name, or area key
- Result count displayed

## Area Tooltips

`AreaInline` components display a rich tooltip on hover showing:
- Area friendly name and short name
- Notable NPCs (up to 8, with "+N more" overflow)
- Monsters in area (up to 12, with "+N more" overflow)
- Click navigates to the area's detail view in the Data Browser

## Detail View

- Area friendly name and short name
- Internal key (e.g. `AreaSerbule`)
- Three-column layout: **NPCs** | **Storage Vaults** | **Monsters**
- **NPCs** — bullet list of NPCs in the area, rendered as clickable `NpcInline` components
- **Storage Vaults** — bullet list showing vault name with "up to X" max storage size
- **Monsters** — bullet list of unique monster names, rendered as clickable `EnemyInline` components (navigate to enemy entry)
- **Raw JSON** (opt-in via settings)

## Data Sources

- **Area list:** CDN `areas.json` — provides friendly names
- **NPC counts:** CDN `npcs.json` — NPCs have an `area_name` field
- **Monster counts:** Translation strings (`strings_requested.json`) — monster keys encode area (e.g. `monster_AreaSerbule/GiantRat_Name`)
- **Storage vaults:** CDN `storagevaults.json` — vaults have an `Area` field
- Backend commands: `get_all_areas()`, `get_npcs_in_area(area)`, `get_monsters_in_area(area)`, `get_storage_vaults_in_area(area)`
- Frontend: `src/components/DataBrowser/AreaBrowser.vue`
