# Inventory: Consolidate Tab

The Consolidate tab helps optimize storage by identifying items scattered across multiple vaults and generating a zone-by-zone plan to consolidate them.

## Overview

When items of the same type exist in multiple storage vaults, each additional stack wastes a vault slot. The consolidation tool:

- **Analyzes** all storage vaults to find duplicate item stacks
- **Generates** a zone-by-zone action plan (what to pick up and drop off at each stop)
- **Provides a wizard** that tracks your current zone and shows what to do there
- **Calculates** how many storage slots you'll save

## How It Works

### Plan Generation

The tool scans `game_state_storage` and groups items by name across all vaults. Items appearing in 2+ vaults become consolidation candidates. For each, the vault with the largest quantity is chosen as the target — all other stacks should move there.

Moves are classified into three types:
- **Pick Up to Carry**: items you'll take from a vault and transport to another zone
- **Drop Off (from travel)**: items you're carrying from a previous zone to deposit here
- **Rearrange Locally**: items moving between vaults in the same zone (no travel needed)

### Zone Ordering

Zones are ordered to ensure logical flow:
1. **Pure source zones** (pickups only) — visit first to load up
2. **Swap zones** (both pickups and dropoffs) — pick up new items and deposit carried ones
3. **Pure destination zones** (dropoffs only) — visit last to deposit everything
4. **Local-only zones** — rearrange at your convenience

### Portable Storage

Vaults with area `"*"` (Saddlebag, Council Storage, Boxes of Space, Home Vault, etc.) are treated as always-accessible — no pickup stops are generated for them since the player can access them anywhere.

## UI Modes

### Plan Mode (default)

Shows the full consolidation plan:
- **Summary bar**: slots saveable, items to move, zones involved
- **Zone cards**: responsive grid showing each zone's actions grouped by type
- **Plan Route button**: calls the trip router for optimal zone visit order
- **Checkboxes**: manually mark items as moved

### Wizard Mode

Activated via "Start Wizard" button:
- **Current zone panel**: gold-highlighted section showing what to do at your current location
- **Progress bar**: tracks completion across all zones
- **Remaining zones**: shown below in the grid, current zone excluded (it's highlighted above)
- Reacts to zone changes via `gameState.world.area`

## Route Planning

Uses the existing [trip_router](../../../src-tauri/src/trip_router.rs) — same engine as the Trip Planner widget. Reads travel config (bind pads, mushroom circles, TP machine) from the trip planner's localStorage settings.

## Key Files

- [ConsolidateTab.vue](../../../src/components/Inventory/ConsolidateTab.vue) — UI component
- [useStorageConsolidation.ts](../../../src/composables/useStorageConsolidation.ts) — plan generation composable
- [InventoryWrapper.vue](../../../src/components/Inventory/InventoryWrapper.vue) — tab container
- [gameStateStore.ts](../../../src/stores/gameStateStore.ts) — storage data source

## Limitations

- Does not yet suggest type-specific vault optimization (e.g., moving potions to a potion-only vault)
- Does not auto-detect item pickups/dropoffs from game events (manual checkbox only)
- Route planning requires trip planner travel config to be set for accurate results
- Some zones may not be in the zone graph and will be skipped
