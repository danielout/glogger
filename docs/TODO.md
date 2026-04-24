# TODO

Small tasks and notes that don't belong in a dedicated plan.

*Last reviewed: 2026-04-23 (standardization wave completed: color tokens, typography, widget sizing, shared components, 10 table migrations)*

---

## Investigations (Completed Research, No Code Changes Needed)

These are investigated items kept for reference — the research is done but the underlying limitation or blocker remains.

- **Item mods/augments not shown in inventory** — Player.log does NOT include TSys mod/augment data. Only `ProcessAddItem` (name + instance_id) and `ProcessUpdateItemCode` (stack size + type ID) are available. TSys data is only in VIP Inventory JSON export (snapshot imports already store it). Fundamental log format limitation.
- **Equipment display limited** — Equipment IS tracked from Player.log via `ProcessSetEquippedItems`, but only provides `slot` + `appearance_key` — no item names, stats, or details. A basic display could be built showing appearance slots only. Full details require VIP JSON export.
- **Match-3 winnings tracker** — No log events exist for the Match-3 minigame. Zero references in parser, chat logs, or Player.log. No feasible path without game-side changes. Minigame niche.
- **Monsters and Mantids winnings tracker** — Same as Match-3: no log events exist. No feasible path without game-side changes. Minigame niche.
- **Hot tips tracker** — No log events, chat channels, or CDN data references to "hot tips" found in current samples. Need to collect samples, or just find what Kaeus has...

---

## Blocked: Additional Log Examples Needed

These items are investigated but can't be resolved without new runtime captures or log samples.

- [ ] Bug: instant-snack foods missing from gourmand report
  - They used to show up and now they're gone. Static analysis of the code path (CDN parsing, DB query, store filtering, UI display) shows everything is wired correctly. Most likely cause: `parse_food_desc()` in `cdn_persistence.rs` silently skips items where parsing fails — items with unparseable `FoodDesc` are logged but never inserted into the `foods` table. Could also be a whitespace/encoding issue in CDN data (e.g. non-breaking space causing the split to miss). Needs runtime debugging — query the `foods` table directly to see if instant-snack rows exist at all.
    - the gourmand completeness bars in the left panel show "0/0" for instant-snacks, so seems like we aren't loading them at all. i bet a couple debug lines can root cause this pretty quickly. seems like it all probably works still _if_ we solve why we aren't finding any in the CDN.
  - **Blocked on:** Runtime debugging session — need to query the `foods` table and check CDN parse output.

- [ ] Bug: rez counter not working
  - **Serde hypothesis disproven** — exhaustive static analysis confirms the full pipeline is correctly wired. Parser tests pass (9/9). Debug logging added to coordinator and frontend store.
  - **Most likely actual cause:** Player's in-game chat configuration doesn't include `[Action Emotes]` channel in any chat tab, so those messages never appear in Chat.log.
  - **Blocked on:** Runtime capture with Action Emotes enabled to confirm.

- [ ] Investigate detecting recipe learning without character.json import
  - **Investigation complete:** `ProcessUpdateRecipe(recipeId, completionCount)` and `ProcessLoadRecipes()` events already exist in `player_event_parser.rs`. `RecipeUpdated` events fire during gameplay when recipes are updated. Log events ARE generated — this feature is implementable without character.json dependency. Remaining work: wire coordinator handler to persist recipe state changes, update cook's helper to consume live events.
  - [ ] RELATED Bug: cook's helper not updating after buying new recipes
  - **Root cause identified:** Cook's helper loads recipes from CDN `items` data via `get_all_foods()`, but new recipes are only detected via character.json re-import or Books SkillReport file re-import. No live event integration exists. Fix: hook into `RecipeUpdated` parser events (which already fire — see recipe detection item above) to refresh the foods table incrementally.
    - STILL UNKNOWN: does this fire when a recipe is first learned, even if completion count is zero? need to capture a log for this.
  - **Blocked on:** Log capture of a recipe being learned for the first time.

  - **Effort: Low-Medium (depends on recipe detection wiring) | Impact: Medium**

- [ ] Gardening helper — see `docs/plans/gardening-helper.md`
  - Parser events exist (`EntityDescriptionUpdated`), plan written. But some garden mechanics (fertilizer counts, watering, growth timers) need additional in-game captures before implementation.
  - **Blocked on:** Additional garden captures to document fertilizer/watering/growth mechanics fully.

- [ ] Hoplology (equipment study) tracker — backfill from book/report
  - Chat parser, DB table, coordinator handler, and dashboard widget are all built and wired. Widget is disabled in `dashboardWidgets.ts` because without backfill it always shows 0 studied items.
  - **Blocked on:** Capture of the hoplology skill report book content (opened via ProcessBook in-game). Need the book_type and HTML content format to build the parser. CDN references `HoplologyBook` NPC source.

---

## Quick Wins (Small Effort, Noticeable Value)

- [x] Crafting projects: loading skeleton/shimmer states for material tables
  - Done: ProjectMaterialsPanel now shows SkeletonLoader + DataTableSkeleton sections while resolving instead of spinner text.

- [x] Crafting projects: accordion recipe summary + split raw/intermediate materials
  - Done: Recipe list wrapped in AccordionSection (auto-collapsed >5 recipes). MaterialSummary splits into Intermediates (craftable) and Raw Materials. New MaterialTable component extracts shared table rendering.

- [x] Crafting projects: in-memory ingredient tree cache
  - Done: Map cache in craftingStore keyed by `recipeId:quantity:sortedExpandItemIds`. Caches top-level resolves only (not recursive sub-calls or bulk with mutable stock).

- [x] Crafting projects: pre-build `item_keyword_index` for dynamic ingredients
  - Done: `item_keyword_index: HashMap<String, Vec<u32>>` built at CDN load time, cached to disk. `get_items_by_keyword` now O(1).

- [x] Manual recipe adding to brewery
  - Done: Collapsible form in recipe detail panel with per-slot ingredient dropdowns and optional effect label. Uses existing brewing_discoveries table. Batch entry supported.

- [x] Stack size in item tooltips
  - Already implemented. Fixed to hide "Max Stack: 1" for non-stackable items.

- [x] "Home zone" setting for route planner
  - Done: Per-character home zone stored in localStorage, configurable via trip planner settings panel. "Home" button next to "Current" for quick start zone selection.

- [x] Configurable critical resources widget
  - Done: Config panel with item search/add/remove, persisted via useViewPrefs. Defaults to original 6 items.

---

## Medium Effort, High Value

- [x] Manual food eaten/not eaten marking for gourmand
  - Done: Click-to-toggle eaten status with v40 migration. Visual distinction: green=imported, blue=manual, red=uneaten. Manual marks preserved during report imports.

- [x] Bug: incorrect survey session start/end times
  - Done: Added incremental `tighten_started_at()` that updates open session bounds on every craft/use/loot event. Fixes auto-start trigger timestamps, live session bounds, and catch-up mode correction.

- [ ] Bug: occasional inventory item miscounts
  - **Root causes identified:** (1) Chat-to-player event correlation uses a ±2 second window — mismatches fall back to stack_size=1. (2) Parser's 1-line lookahead buffer for deletions (`pending_deletes`) can fail if log lines reorder. (3) During catch-up replay, stack count corrections from chat may arrive before/after Player.log events depending on replay ordering. (4) Login doesn't treat inventory as a full state dump, so items from earlier sessions can pollute later ones. The widget itself warns "item tracking is not great right now." Fix requires reconciliation logic and smarter replay-mode handling.
  - **Effort: Medium-High (multiple interacting root causes) | Impact: Medium (data accuracy)**

- [ ] Smarter gamestate saving and initializing
  - **Investigation complete:** Three core problems: (1) `ProcessAddItem` always records stack_size=1 as defensive default; `correct_stack_from_chat()` patches this via status messages but timing is fragile during replay. (2) No detection of "login reload" full-state dumps vs. incremental changes — items are UPSERTed individually, never cleared on login during catch-up. (3) During catch-up replay (`live_mode=false`), inventory is not cleared on character login, so items from earlier replayed sessions pollute later ones. Fix requires: detecting login-phase full-state dumps, intelligently clearing transient state, coordinating stack-correction timing with chat log replay order.
  - **Effort: Medium-High (underestimated at Medium — touches replay sequencing, state isolation, transaction ordering) | Impact: Medium (data accuracy)**

- [x] Changelog formatting improvement
  - Done: Structured parsing with AccordionSection per release, color-coded category badges, proper typography. Removed style block.

- [x] Help popup redesign
  - Done: Frosted glass backdrop, side nav with icons, styled 3D keycap badges for shortcuts, card-based sections, gold accents throughout.

- [x] Color standards write-up and enforcement
  - Audit completed: see `docs/architecture/color-standards.md`. 19 semantic tokens added to theme.css. ~250 hardcoded hex values migrated to tokens. ChatMessage channel colors fixed. Component classes updated.
  - **Remaining:** ~30 `text-red-400`/`text-green-400` instances remain where used for UI chrome (delete buttons, hover states) rather than gain/loss semantics. Progress bar colors (`bg-green-500`, `bg-red-500` in vault capacity, crafting progress) could get tokens. Chart palette still outside the theme (needs Vue Data UI integration research). See `docs/architecture/color-standards.md` Tier 3 section for full remaining list.

- [x] Font size/family/color readability audit
  - Audit completed: see `docs/architecture/typography-audit.md`. 13 arbitrary sizes consolidated to 7-level scale. 215 redundant `font-mono` removed. `tabular-nums` added to numeric table columns. No text below 10px remains.
  - **Remaining:** Heading standardization (~170 heading tags across ~60 files still use varying class patterns). Could define `.section-heading`, `.panel-label` etc. component classes in `components.css` to reduce repetition. See typography-audit.md "Heading Classes" section.

- [x] More shared components (tables, etc.)
  - Built: `DataTable.vue` (sortable, loading skeleton, empty state, dynamic cell slots), `FilterBar.vue` (v-model search, result count, filter slots), `SkeletonLoader.vue` (text/circle/rect variants), `DataTableSkeleton.vue`. Migrated 10 tables to DataTable (Character: StatsTable, SkillTable, CurrencyTable, RecipeTable, PlayerAttributesCard; StallTracker: StallSalesTab, StallInventoryTab, StallShopLogTab; Market: MarketView; Settings: DomainTable).
  - **Remaining:** ~30 more tables across Crafting, Surveying, and DataBrowser could be migrated. These tend to have more complex layouts (merged cells, nested sub-tables). SkeletonLoader is built but not yet adopted in existing screens — needs a pass to replace "Loading..." text with skeleton states. ComputedStatsCard intentionally skipped (key-value layout, not columnar).

- [x] Better screen persistence across the app
  - Done: `currentView` in App.vue persisted via `useViewPrefs` (app restarts remember which screen was open). MenuBar sub-tab selections persisted across navigation and restarts. Inner sub-tabs persisted for: Economics Farming, Economics Survey, Stall Tracker, Data Browser sidebar. Remaining gap: scroll position and selected entities still reset on navigation (would require significant complexity for marginal benefit).

- [ ] Item provenance downstream features
  - Now that provenance is in the transaction ledger (item-provenance plan phases 1-5 complete), new analytics become possible: mining node yield stats per node type, vendor purchase history with total spend, kill loot breakdown by mob type, crafting yield analysis per recipe, "unknown source" diagnostic reports for discovering new signal patterns.
  - **Effort: Medium per feature | Impact: Medium-High (analytics depth)**

- [ ] Hoplology (equipment study) tracker
  - **Partially implemented (widget disabled):** Chat parser for "carefully study" messages, hoplology_studies table, dashboard widget with study count, 5-min cooldown timer, and searchable studied items list all built — but widget is disabled in dashboardWidgets.ts because it's useless without backfill.
  - **Blocked on:** Without parsing the hoplology book/report to backfill existing studies, the widget always shows 0. Need a capture of the hoplology skill report book content to build the import. CDN references `HoplologyBook` NPC source but we don't have a sample of the actual book content format.
  - Moved to Blocked section below.

- [ ] Boss kill loot timers
  - Currently only player *deaths* are tracked (via `ChatCombatEvent::PlayerDeath` with killer detection). No reverse tracking exists (player kills boss). Would need to extend `chat_combat_parser` or `player_event_parser` for enemy kill events with boss identification. Loot timer logic would layer on top.
  - **Effort: Medium-High (parser extension + new feature) | Impact: Medium**

- [x] Enemy database in data browser
  - Done: EnemyBrowser with search, strategy filter, detail panel (combat properties, abilities). Backend AiSummary struct + 3 new Tauri commands. Registered in sidebar and overlay.

- [x] Storage view: "show totals" mode
  - Done: Toggle collapses items across locations into total rows with expandable per-location breakdown. Search filters both modes. Preference persisted via useViewPrefs.

- [ ] Color theme support
  - Investigate whether supporting user-selectable color themes makes sense. Low priority but high delight.
  - **Effort: Medium (investigation + implementation) | Impact: Low (personalization/delight)**

- [x] Area tooltips with useful information
  - Done: AreaTooltip.vue shows area name, short name, and notable NPCs (up to 8, via NPC cross-reference). AreaInline updated with EntityTooltipWrapper for hover/click/pin. CDN area data is sparse (no descriptions/level ranges), but NPC enrichment adds useful info.

- [ ] Evaluate ingestion pipeline / coordinator architecture
  - **Partial investigation (see `docs/architecture/pipeline-structure.md`):** Coordinator is a manual dispatch hub (~2K lines). No formal event bus — each new feature adds match arms. No history/audit tables for most domains (only `item_transactions` is append-only). No shared schema contract for Tauri event payloads. Current design works at this scale but these are real gaps to watch as features grow.
  - **Partial investigation:** Two-tier architecture: `PlayerLogWatcher` → `PlayerEventParser` → `GameStateManager` (50-event batches, 20ms flush window). Chat parallel: `ChatLogWatcher` → bulk insert + per-event status parsing. Main concerns: (1) chat-to-player correlation window is tight at ±2 seconds, (2) no query-side indexing on `game_state_inventory` for per-character/per-server lookups, (3) pending chat gains buffer ages items after 10s with no metrics on correlation failure rate, (4) all inventory deletes require 1-line lookahead. Overall reasonable design — main focus should be correlation tuning and reconciliation.
  - **Effort: Medium | Impact: Medium (reliability)**

- [x] Standardize skeletons and loading states
  - Done: SkeletonLoader and DataTableSkeleton adopted across 24 screens (dashboard, crafting, character, data browser, surveying, farming, stall tracker, market, help). Button label toggles and brief inline states intentionally kept as-is.

- [x] Document standards around persistence, data access, naming
  - Done: See `docs/architecture/standards-persistence-naming.md`. Covers naming conventions, persistence decision table, store patterns, Tauri command patterns, migration patterns, type organization.

- [x] Timer widget
  - Done: Countdown timers with add/pause/resume/restart/remove. Flexible duration input, preset quick-start buttons (Mushroom Barrel, Brewing, Cheesemaking, Composting). Configurable presets. Persisted via localStorage with absolute timestamps. Expired timers pulse red.

- [ ] Show work order completion state in tooltips for all characters
  - Work order tooltips should show completion/cooldown state across all tracked characters.
  - **Effort: Medium | Impact: Medium (multi-character awareness)**

- [x] Recurrent event timer widget
  - Done: Calendar-based recurring event timers with countdowns. Supports daily, weekly, biweekly (with anchor date), and monthly recurrence patterns. Events auto-advance to next occurrence after passing. Sorted by soonest next occurrence, imminent events (<1h) highlighted. Persisted via useViewPrefs. Config popover for managing events.

- [x] Audit time handling across the app
  - Done: See `docs/architecture/time-handling-audit.md`. Backend is clean (consistent UTC). Frontend useTimestamp composable well-adopted. Fixed one manual duration formatting violation. Documented handful of low-severity toLocaleDateString bypasses.

- [x] Dashboard widget sizing pass — consistent heights
  - Done: DashboardCard enforces uniform `h-80` (320px) height. All per-widget `max-h-*` overrides removed. Flex-1 scroll pattern applied to all widgets with variable content. Fixed broken scroll on MilkingTimers and WordsOfPower. See `docs/architecture/widget-sizing-audit.md`.
  - **Remaining:** The exact height value (320px) may need tuning after visual testing. Some widgets with very little content leave empty space — could center content vertically where appropriate.

- [x] Dashboard widget sizing pass — consistent widths across three sizes
  - Audited: see `docs/architecture/widget-sizing-audit.md`. Current `auto-fill, minmax(280px, 1fr)` grid approach works at reasonable window sizes. Column count derived from `floor(panel_width / small_widget_width)`.
  - **Remaining:** `col-span-4` can still overflow if window is narrower than 4×280px. Low risk for a desktop Tauri app but could add a CSS `min()` clamp as a safety net.

- [ ] Investigate dashboard refresh issues
  - Reported as "weird page refresh issues." No explicit refresh/reload logic in `DashboardView.vue`. Main reactive trigger is `orderedWidgets` computed property tied to `useViewPrefs` + settings store. `useViewPrefs` debounces writes (500ms) but mutations can fire during transitions. Likely caused by reactive store updates triggering unexpected re-renders on pane resizing or settings updates. Needs runtime debugging to reproduce.
  - **Effort: Low-Medium (once reproduced) | Impact: Unknown**

- [x] Switch report detection from folder polling to chat log events
  - Done: Parses "Saved report to <path>" from chat Status channel. Immediate import trigger via "report-saved" event. Polling kept as fallback at 60s (was 10s).

- [x] Actually implement audio alerts for watchwords
  - Done: watchwordAlertStore plays two-tone ascending beep via Web Audio API when rules trigger with sound enabled. Also shows toast notifications. Listener registered at app startup in startupStore.

- [x] Market Prices screen needs better layout
  - Done: Summary stat cards, add form and valuation settings in AccordionSections, full-width search, bordered table container, ItemInline for names, inline edit with Cancel button.

- [x] Bulk price setting for market values
  - Done: Multi-select checkboxes, bulk action bar with Set Price, Adjust Prices (%), and Delete Selected. Backend bulk_update/bulk_delete commands. Confirmation for destructive ops.

- [x] Better UX for adding market prices
  - Done: Autocomplete item search with ItemInline previews, auto-focus price, Enter/Escape shortcuts, batch entry with success feedback, duplicate detection with update option.

- [ ] Continue UI/UX standardization across screens
  - Some screens still don't look like they fit within the app, or have their own paradigms. Sidebars that don't use standardized panels, inconsistent patterns, etc. Should write a consolidated UI/UX checklist for new frontend features and then do a pass on existing screens against it.
  - **Progress:** Color tokens, typography scale, and widget sizing are now standardized. DataTable/FilterBar/SkeletonLoader shared components are available. 10 tables migrated. Heading classes and remaining table migrations are the next incremental steps.
  - **Effort: Medium (iterative) | Impact: Medium (consistency/polish)**

- [x] Investigate seedling/plant/milling product linkage in CDN data
  - **Done.** Seedlings are identified by `Seedling=XX` keyword. The seedling→plant link is implicit via naming convention (strip "Seedling"/"Leafling"/"Seeds" suffix from InternalName to find the plant item). Plant→product links use the existing `recipes_using_item` index. Implemented `get_gardening_product_chain` Tauri command that walks the full chain, displayed in the Item Browser detail view as a "Gardening Chain" section with inline item/recipe links.

- [x] Garden almanac widget
  - Done: Parses GardeningAlmanac book content, extracts current/upcoming crop bonus events. Dashboard widget with ItemInline/AreaInline display, countdown timer, upcoming events list. New garden_almanac table.

- [x] General-purpose timer system (mushroom barrels, brewing, cheesemaking, fletching, boss respawns)
  - Done: Backend `user_timers` table with save/get/delete commands. Timer store upgraded to SQLite persistence with localStorage write-through cache. Timers scoped per character/server. Auto-detection from EntityDescriptionUpdated deferred (doesn't reliably provide total duration for offline timers).


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
  - Need t4o have some standard place in the panel layout for a button or something, perhaps. that explains how to use it. maybe top right of the panel view? we will need a way to turn it off/off (not all panel layout consumers will want it) and then how to define what it contains.
  - **Effort: Large (breadth) | Impact: Medium (onboarding/discoverability)**

- [ ] Reevaluate test suite
  - Think about what tests make sense, what isn't giving value, and how to harden against future failures.
  - need to figure out if we can build some sample data or something mthat makes sense for tests.
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
  - kaeus is working on this i believe.
  - **Effort: Large (mostly frontend) | Impact: High**

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

- [ ] Gardening helper — see `docs/plans/gardening-helper.md`
  - **Moved to Blocked section.** Some garden features (fertilizer types, watering mechanics, growth timers) need additional in-game captures to document properly before implementation.
  - **Effort: Medium-Large | Impact: High**

- [ ] Macros or process interaction
  - Can we target the game process and send commands? Can we screen-read the process? Major research question with significant technical and policy implications.
  - We know from existing discussions that things like mouse/keyboard macros are okay. 
  - **Effort: Large (research) | Impact: Unknown (depends on feasibility)**

- [ ] Track total owned quantity changes over time
  - Start with currencies, expand to other items. Currencies may be in character JSON, inventory JSON, or both. PG doesn't overwrite inventory reports in the reports folder — could backfill historical data from old reports. Character JSONs are overwritten on export though. **Infrastructure gap:** database supports point-in-time snapshots only (`character_item_snapshots`). `sales_history` tracks vendor transactions but no time-series quantity tracking exists. Needs: new tables for quantity history, periodic delta recording (event-driven or snapshot-based), UI for time-range queries and charts.
  - **Effort: Large (new infrastructure) | Impact: High (trend data is very valuable)**

- [ ] Auto cleanup of old exported/saved PG data
  - Old chatlogs, reports, etc. Let the user set their retention policies. Needs UI for policy config and safe file deletion logic.
  - **Effort: Large | Impact: Low-Medium (housekeeping feature)**
