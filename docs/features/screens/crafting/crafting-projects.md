# Crafting — Projects

## Overview

Users create named projects containing multiple target crafts. The helper aggregates all ingredient requirements, deduplicating shared materials.

## Project Management

- Create, rename, delete, and duplicate projects
- Add recipes via search, set desired quantity per entry
- Reorder entries with drag-and-drop
- Per-entry toggle for intermediate expansion
- Projects persist to SQLite across sessions

## Material Breakdown

For the entire project, the material summary shows:
- **Total ingredients needed** — summed across all recipes
- **Inventory stock** — what the player has on-hand (live tracking)
- **Storage stock** — from latest inventory snapshot, broken down by vault
- **Shortfall** — items still needed, with estimated vendor cost

## Pickup List

A vault-organized "go get these items" list:
- Groups needed materials by storage vault location
- Shows quantity to pick up from each vault
- Sorted so the player can visit each vault once

## Shopping List

Items the player doesn't have anywhere:
- Estimated vendor cost per item (`value × 1.5`)
- Total gold needed for all missing materials

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
