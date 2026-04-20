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

### 3. `getRecipesForItem` does full linear scan despite existing index

**This is a high-impact, low-effort fix.** The backend already has pre-built `recipes_producing_item: HashMap<u32, Vec<u32>>` and `recipes_using_item: HashMap<u32, Vec<u32>>` indices on `GameData` (see `src-tauri/src/game_data/mod.rs:167-168`). But both `get_recipes_for_item` and `get_recipes_using_item` in `cdn_commands.rs:711-739` ignore these indices and do O(n) full scans over all ~12k recipes, checking `result_item_ids.contains()` on each one.

During resolution, every craftable ingredient calls `getRecipesForItem` via Tauri invoke. Even ingredients that aren't being expanded still get checked for craftability (line 259 of `craftingStore.ts`). For a project with many shared ingredients across entries, the same item ID is looked up repeatedly — each time triggering an IPC round-trip plus a full scan.

**Fix (immediate):**
- Rewrite `get_recipes_for_item` to use `data.recipes_producing_item.get(&item_id)` → lookup recipe IDs → collect from `data.recipes`. Same for `get_recipes_using_item` with `data.recipes_using_item`. Changes O(n) per call to O(k) where k is the number of matching recipes (usually 1-3).

**Fix (additional):**
- Add a client-side cache for `getRecipesForItem` results (recipe data is static within a session). Even with the index fix, each call is still an IPC round-trip. A `Map<number, RecipeInfo[]>` on the crafting store avoids repeated invokes for the same item.
- Batch craftability checks — collect all ingredient item IDs, send one query to check which have producing recipes. A new `check_items_craftable(item_ids: Vec<u32>) -> HashMap<u32, bool>` command would replace N individual calls with one.

### 4. `intermediateStock` queries

Stock is queried for all expanded intermediates at the start of each resolve. This is a single batch query, but it runs on every re-resolve even when stock hasn't changed.

**Potential fix:**
- Cache stock results and only re-query when the user explicitly clicks "Recheck Inventory" or when a crafting event is detected

### 5. Material availability check overhead

`checkMaterialAvailability` runs after every resolve. It queries inventory + storage for all materials, resolves dynamic keywords, fetches vendor prices, and resolves item data in batch.

**Potential fixes:**
- Skip availability re-check when only expansion state changed (the set of materials changes but stock data is the same)
- Incrementally update availability — when materials change, only query newly added items

### 6. `resolveItem` called per-ingredient during tree resolution

Inside `resolveRecipeIngredients`, each ingredient calls `gameData.resolveItem(ing.item_id)` individually (line 266 of `craftingStore.ts`) just to get the display name. This is another IPC round-trip per ingredient.

**Potential fix:**
- Batch item name resolution. Collect all item IDs during the tree walk and resolve names in one `resolveItemsBatch` call at the end, or pass a pre-resolved item name map into the resolver.

## Additional Opportunities

### 7. Lookup tables to pre-build from CDN data

The `GameData` struct already has several cross-reference indices. Additional indices that would help projects:

- **`item_keyword_index: HashMap<String, Vec<u32>>`** — The `getItemsByKeyword` function (used for dynamic/keyword ingredients like "Any Bone") currently does an O(n) scan through all items. A pre-built keyword→item_ids map at CDN load time would make dynamic ingredient resolution O(1). Check if `brewing_keyword_to_items` already covers this — it may only cover brewing keywords, not crafting ingredient keywords.
- **Recipe craftability set: `HashSet<u32>`** — A simple set of all item IDs that appear in any recipe's `result_item_ids`. The `is_craftable` check during resolution just needs to know "does any recipe produce this item?" — it doesn't need the full recipe list unless it's going to expand. This would let us skip the recipe lookup entirely for non-expanded ingredients.

### 8. Data to persist to the database

Currently, ingredient trees are fully recomputed on every resolve. Some of this could be cached:

- **Resolved ingredient tree per entry** — Store the serialized `ResolvedIngredient[]` tree alongside each entry. Invalidate when: recipe CDN data changes (version bump), quantity changes, or expansion set changes. This avoids re-resolving unchanged entries during bulk operations.
- **Intermediate craftability flags** — Store which ingredients are craftable alongside the entry. This is derivable from CDN data and doesn't change between resolves — only on CDN updates.
- **Caveat:** Persisting computed trees adds schema complexity and cache invalidation concerns. A simpler in-memory cache (keyed by `recipe_id + quantity + expandedItemIds hash`) achieves the same benefit without migration burden. Given the project's early alpha status, the in-memory approach is probably better.

### 9. Loading states and progress indicators

The current UI has minimal loading feedback:
- The "Recheck Inventory" button shows "Refreshing..." text when `resolving` is true
- `checkingAvailability` is tracked but not visibly surfaced in the materials panel

**Improvements:**
- **Skeleton/shimmer states for material tables** — When `resolvingAll` is true, show skeleton rows instead of empty space. The user currently sees content disappear and reappear.
- **Per-entry resolution progress** — For large projects, show which entry is currently being resolved (e.g., "Resolving 3/8 entries..."). The sequential entry loop in `resolveProject` makes this straightforward — increment a counter ref in the loop.
- **Availability indicator** — Surface the `checkingAvailability` state visually. Material rows could show a subtle pulse or "checking..." state while availability is being refreshed, rather than showing stale data that silently updates.
- **Intermediate toggle feedback** — When toggling craft/buy on an intermediate, there's a lag before the materials update. A brief inline spinner on the toggled row would communicate that work is happening.
- **"Recheck Inventory" should be a proper button** — not just a text toggle. Consider making it more prominent since it's the explicit "I want fresh data" action.

### 10. Card and panel organization

The current three-pane layout puts materials center and configuration right. Some structural questions:

- **Keep everything on one screen** — Having materials, shopping, and pickup all visible without tab-switching is important for usability. No tabs.
- **Accordion the recipe summary at the top** — The list of "what you're crafting" at the top of the center panel takes significant vertical space on large projects. It should be an accordion section (collapsed by default once the project is resolved, since you already know what you're crafting). Expand to review/verify, but don't eat screen real estate by default.
- **Split "Materials" into "Raw Materials" and "Intermediates"** — The current materials accordion mixes leaf materials (things you need to gather/buy) with intermediate crafts (things you need to craft). These serve different workflows and splitting them would:
  - Make the "what do I need to go get?" question immediately answerable without mentally filtering out intermediates
  - Let the intermediates section show craft-specific info (craft count, recipe name, craft order/dependencies) without cluttering the raw materials table
  - Enable showing intermediates in dependency order (craft A before B before C), which the flat mixed list can't express
  - The "Craft or Buy?" toggles naturally belong in the intermediates section — toggling an item between craft/buy moves it between sections
  - Raw materials section focuses on: item, quantity needed, quantity on hand, shortfall, source (vendor/gather)
  - Intermediates section focuses on: item, crafts needed, recipe used, ingredients (collapsed), stock on hand
- **Entry cards could be collapsible** — For projects with many entries, the right panel gets long. Collapsible entry cards (showing just recipe name + quantity when collapsed) would reduce scroll distance.
- **Group view loses the right panel entirely** — When viewing a group summary, the configuration panel disappears. This makes sense (you can't edit individual entries across projects), but the center panel doesn't expand to use the space. Consider making the center panel full-width in group view.

## Architecture Observations

- The `resolveRecipeIngredients` function is recursive and async. Each level awaits `getRecipesForItem` and potentially `resolveRecipeIngredients` again. Deep intermediate chains multiply the async overhead.
- The `flattenIngredients` and `collectIntermediates` functions are synchronous and fast — they walk the already-resolved tree. The bottleneck is the async resolution, not the post-processing.
- The watcher on `store.activeProject` is the main trigger for cascading resolves. Any code path that calls `loadProject` (including `updateEntry`) triggers it.
- The `updateEntry` function always calls `loadProject` after the DB write (line 171 of `craftingStore.ts`), which triggers the activeProject watcher, which calls `resolveProject`. This means every `updateEntry` in a batch triggers a full resolve cycle, even though only the last one matters.

## Recommended Priority Order

1. **Fix `get_recipes_for_item` to use existing index** — Trivial code change, eliminates the biggest per-call bottleneck
2. **Add client-side recipe cache** — Small frontend change, eliminates repeated IPC round-trips
3. **Batch `updateEntry` for intermediate toggles** — Add a `batch_update_entry_expansions` Tauri command that updates all entries in one call without triggering `loadProject` per entry
4. **Add loading states** — Better UX while the above improvements are in progress and for genuinely large projects
5. **Accordion the recipe list + split raw/intermediate materials** — Reclaim vertical space and clarify the two workflows
6. **In-memory ingredient tree cache** — More complex but eliminates redundant resolution for unchanged entries
7. **Pre-build keyword→items index** — Helps dynamic ingredient resolution
