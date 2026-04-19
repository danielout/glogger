# Crafting Projects — Performance Improvements

## Problem

Large crafting projects with many entries and expanded intermediates exhibit noticeable lag, especially when toggling craft/buy state. The current architecture re-resolves the entire project on every change, and several structural issues compound the cost.

## Known Issues

### 1. Cascading re-resolves on toggle

When toggling an intermediate craft/buy, `toggleIntermediateGlobal` updates every entry's `expanded_ingredient_ids` in the DB. Each `updateEntry` call triggers `loadProject` → watcher → `resolveProject`. For a project with N entries, this fires N redundant resolves before the final correct one completes.

**Current mitigation:** DB writes are batched with `Promise.all` instead of sequential `await`. The generation guard (`resolveGeneration`) ensures only the last resolve completes. But the intermediate resolves still fire and do work before bailing.

**Potential fixes:**
- Debounce or batch `resolveProject` calls — skip re-resolve during bulk operations, only resolve once at the end
- Split the `updateEntry` Tauri command so it doesn't reload the full project. Add a dedicated `update_entry_expansions` batch command that updates multiple entries in one round-trip without triggering `loadProject`
- Move intermediate expansion state out of the DB-per-entry model into a project-level set, reducing writes from N (per entry) to 1

### 2. Full project re-resolve on every change

Every toggle, quantity change, or stock target update triggers a full `resolveProject` which:
1. Queries stock for all expanded intermediates
2. Resolves every entry's recipe ingredients (async, sequential per entry)
3. Flattens and deduplicates all materials
4. Checks availability across inventory + storage

For large projects this is expensive. Most changes only affect a subset of entries.

**Potential fixes:**
- Cache resolved ingredient trees per entry. Only re-resolve entries whose inputs changed (recipe, quantity, or expansion set). Merge cached results for unchanged entries with fresh results for changed ones.
- Make entry resolution parallel (`Promise.all` over entries) instead of sequential
- Separate "resolve ingredients" from "check availability" — availability can be checked independently and less frequently

### 3. `getRecipesForItem` called per-ingredient

During resolution, every craftable ingredient check calls `getRecipesForItem` via Tauri invoke. For a project with many shared ingredients across entries, the same item ID is looked up repeatedly.

**Potential fixes:**
- Add a client-side cache for `getRecipesForItem` results (recipe data is static within a session)
- Batch craftability checks — collect all ingredient item IDs, send one query to check which have producing recipes

### 4. `intermediateStock` queries

Stock is queried for all expanded intermediates at the start of each resolve. This is a single batch query, but it runs on every re-resolve even when stock hasn't changed.

**Potential fix:**
- Cache stock results and only re-query when the user explicitly clicks "Recheck Inventory" or when a crafting event is detected

### 5. Material availability check overhead

`checkMaterialAvailability` runs after every resolve. It queries inventory + storage for all materials, resolves dynamic keywords, fetches vendor prices, and resolves item data in batch.

**Potential fixes:**
- Skip availability re-check when only expansion state changed (the set of materials changes but stock data is the same)
- Incrementally update availability — when materials change, only query newly added items

## Architecture Observations

- The `resolveRecipeIngredients` function is recursive and async. Each level awaits `getRecipesForItem` and potentially `resolveRecipeIngredients` again. Deep intermediate chains multiply the async overhead.
- The `flattenIngredients` and `collectIntermediates` functions are synchronous and fast — they walk the already-resolved tree. The bottleneck is the async resolution, not the post-processing.
- The watcher on `store.activeProject` is the main trigger for cascading resolves. Any code path that calls `loadProject` (including `updateEntry`) triggers it.
