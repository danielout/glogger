# TODO

Small tasks and notes that don't belong in a dedicated plan.

*Last reviewed: 2026-04-23*

---

## Investigations (Completed Research, No Code Changes Needed)

These are investigated items kept for reference — the research is done but the underlying limitation or blocker remains.

- **Item mods/augments not shown in inventory** — Player.log does NOT include TSys mod/augment data. Only `ProcessAddItem` (name + instance_id) and `ProcessUpdateItemCode` (stack size + type ID) are available. TSys data is only in VIP Inventory JSON export (snapshot imports already store it). Fundamental log format limitation.
- **Equipment display limited** — Equipment IS tracked from Player.log via `ProcessSetEquippedItems`, but only provides `slot` + `appearance_key` — no item names, stats, or details. A basic display could be built showing appearance slots only. Full details require VIP JSON export.
- **Match-3 winnings tracker** — No log events exist for the Match-3 minigame. Zero references in parser, chat logs, or Player.log. No feasible path without game-side changes. Minigame niche.
- **Monsters and Mantids winnings tracker** — Same as Match-3: no log events exist. No feasible path without game-side changes. Minigame niche.
- **Hot tips tracker** — No log events, chat channels, or CDN data references to "hot tips" found in current samples. Need to collect samples, or just find what Kaeus has...

---

## Quick Wins (Small Effort, Noticeable Value)

- [ ] Bug: instant-snack foods missing from gourmand report
  - They used to show up and now they're gone. Static analysis of the code path (CDN parsing, DB query, store filtering, UI display) shows everything is wired correctly. Most likely cause: `parse_food_desc()` in `cdn_persistence.rs` silently skips items where parsing fails — items with unparseable `FoodDesc` are logged but never inserted into the `foods` table. Could also be a whitespace/encoding issue in CDN data (e.g. non-breaking space causing the split to miss). Needs runtime debugging — query the `foods` table directly to see if instant-snack rows exist at all.
  - **Effort: Low | Impact: Medium (data correctness — entire food category invisible)**

- [ ] Bug: rez counter not working
  - Should be counting but isn't. The full pipeline (parser → coordinator → DB → store → widget) looks correctly wired. **Most likely cause:** serde enum case mismatch. `ChatResuscitateEvent` uses `#[serde(tag = "kind")]` but has no `rename_all` directive — if Tauri or middleware applies snake_case serialization, the variant serializes as `"resuscitated"` instead of `"Resuscitated"`, causing the frontend `payload.kind === 'Resuscitated'` check to silently fail. DB persistence works independently (inserts directly), so the bug is likely live-event-only. Fix is probably 1 line: add `#[serde(rename_all = "PascalCase")]` or lowercase the frontend check.
  - **Effort: Low | Impact: Low**

- [x] Clean up documents folder structure
  - Completed: archived finished plans, moved research/architecture docs to proper folders, updated index.md with Plans and Archive sections, sorted TODO items.

- [ ] Crafting projects: loading skeleton/shimmer states for material tables
  - When `resolvingAll` is true, show skeleton rows instead of empty space. From archived projects-performance plan (top 3 perf fixes already landed).
  - **Effort: Low | Impact: Medium (UX polish)**

- [ ] Crafting projects: accordion recipe summary + split raw/intermediate materials
  - Recipe list at top takes too much space on large projects — should be collapsible. Materials should split into "Raw Materials" (things to gather/buy) and "Intermediates" (things to craft), shown in dependency order.
  - **Effort: Medium | Impact: Medium (usability)**

- [ ] Crafting projects: in-memory ingredient tree cache
  - Cache resolved ingredient trees keyed by `recipe_id + quantity + expandedItemIds hash`. Avoids re-resolving unchanged entries during bulk operations. Better than DB persistence at this stage.
  - **Effort: Medium | Impact: Medium (performance)**

- [ ] Crafting projects: pre-build `item_keyword_index` for dynamic ingredients
  - `getItemsByKeyword` (used for "Any Bone" etc.) currently does O(n) scan. Pre-built `HashMap<String, Vec<u32>>` at CDN load time would make it O(1).
  - **Effort: Low | Impact: Low-Medium (performance for keyword-heavy recipes)**

- [ ] Manual recipe adding to brewery
  - Let users manually add recipes/discoveries to the brewery journal instead of requiring JSON import.
  - **Effort: Low-Medium | Impact: Medium (accessibility)**

- [ ] Stack size in item tooltips
  - Show stack size information in item tooltips.
  - **Effort: Low | Impact: Low (polish)**

- [ ] "Home zone" setting for route planner
  - User-set per character. Option to start routes from home zone instead of current location.
  - **Effort: Low | Impact: Medium (convenience)**

- [ ] Configurable critical resources widget
  - Currently fully hardcoded — `CriticalResourcesWidget.vue` has a static `DEFAULT_TRACKED_ITEMS` array (`Diamond`, `Amethyst`, `Aquamarine`, `Eternal Greens`, `Salt`, `Fire Dust`). No config component registered in `dashboardWidgets.ts` (unlike other configurable widgets). Needs: config store per character, item selection UI, settings persistence. Widget already accepts computed data, just needs the binding change.
  - **Effort: Low-Medium | Impact: Medium (personalization)**

- [ ] Investigate detecting recipe learning without character.json import
  - **Investigation complete:** `ProcessUpdateRecipe(recipeId, completionCount)` and `ProcessLoadRecipes()` events already exist in `player_event_parser.rs`. `RecipeUpdated` events fire during gameplay when recipes are updated. Log events ARE generated — this feature is implementable without character.json dependency. Remaining work: wire coordinator handler to persist recipe state changes, update cook's helper to consume live events.
  - **Effort: Low-Medium (wiring only) | Impact: Medium-High (reduces manual import dependency)**

---

## Medium Effort, High Value

- [ ] Manual food eaten/not eaten marking for gourmand
  - Users should be able to manually mark foods if they can't get their skill report. Fallback for when auto-import isn't available.
  - **Effort: Medium | Impact: Medium (accessibility)**

- [ ] Bug: incorrect survey session start/end times
  - **Root causes identified:** (1) Auto-started sessions set `started_at` to the triggering event timestamp, not actual activity start. (2) `recompute_session_bounds_and_end()` only corrects bounds when a session closes — live/open sessions show uncorrected timestamps. (3) During initial log replay, if a manual session is started during catch-up phase, `started_at` uses wall-clock time instead of log date. Fix path: proactively update bounds for open sessions (not just at close), and use `base_date_override` for timestamps during catch-up mode.
  - **Effort: Medium (investigation done, fix touches replay/live mode distinction) | Impact: Medium (data accuracy)**

- [ ] Bug: cook's helper not updating after buying new recipes
  - **Root cause identified:** Cook's helper loads recipes from CDN `items` data via `get_all_foods()`, but new recipes are only detected via character.json re-import or Books SkillReport file re-import. No live event integration exists. Fix: hook into `RecipeUpdated` parser events (which already fire — see recipe detection item above) to refresh the foods table incrementally.
  - **Effort: Low-Medium (depends on recipe detection wiring) | Impact: Medium**

- [ ] Bug: occasional inventory item miscounts
  - **Root causes identified:** (1) Chat-to-player event correlation uses a ±2 second window — mismatches fall back to stack_size=1. (2) Parser's 1-line lookahead buffer for deletions (`pending_deletes`) can fail if log lines reorder. (3) During catch-up replay, stack count corrections from chat may arrive before/after Player.log events depending on replay ordering. (4) Login doesn't treat inventory as a full state dump, so items from earlier sessions can pollute later ones. The widget itself warns "item tracking is not great right now." Fix requires reconciliation logic and smarter replay-mode handling.
  - **Effort: Medium-High (multiple interacting root causes) | Impact: Medium (data accuracy)**

- [ ] Smarter gamestate saving and initializing
  - **Investigation complete:** Three core problems: (1) `ProcessAddItem` always records stack_size=1 as defensive default; `correct_stack_from_chat()` patches this via status messages but timing is fragile during replay. (2) No detection of "login reload" full-state dumps vs. incremental changes — items are UPSERTed individually, never cleared on login during catch-up. (3) During catch-up replay (`live_mode=false`), inventory is not cleared on character login, so items from earlier replayed sessions pollute later ones. Fix requires: detecting login-phase full-state dumps, intelligently clearing transient state, coordinating stack-correction timing with chat log replay order.
  - **Effort: Medium-High (underestimated at Medium — touches replay sequencing, state isolation, transaction ordering) | Impact: Medium (data accuracy)**

- [ ] Changelog formatting improvement
  - Current in-app changelog rendering is poor. Needs better formatting/styling.
  - **Effort: Medium | Impact: Medium (polish)**

- [ ] Help popup redesign
  - Current help popup is ugly. Could be made much prettier.
  - **Effort: Medium | Impact: Medium (polish)**

- [ ] Color standards write-up and enforcement
  - Colors are scattershot across the app. Need documented standards and consistent usage.
  - **Effort: Medium | Impact: Medium (visual consistency)**

- [ ] Font size/family/color readability audit
  - Evaluate typography across the app for readability and consistency.
  - **Effort: Medium | Impact: Medium (accessibility/polish)**

- [ ] More shared components (tables, etc.)
  - Build reusable components for common patterns like tables to improve consistency. Audit existing components for duplicate behavior that should be consolidated.
  - **Effort: Medium (iterative) | Impact: Medium (consistency)**

- [ ] Better screen persistence across the app
  - Currently `useViewPrefs()` + settings store persists pane collapse/width and card order/visibility. **Gap:** no deep navigation state persisted — selected entity, scroll position, sub-tab state all reset on navigation. Main app state uses simple `ref(currentView)` with no persistence. Could extend `useViewPrefs` pattern to store sub-tab + context across sessions.
  - **Effort: Medium | Impact: Medium (UX)**

- [ ] Item provenance downstream features
  - Now that provenance is in the transaction ledger (item-provenance plan phases 1-5 complete), new analytics become possible: mining node yield stats per node type, vendor purchase history with total spend, kill loot breakdown by mob type, crafting yield analysis per recipe, "unknown source" diagnostic reports for discovering new signal patterns.
  - **Effort: Medium per feature | Impact: Medium-High (analytics depth)**

- [ ] Hoplology (equipment study) tracker
  - Track "carefully studied" messages from Player.log, record studied items per character, show completion % by equipment category with CDN data, 5-minute study cooldown timer. Small parser addition + simple CRUD table.
  - **Effort: Medium | Impact: Medium**

- [ ] Boss kill loot timers
  - Currently only player *deaths* are tracked (via `ChatCombatEvent::PlayerDeath` with killer detection). No reverse tracking exists (player kills boss). Would need to extend `chat_combat_parser` or `player_event_parser` for enemy kill events with boss identification. Loot timer logic would layer on top.
  - **Effort: Medium-High (parser extension + new feature) | Impact: Medium**

- [ ] Enemy database in data browser
  - Add enemies as a browsable entity type in the data browser.
  - **Effort: Medium | Impact: Medium (completeness)**

- [ ] Storage view: "show totals" mode
  - Items stored in multiple locations should optionally collapse into a single row with total quantity. Accordion to expand and see per-location breakdown.
  - **Effort: Medium | Impact: Medium (usability)**

- [ ] Color theme support
  - Investigate whether supporting user-selectable color themes makes sense. Low priority but high delight.
  - **Effort: Medium (investigation + implementation) | Impact: Low (personalization/delight)**

- [ ] Area tooltips with useful information
  - Add informative tooltips when hovering area references.
  - **Effort: Medium | Impact: Medium (discoverability)**

- [ ] Evaluate ingestion pipeline / coordinator architecture
  - **Partial investigation (see `docs/architecture/pipeline-structure.md`):** Coordinator is a manual dispatch hub (~2K lines). No formal event bus — each new feature adds match arms. No history/audit tables for most domains (only `item_transactions` is append-only). No shared schema contract for Tauri event payloads. Current design works at this scale but these are real gaps to watch as features grow.
  - **Partial investigation:** Two-tier architecture: `PlayerLogWatcher` → `PlayerEventParser` → `GameStateManager` (50-event batches, 20ms flush window). Chat parallel: `ChatLogWatcher` → bulk insert + per-event status parsing. Main concerns: (1) chat-to-player correlation window is tight at ±2 seconds, (2) no query-side indexing on `game_state_inventory` for per-character/per-server lookups, (3) pending chat gains buffer ages items after 10s with no metrics on correlation failure rate, (4) all inventory deletes require 1-line lookahead. Overall reasonable design — main focus should be correlation tuning and reconciliation.
  - **Effort: Medium | Impact: Medium (reliability)**

- [ ] Standardize skeletons and loading states
  - Create reusable skeleton/loading components for consistent loading UX across screens.
  - **Effort: Medium | Impact: Medium (polish/consistency)**

- [ ] Document standards around persistence, data access, naming
  - Write up conventions so development stays consistent.
  - **Effort: Medium | Impact: Medium (maintainability)**

- [ ] Timer widget
  - General-purpose timer widget for the dashboard. Related to the general-purpose timer system in Larger Efforts.
  - **Effort: Medium | Impact: Medium**

- [ ] Show work order completion state in tooltips for all characters
  - Work order tooltips should show completion/cooldown state across all tracked characters.
  - **Effort: Medium | Impact: Medium (multi-character awareness)**

- [ ] Recurrent event timer widget
  - User-configurable recurring timers (e.g., "every other Wednesday @ 10") with countdowns. Distinct from the general-purpose timer system — this is calendar-based recurrence.
  - **Effort: Medium | Impact: Medium (scheduling)**

- [ ] Audit time handling across the app
  - Suspicious that time handling isn't fully standardized. Need to dig in and verify consistency (timezones, UTC vs local, formatting).
  - **Effort: Medium (investigation) | Impact: Medium (correctness)**

- [ ] Dashboard widget sizing pass — consistent heights
  - Widgets have inconsistent `max-h-*` values (some 80, some 52, some 40, many have none). No systematic height management exists. Need a standardized sizing approach in `DashboardCard.vue` or the widget registry.
  - **Effort: Medium | Impact: Medium (visual consistency)**

- [ ] Dashboard widget sizing pass — consistent widths across three sizes
  - The sizing system exists in `dashboardWidgets.ts` (`small`/`medium`/`large` → col-span classes), but the responsive `auto-fill` grid means actual widths vary with viewport. `col-span-4` assumes enough columns exist. Needs either a fixed column count or size-aware breakpoints.
  - **Effort: Medium | Impact: Medium (visual consistency)**

- [ ] Investigate dashboard refresh issues
  - Reported as "weird page refresh issues." No explicit refresh/reload logic in `DashboardView.vue`. Main reactive trigger is `orderedWidgets` computed property tied to `useViewPrefs` + settings store. `useViewPrefs` debounces writes (500ms) but mutations can fire during transitions. Likely caused by reactive store updates triggering unexpected re-renders on pane resizing or settings updates. Needs runtime debugging to reproduce.
  - **Effort: Low-Medium (once reproduced) | Impact: Unknown**

- [ ] Switch report detection from folder polling to chat log events
  - Currently the Reports folder is polled on a timer (`characterStore.ts` `startReportWatching()`, configurable 5–300s interval). The chat log already announces when exports happen. Switching to chat-log-triggered detection would be more responsive and eliminate unnecessary polling. Needs a new handler in the chat status parser for export messages.
  - **Effort: Medium | Impact: Small-Medium (efficiency/responsiveness)**

- [ ] Actually implement audio alerts for watchwords
  - The "Play sound" checkbox exists in the rule editor UI (`WatchwordsView.vue`) and the setting is stored in `WatchNotifyConfig`. Backend emits `watch-rule-triggered` event with `notify: WatchNotifyConfig` containing the `sound` flag. **Missing:** zero frontend listeners for the emitted event; no audio file loading or playback logic. Needs: event listener, preloaded audio file(s), `new Audio().play()` on match when `notify.sound === true`.
  - **Effort: Medium | Impact: Medium (key alerting feature)**

- [ ] Market Prices screen needs better layout
  - Currently a simple card-based vertical layout inside EconomicsView's PaneLayout. Table columns are fixed (Item | Price | Notes | Updated | Actions). Could benefit from a two-pane layout, better spacing, and visual hierarchy. Filtering already exists. No pricing history, charts, or comparative features.
  - **Effort: Medium | Impact: Medium (usability)**

- [ ] Bulk price setting for market values
  - Currently single-item add/edit only. Import/export exists for JSON data migration but no in-app bulk operations. Could add multi-select with batch price update, percentage adjustments, or category-based pricing. Would need DB schema additions for pricing tiers/rules.
  - **Effort: Medium-High | Impact: Medium (power-user workflow)**

- [ ] Better UX for adding market prices
  - Current add flow could be streamlined. Related to market prices layout and bulk price setting items above.
  - **Effort: Medium | Impact: Medium (usability)**

- [ ] Continue UI/UX standardization across screens
  - Some screens still don't look like they fit within the app, or have their own paradigms. Sidebars that don't use standardized panels, inconsistent patterns, etc. Should write a consolidated UI/UX checklist for new frontend features and then do a pass on existing screens against it.
  - **Effort: Medium (iterative) | Impact: Medium (consistency/polish)**

- [ ] Investigate seedling/plant/milling product linkage in CDN data
  - **Partial investigation:** CDN items have `bestow_recipes: Vec<Value>` field that links items → recipes (e.g. `"recipe_1234"`). Architecture supports building the full product chain (seedling → plant → milling product) but no code currently traverses these links. Would need to query `bestow_recipes` relationships and build a chain-walking query.
  - **Effort: Low-Medium (data exists, needs traversal code) | Impact: Low-Medium (data browser completeness)**

- [ ] Garden almanac widget
  - Saves almanac data when you check it in-game. The almanac shows daily bonus (item + zone for guaranteed double-yield), rotating at midnight EST. `ProcessBook` parser event already exists and emits `BookOpened { title, content, book_type }`, but only `PlayerShopLog` is handled in the coordinator. **Blocker resolved:** capture analysis confirms book_type is `"GardeningAlmanac"` with parseable HTML content including current events (crop + zone + time remaining) and upcoming events. See `docs/architecture/capture-analysis-results.md` and sample data in `docs/samples/devtolsCaptures/gardening-almanac-01.json`. Next: add coordinator handler, new SQLite table, frontend widget using `ItemInline`/`AreaInline`.
  - **Effort: Medium**

- [ ] General-purpose timer system (mushroom barrels, brewing, cheesemaking, fletching, boss respawns)
  - All these skills share a real-time waiting pattern with no log events for the timer portion. Mushroom barrel timers, brewing cask aging (1–3h), cheesemaking aging (1–9h), and fletching drying (1–30m, daylight+sunny only) would all need manual-entry timers. Could share a single reusable timer system. Also subsumes boss/chest respawn timers (from Kaeus's tools). Talk to buppis for brewing specifics. **Partial update:** `ProcessUpdateDescription` does fire for timed crafting items while the player is nearby (e.g. "Rising Simple Sourdough" with proofing countdown and increasing scale value). This provides live progress for items in proximity but won't help with offline/away timers. See `docs/architecture/capture-analysis-results.md`. **Parser confirmed:** `parse_update_description` exists in `player_event_parser.rs` emitting `EntityDescriptionUpdated` events, but the coordinator has no handler for it yet. **DB approach:** `user_timers` table with label, duration, area grouping, `last_triggered_at`. Frontend-driven countdowns with backend persistence (the current milking timer pattern).
  - **Effort: Medium-High**


---

## Larger Efforts / Research Needed

- [ ] Quest tracking system (work orders, repeatables, Statehelm, active quests)
  - **See `docs/plans/quest-tracking.md`** — consolidated plan covering quest event parsing, repeatable cooldown tracking, work order cooldowns, Statehelm quest tracking, and active quest browsing. Prerequisite: add quest events to the player event parser.
  - **Effort: Large | Impact: High**

- [ ] Skillbook autowatchwords
  - Automatically watch chat for item links of skill books the player could learn but doesn't own or know. Two modes: (1) books for currently-trained skills, (2) "future skill" mode where players select skills and we watch for any skillbooks they don't already own/know, even if skill level is too low to use them yet. **Investigation:** watchword system is fully operational (`watch_rules.rs`) with pattern matching and toast notifications. CDN has skillbook items with pattern `Skillbook_*`/`SkillBook_*` (e.g. `Skillbook_FoxInABox` with `AbilityRecipe` keyword). Implementation requires: (1) mapping skillbooks → skills via CDN items, (2) filtering against player's known skills, (3) auto-generating watch rules for desired skillbooks.
  - **Effort: Large | Impact: High (proactive skill progression)**

- [ ] Standardize search across the app (scryfall-inspired)
  - Search is implemented differently in different places. Need a smart, unified search system. Take inspiration from Scryfall's search syntax (https://scryfall.com/docs/syntax) for filtering and querying. Note: some game entity names contain `:` so syntax needs to account for that. Search should also cover descriptions, effects/mods, quest details/objectives — currently we search names only for most things.
  - **Effort: Large | Impact: High (UX consistency, power-user feature)**

- [ ] Write 'how to use' docs for each screen
  - Even brief docs for each screen would help users understand features. Could be in-app help or docs folder.
  - **Effort: Large (breadth) | Impact: Medium (onboarding/discoverability)**

- [ ] Reevaluate test suite
  - Think about what tests make sense, what isn't giving value, and how to harden against future failures.
  - **Effort: Large | Impact: Medium (reliability/confidence)**

- [ ] Analyze what should move from frontend to Rust
  - Some frontend logic may be better served on the Rust backend. Needs analysis to identify candidates.
  - **Effort: Large (research) | Impact: Medium (performance/architecture)**

- [ ] Statehelm repeatable quest tracking — see `docs/plans/quest-tracking.md` §3
  - Sub-task: track statehelm renown possible vs earned.
  - **Effort: High | Impact: Medium-High**


- [ ] Casino arena bet tracker
  - Parse Player.log for arena fight announcements, bet confirmations, outcomes. Parse chat for arena NPC messages. Track bet history with win/loss stats and P&L. Needs a cross-source state machine (Player.log + chat correlation) — similar pattern to survey aggregator. Originally from Kaeus's GorgonBetTracker. Niche but popular feature.
  - **Effort: Large | Impact: Medium (niche but high engagement)**

- [ ] Interactive zone maps — see `docs/plans/interactive-maps.md`
  - **Effort: Large (mostly frontend) | Impact: High**

- [ ] CraftingCorner / community marketplace
  - Community platform for player crafting services (artisan listings, order tracking). Architecturally very different from glogger's local-first model — would need external service integration (Firebase or custom backend). Consider keeping as external link initially. Originally from Kaeus's GorgonCraftingTools.
  - **Effort: Very Large | Impact: Medium (community feature)**

- [ ] Consolidate storage helper
  - Uses the route planner and storage vault data to find items stored in multiple locations, then creates a pickup/dropoff route to consolidate them.
  - **Effort: Large | Impact: Medium (inventory management)**

- [ ] Statehelm favor planner
  - Looks at storage vaults, finds appropriate gifts for Statehelm NPCs, creates a route to pick everything up before delivering. Respects remaining gift count needed per NPC that week. Combines storage data + gift preferences + route planner.
  - **Effort: Large | Impact: High (cross-system planning)**

- [ ] Work order route/crafting planner
  - Looks at all available work orders, shows where to go/craft/turn-in in optimal order using route planner. Depends on quest tracking system for work order state.
  - **Effort: Large | Impact: High (cross-system planning)**

- [ ] Quest turn-in helper
  - Looks at storage/inventory and active quests to find completable quests. Suggests pickup and turn-in routes using route planner. Depends on quest tracking system.
  - **Effort: Large | Impact: High (cross-system planning)**

- [ ] Nightmare cave challenge door tracker
  - Need to look up all the challenges and see which ones we can track. Some are easy (1200 armor) and some are harder (have 4x 10-second premonition buffs). Could also track letters of authority as alternate path. **No existing code found** — no parser events, coordinator handlers, or database tables. Requires research into all challenge types + log event identification + new persistence layer.
  - **Effort: Large (research + implementation) | Impact: Medium (niche but useful)**

- [ ] Gardening helper
  - Should be able to detect seeds, fertilizer, water in inventory. Could also track what nearby plants need. **Research resolved:** `ProcessUpdateDescription` fires for every nearby garden plot with full state (Thirsty/Hungry/Growing/Ripe + crop name + action needed). **Parser confirmed:** `parse_update_description` in `player_event_parser.rs` emits `EntityDescriptionUpdated` events with full garden state. **Missing:** coordinator handler for `EntityDescriptionUpdated`, database persistence for plant states, frontend dashboard with inventory integration. Combined with inventory data, this enables a real-time gardening dashboard. See `docs/architecture/capture-analysis-results.md` for the full state machine and sample data.
  - **Effort: Medium-Large (implementation only, research done) | Impact: High**

- [ ] Macros or process interaction
  - Can we target the game process and send commands? Can we screen-read the process? Major research question with significant technical and policy implications.
  - **Effort: Large (research) | Impact: Unknown (depends on feasibility)**

- [ ] Track total owned quantity changes over time
  - Start with currencies, expand to other items. Currencies may be in character JSON, inventory JSON, or both. PG doesn't overwrite inventory reports in the reports folder — could backfill historical data from old reports. Character JSONs are overwritten on export though. **Infrastructure gap:** database supports point-in-time snapshots only (`character_item_snapshots`). `sales_history` tracks vendor transactions but no time-series quantity tracking exists. Needs: new tables for quantity history, periodic delta recording (event-driven or snapshot-based), UI for time-range queries and charts.
  - **Effort: Large (new infrastructure) | Impact: High (trend data is very valuable)**

- [ ] Auto cleanup of old exported/saved PG data
  - Old chatlogs, reports, etc. Let the user set their retention policies. Needs UI for policy config and safe file deletion logic.
  - **Effort: Large | Impact: Low-Medium (housekeeping feature)**
