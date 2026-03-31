# Crafting — Projects

## Overview

Users create named projects containing multiple target crafts. The helper aggregates all ingredient requirements, deduplicating shared materials. Projects auto-resolve materials and check inventory availability when selected.

## Project Management

- Create, rename, delete, and duplicate projects
- Add recipes via search, set desired quantity per entry
- Reorder entries with drag-and-drop
- Per-entry toggle for intermediate expansion
- Projects persist to SQLite across sessions
- **Sorting** — sidebar supports Recent, A-Z, Z-A sort modes
- **Grouping** — projects can be assigned a group name; grouped projects display under collapsible headers in the sidebar

## Project Groups

Projects with the same `group_name` appear under a collapsible group header in the sidebar. The group name is set via an inline editor in the recipe panel header, with autocomplete from existing group names. Ungrouped projects appear below all groups.

### Group Summary ("Super-Project")

Clicking a group header in the sidebar selects the group and shows a combined materials/availability view across all projects in that group. The chevron arrow toggles collapse/expand of the group's project list independently.

When a group is selected:
- The materials panel aggregates ingredients from all projects in the group
- Stock targets are resolved across all entries
- Pickup list, shopping list, and availability breakdown reflect the combined totals
- The recipe panel is hidden (individual projects are still editable by clicking them)
- A "Recalculate Group Materials" button allows manual re-resolve
- Export generates a combined material list named after the group

## Per-Entry Cost Display

Each recipe entry shows an estimated total crafting cost (`~Xg`) on the collapsed header row, so users can compare costs at a glance without expanding entries. Cost calculation respects chance-to-consume ingredients.

## Stock Targets

Entries can operate in two modes:
- **Manual mode** (default) — user specifies how many to craft
- **Target mode** — user specifies a target stock count (e.g., "keep 5 in storage"); the system queries current inventory + storage for the output item and auto-calculates how many to craft (`target - current stock`). When the target is met, the entry shows "met" and contributes 0 to material requirements.

Toggle between modes via the `manual` / `target` button on each entry card.

## Auto-resolve

When a project is selected, materials and inventory availability are automatically calculated (no need to click "Calculate All Materials"). The button remains available for manual re-resolve. A generation guard prevents stale results if the user switches projects during resolution.

## Material Breakdown

Two-column layout for efficient use of horizontal space:

**Left column:**
- **Materials table** — unified view showing Need/Inv/Storage/Shortfall when availability has been checked. Shows all ingredients needed across recipes, with deduplication.
- **Craft or Buy** — centralized list of all craftable ingredients. Toggle each between "craft this" (expand into sub-ingredients) and "buy this" (keep as a leaf material). Toggling applies project-wide across all recipe entries that share that ingredient. Replaces needing to dig into individual recipe entries.

**Right column:**
- **Shopping / Gathering** — items to buy from vendors or source elsewhere, with estimated costs
- **Pickup List** — vault-organized list of items to retrieve from storage

**Full width (top):**
- **Recipe summary** — compact inline list of what you're crafting (recipe name × quantity)
- **Resolve button** — auto-runs on project load; manual re-resolve available

## Intermediate Crafts

Ingredients that can themselves be crafted are identified during material resolution. The "Craft or Buy?" section in the materials panel provides a centralized toggle:
- Items marked for crafting are expanded into their sub-ingredients in the material totals
- The toggle applies project-wide — all recipe entries sharing that ingredient are updated
- Per-entry toggle buttons in recipe cards also apply project-wide for consistency
- State is persisted per-entry in `expanded_ingredient_ids` (JSON column)

## Live Crafting Detection

Real-time tracking of crafting progress against active projects or quick-calc targets. Displayed as an overlay panel within the Projects tab (via `LiveCraftingPanel.vue`).

### How it works

- Listens to `player-event` Tauri events for `ItemAdded` and `ItemStackChanged`
- Matches detected item outputs against tracked recipe entries
- Updates crafts completed and detected output quantities in real-time
- Maintains a rolling log of recent craft detections (max 100 entries)

### Tracking modes

- **Project tracking** — monitors all recipe entries in the active project
- **Quick-calc tracking** — monitors a single recipe from the quick calculator

## Database Schema

- `crafting_projects` — id, name, notes, group_name (nullable), created_at, updated_at
- `crafting_project_entries` — id, project_id, recipe_id, recipe_name, quantity, sort_order, expanded_ingredient_ids (JSON), target_stock (nullable)
- Migration v10 added `group_name` and `target_stock` columns
