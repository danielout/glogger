# Crafting — Projects

## Overview

Users create named projects containing multiple target crafts. The helper aggregates all ingredient requirements, deduplicating shared materials. Projects auto-resolve materials and check inventory availability when selected.

## Project Management

- Create, rename, delete, and duplicate projects
- Add recipes via search, set desired quantity per entry
- Reorder entries with drag-and-drop
- Per-entry toggle for intermediate expansion
- Projects persist to SQLite across sessions
- Last selected project or group is remembered via `useViewPrefs` and restored on tab mount
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
- **Materials table** — unified view showing Need/Inv/Storage/Shortfall when availability has been checked. Shows all ingredients needed across recipes, with deduplication. Dynamic/wildcard slots (e.g. "Any Crystal") are included with aggregated inventory from user-configured enabled items (see Dynamic Items tab).
- **Craft or Buy** — centralized list of all craftable ingredients. Toggle each between "craft this" (expand into sub-ingredients) and "buy this" (keep as a leaf material). Toggling applies project-wide across all recipe entries that share that ingredient. Replaces needing to dig into individual recipe entries.

**Right column:**
- **Shopping / Gathering** — items confirmed vendor-purchasable (via CDN sources data) with estimated NPC buy costs. Only items with Vendor/Barter entries in `sources_items.json` appear here — player-set market prices do not affect this list.
- **Source Elsewhere** — items not sold by NPC vendors; need to be found, farmed, crafted, or bought from player shops. Craftable items are tagged.
- **Pickup List** — vault-organized list of items to retrieve from storage

**Full width (top):**
- **Recipe summary** — compact inline list of what you're crafting (recipe name × quantity)
- **Resolve button** — auto-runs on project load; manual re-resolve available

## Intermediate Crafts

Ingredients that can themselves be crafted are identified during material resolution. The "Craft or Buy?" section in the materials panel provides a centralized toggle:
- Items marked for crafting are expanded into their sub-ingredients in the material totals
- The toggle applies project-wide — all recipe entries sharing that ingredient are updated
- Per-entry toggle buttons in recipe cards also apply project-wide for consistency
- "Craft all" / "Buy all" buttons in the section header for bulk toggling
- State is persisted per-entry in `expanded_ingredient_ids` (JSON column)

### How intermediate resolution works

The ingredient resolver (`resolveRecipeIngredients`) marks an ingredient as an intermediate when it finds a producing recipe and the item is in the project's `expandedItemIds` set. The marker is `source_recipe_id` on the `ResolvedIngredient`.

**`flattenIngredients`** builds the leaf material list. Any ingredient with `source_recipe_id` set (an expanded intermediate) is excluded from the flat list — its children (sub-ingredients) are walked instead. Stock-satisfied intermediates (where inventory covers the need) have no children but are still excluded from the flat list.

**`collectIntermediates`** collects all ingredients with `source_recipe_id` set, regardless of whether they have children. This ensures stock-satisfied intermediates still appear in the Craft or Buy section.

**Cross-entry deduplication:** Both materials and intermediates are deduplicated across project entries. Materials sum `quantity` and `expected_quantity`. Intermediates sum `quantity_produced` and `crafts_needed`. This ensures quantities in the Craft or Buy section reflect the total need across all entries in the project.

## Live Crafting Detection

Real-time tracking of crafting progress against active projects or quick-calc targets. Displayed as an overlay panel within the Projects tab (via `LiveCraftingPanel.vue`).

### How it works

- Listens to `player-event` Tauri events for `ItemAdded` and `ItemStackChanged`
- Matches detected item outputs against tracked recipe entries
- Updates crafts completed and detected output quantities in real-time
- Maintains a rolling log of recent craft detections (max 100 entries)

### Starting tracking

- **From Projects tab** — the "Start Tracking" button appears in the materials panel footer when a project is selected and no tracking session is active. Calls `startProjectTracking()` to resolve all project recipe outputs and begin detection.
- **From Quick Calc tab** — the "Track Crafting" button starts single-recipe tracking.

### Tracking modes

- **Project tracking** — monitors all recipe entries in the active project
- **Quick-calc tracking** — monitors a single recipe from the quick calculator

## Database Schema

- `crafting_projects` — id, name, notes, group_name (nullable), created_at, updated_at
- `crafting_project_entries` — id, project_id, recipe_id, recipe_name, quantity, sort_order, expanded_ingredient_ids (JSON), target_stock (nullable)
- Migration v10 added `group_name` and `target_stock` columns
