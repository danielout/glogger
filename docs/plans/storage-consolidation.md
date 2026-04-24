# Storage Consolidation Helper

Find items scattered across multiple storage vaults and generate optimized pickup/dropoff routes.

## Overview

Builds on "Show Totals" mode in VaultDatabaseTab, which already computes per-item location breakdowns. Items in 2+ locations are consolidation candidates.

## Candidate Identification

Data path: `gameState.storage` → group by `item_name` → filter `locations.length >= 2`. `StorageVaultDetail.area` provides the CDN area key for routing. `required_item_keywords` must be respected for target validation.

Extracted into `src/composables/useStorageConsolidation.ts` for reusability.

## Target Selection Strategies

1. **Most items (default):** Pick vault holding the most of each item — minimizes items to carry
2. **User-specified vault:** Consolidate everything to one chosen location (home base)
3. **Fewest stops:** Find the single area covering the most pickups globally

Constraints: validate target vault free slots, check `required_item_keywords` restrictions.

## Route Generation

Uses existing `plan_trip` Tauri command — no changes to [trip_router.rs](../../src-tauri/src/trip_router.rs) needed.

Stops:
- Pickup stops: one per source vault area (multiple items in same area collapse)
- Deposit stop: target vault area
- Within-zone ordering already handles pickup-before-deposit (StopPurpose ordinals)

Travel config reused from trip planner widget's localStorage config.

## UI

### Primary entry: Show Totals mode in VaultDatabaseTab
- "Consolidate" toolbar button (enabled when candidates exist)
- Checkboxes on candidate rows, bottom action bar with target selector + "Plan Route"
- Multi-location badge indicator on qualifying rows

### Route display
- Inline panel or modal showing planned route steps (reusing TripPlannerWidget step rendering)
- Summary, item list with source/destination, route steps, total hops, capacity warnings

## User Workflow

1. Inventory > Vault Database > enable Show Totals
2. Click "Consolidate" button
3. Select/deselect items, choose strategy
4. Click "Plan Route" → generates and displays route
5. Follow route in-game manually

## Phases

### Phase 1: Composable + candidate logic
- `useStorageConsolidation.ts`: candidate detection, target strategies, capacity validation, RouteStop generation

### Phase 2: UI integration
- Consolidate toggle in VaultDatabaseTab toolbar
- Checkbox support in VaultTotalsRow (via `selectable` prop)
- Action bar with target selector

### Phase 3: Route display
- `ConsolidationRoutePanel.vue`: route steps, capacity warnings
- Call `plan_trip` and display results

### Phase 4: Polish
- Persist preferences, candidate count badge, edge case handling

## Key Files

- [VaultDatabaseTab.vue](../../src/components/Inventory/VaultDatabaseTab.vue) — host view
- [VaultTotalsRow.vue](../../src/components/Inventory/VaultTotalsRow.vue) — per-item row
- [trip_router.rs](../../src-tauri/src/trip_router.rs) — route solver (no changes needed)
- [TripPlannerWidget.vue](../../src/components/Dashboard/widgets/TripPlannerWidget.vue) — route display reference
- [gameStateStore.ts](../../src/stores/gameStateStore.ts) — storage data + vault metadata
