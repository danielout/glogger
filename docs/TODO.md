# TODO

Small tasks and notes that don't belong in a dedicated plan.

*Last reviewed: 2026-04-21*

---


## TO SORT

*(empty — all sorted below)*

---

## Investigations (Completed Research, No Code Changes Needed)

These are investigated items kept for reference — the research is done but the underlying limitation or blocker remains.

- **Item mods/augments not shown in inventory** — Player.log does NOT include TSys mod/augment data. Only `ProcessAddItem` (name + instance_id) and `ProcessUpdateItemCode` (stack size + type ID) are available. TSys data is only in VIP Inventory JSON export (snapshot imports already store it). Fundamental log format limitation.
- **Equipment display limited** — Equipment IS tracked from Player.log via `ProcessSetEquippedItems`, but only provides `slot` + `appearance_key` — no item names, stats, or details. A basic display could be built showing appearance slots only. Full details require VIP JSON export.

---

## Quick Wins (Small Effort, Noticeable Value)

- [x] Parse ProcessSetCelestialInfo for moon phase
  - Parsed and persisted to `game_state_moon`. Also added parsers for ProcessGuildGeneralInfo, ProcessCompleteDirectedGoals, and ProcessSetString (9 known keys) — all persisted to new game state tables in migration v33.

- [x] Auto-import gourmand report from ProcessBook
  - When the Foods Consumed SkillReport book is opened in-game, the coordinator auto-imports it to `gourmand_eaten_foods` using the existing parser. No file save needed.

- [x] Parse PlayerAge and Behavior Report stats from ProcessBook
  - Structured stats (kills, deaths, damage, time played, badges, challenge restrictions, food stats, etc.) extracted from HelpScreen and PlayerAge book content, persisted to `character_report_stats`. Displayed on Character > Stats tab via `ReportStatsSection.vue`.

- [x] Milking timers dashboard widget
  - NPC cow milking cooldown tracker. Detects milks from ProcessStartInteraction + chat "Bottle of Milk" gain. Backfills from cooldown error messages. Small dashboard widget with cows grouped by zone, 1h countdown timers, current zone floated to top.

- [ ] Bug: instant-snack foods missing from gourmand report
  - They used to show up and now they're gone. Likely a regression in filtering or category logic.
  - **Effort: Low | Impact: Medium (data correctness)**

- [ ] Bug: food tooltip parsing broken on gourmand tracker
  - Tooltip parsing regressed at some point. Needs investigation into what changed. This is for the combined selected meal and snack buffs in the right panel. 
  - **Effort: Low | Impact: Medium (gourmand feature usability)**

- [ ] Bug: rez counter not working
  - Should be counting but isn't. Low priority. Maybe need another capture or two? unsure what's wrong here.
  - **Effort: Low | Impact: Low**

- [ ] Bug: crafting levelling planner keeps first-time-craft XP after removal
  - If you add a first-time craft and then remove it, the planner still thinks the first-time XP was used up. Logic bug in state cleanup.
  - **Effort: Low | Impact: Medium (planner accuracy)**

- [ ] Display last character.json and inventory.json import timestamps on dashboard... can show as tooltip when hovering the character name in the status widget.
  - Quick info display so users know how fresh their imported data is.
  - **Effort: Low | Impact: Medium (user awareness)**

- [ ] Bug: missing NPCs in NPC searches
  - Some NPCs don't appear in search results. Needs investigation — could be CDN data gap or resolver issue. Looks like maybe it is NPCs without an area? unsure.
  - **Effort: Low (investigation) | Impact: Medium**

- [ ] Show learned/unlearned status in recipe and skillbook tooltips
  - Recipe tooltips and recipe/skillbook item tooltips should indicate whether the current character has learned them.
  - **Effort: Low | Impact: Medium (discoverability)**

- [ ] Clean up documents folder structure
  - Better organization, establish clearer structure for docs.
  - **Effort: Low | Impact: Low (maintainability)**

- [ ] Parse ProcessUpdateDescription for entity state changes
  - Fires for nearby entities changing state (garden plants, crafting items with timers, etc.). Format: `(entityId, "name", "description", "action", actionType, "appearance", flags)`. This is the foundation event for gardening tracker and crafting timers — parsing it in the event parser is the first step.
  - **Effort: Low (parser only) | Impact: High (unblocks gardening + crafting features)**

---

## Medium Effort, High Value

- [ ] Manual food eaten/not eaten marking for gourmand
  - Users should be able to manually mark foods if they can't get their skill report. Fallback for when auto-import isn't available.
  - **Effort: Medium | Impact: Medium (accessibility)**

- [ ] Bug: incorrect survey session start/end times
  - Sometimes sessions get wrong timestamps. Needs investigation into what causes the mismatch.
  - **Effort: Medium (investigation) | Impact: Medium (data accuracy)**

- [ ] Bug: survey analytics "fastest method" times wildly off
  - Calculated times show hundreds or thousands of hours. Should be average time per completion × number needed. Something is fundamentally wrong with the calculation.
  - **Effort: Medium (investigation) | Impact: High (analytics credibility)**

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
  - Build reusable components for common patterns like tables to improve consistency.
  - **Effort: Medium (iterative) | Impact: Medium (consistency)**

- [ ] Smarter gamestate saving and initializing
  - Initial log parse needs to be more intelligent about stack counts. Better state reconstruction.
  - **Effort: Medium | Impact: Medium (data accuracy)**

- [ ] Better screen persistence across the app
  - Remember more navigation state so users don't lose their place.
  - **Effort: Medium | Impact: Medium (UX)**

- [ ] Boss kill loot timers
  - Track loot timers from boss kills.
  - **Effort: Medium | Impact: Medium**

- [ ] Enemy database in data browser
  - Add enemies as a browsable entity type in the data browser.
  - **Effort: Medium | Impact: Medium (completeness)**

- [ ] Area tooltips with useful information
  - Add informative tooltips when hovering area references.
  - **Effort: Medium | Impact: Medium (discoverability)**

- [ ] Configurable critical resources widget
  - Let users configure which resources appear in the critical resources dashboard widget.
  - **Effort: Medium | Impact: Medium (personalization)**

- [ ] Evaluate ingestion pipeline
  - Review the current log ingestion pipeline for correctness and efficiency.
  - **Effort: Medium | Impact: Medium (reliability)**

- [ ] Standardize skeletons and loading states
  - Create reusable skeleton/loading components for consistent loading UX across screens.
  - **Effort: Medium | Impact: Medium (polish/consistency)**

- [ ] Document standards around persistence, data access, naming
  - Write up conventions so development stays consistent.
  - **Effort: Medium | Impact: Medium (maintainability)**

- [ ] Bug: occasional inventory item miscounts
  - Rare cases where items coming into inventory aren't seen or are miscounted. Intermittent and hard to reproduce.
  - **Effort: Medium (investigation) | Impact: Medium (data accuracy)**

- [ ] Bug: cook's helper not updating after buying new recipes
  - User-reported. After purchasing new recipes, cook's helper doesn't reflect them. Hard to reproduce but worth investigating.
  - **Effort: Medium (investigation) | Impact: Medium**

- [ ] Investigate detecting recipe learning without character.json import
  - Currently recipes are only detected via character.json import. Are there log events when a player learns a recipe?
  - **Effort: Medium (investigation) | Impact: Medium (reduces manual import dependency)**

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

- [ ] Investigate PGQuartermaster for feature ideas
  - Users have reported it has features we need. Need to review: https://shamorshinf.github.io/PGQuartermaster/
  - **Effort: Medium (research) | Impact: Unknown**

- [ ] Dashboard widget sizing pass — consistent heights
  - Widgets have inconsistent `max-h-*` values (some 80, some 52, some 40, many have none). No systematic height management exists. Need a standardized sizing approach in `DashboardCard.vue` or the widget registry.
  - **Effort: Medium | Impact: Medium (visual consistency)**

- [ ] Dashboard widget sizing pass — consistent widths across three sizes
  - The sizing system exists in `dashboardWidgets.ts` (`small`/`medium`/`large` → col-span classes), but the responsive `auto-fill` grid means actual widths vary with viewport. `col-span-4` assumes enough columns exist. Needs either a fixed column count or size-aware breakpoints.
  - **Effort: Medium | Impact: Medium (visual consistency)**

- [ ] Investigate dashboard refresh issues
  - Reported as "weird page refresh issues." No obvious refresh/reload logic in `DashboardView.vue`. Likely caused by reactive store updates triggering unexpected re-renders, or PaneLayout interactions. Needs runtime debugging to identify the trigger.
  - **Effort: Unknown (investigation) | Impact: Unknown**

- [ ] Switch report detection from folder polling to chat log events
  - Currently the Reports folder is polled on a timer (`characterStore.ts` `startReportWatching()`, configurable 5–300s interval). The chat log already announces when exports happen. Switching to chat-log-triggered detection would be more responsive and eliminate unnecessary polling. Needs a new handler in the chat status parser for export messages.
  - **Effort: Medium | Impact: Small-Medium (efficiency/responsiveness)**

- [ ] Actually implement audio alerts for watchwords
  - The "Play sound" checkbox exists in the rule editor UI and the setting is stored in `WatchNotifyConfig`, but there's no audio playback implementation behind it. Toast notifications appear to be wired up. Need to add actual audio file(s) and playback logic.
  - **Effort: Medium | Impact: Medium (key alerting feature)**

- [ ] Market Prices screen needs better layout
  - Currently a simple card-based vertical layout inside EconomicsView's PaneLayout. Table columns are fixed (Item | Price | Notes | Updated | Actions). Could benefit from a two-pane layout, better spacing, and visual hierarchy. Filtering already exists.
  - **Effort: Medium | Impact: Medium (usability)**

- [ ] Bulk price setting for market values
  - Currently single-item add/edit only. Import/export exists for JSON data migration but no in-app bulk operations. Could add multi-select with batch price update, percentage adjustments, or category-based pricing.
  - **Effort: Medium | Impact: Medium (power-user workflow)**

- [ ] Better UX for adding market prices
  - Current add flow could be streamlined. Related to market prices layout and bulk price setting items above.
  - **Effort: Medium | Impact: Medium (usability)**

- [ ] Continue UI/UX standardization across screens
  - Some screens still don't look like they fit within the app, or have their own paradigms. Sidebars that don't use standardized panels, inconsistent patterns, etc.
  - **Effort: Medium (iterative) | Impact: Medium (consistency/polish)**

- [ ] Hot tips tracker
  - Track hot tips in the game. Needs investigation into what data is available.
  - **Effort: Medium | Impact: Medium**

- [ ] Investigate seedling/plant/milling product linkage in CDN data
  - Right now in the data browser, items show up as "gift from grindstone" or "gift from ripesunflowerplant" with no direct linkage. Need to check if CDN data has a way to link seedling → plant → milling product.
  - **Effort: Medium | Impact: Low-Medium (data browser completeness)**

- [ ] Garden almanac widget
  - Saves almanac data when you check it in-game. The almanac shows daily bonus (item + zone for guaranteed double-yield), rotating at midnight EST. `ProcessBook` parser event already exists and emits `BookOpened { title, content, book_type }`, but only `PlayerShopLog` is handled in the coordinator. **Blocker resolved:** capture analysis confirms book_type is `"GardeningAlmanac"` with parseable HTML content including current events (crop + zone + time remaining) and upcoming events. See `docs/plans/capture-results.md` and sample data in `docs/samples/devtolsCaptures/gardening-almanac-01.json`. Next: add coordinator handler, new SQLite table, frontend widget using `ItemInline`/`AreaInline`.
  - **Effort: Medium**

- [ ] General-purpose timer system (mushroom barrels, brewing, cheesemaking, fletching)
  - All these skills share a real-time waiting pattern with no log events for the timer portion. Mushroom barrel timers, brewing cask aging (1–3h), cheesemaking aging (1–9h), and fletching drying (1–30m, daylight+sunny only) would all need manual-entry timers. Could share a single reusable timer system. Talk to buppis for brewing specifics. **Partial update:** `ProcessUpdateDescription` does fire for timed crafting items while the player is nearby (e.g. "Rising Simple Sourdough" with proofing countdown and increasing scale value). This provides live progress for items in proximity but won't help with offline/away timers. See `docs/plans/capture-results.md`.
  - **Effort: Medium-High**


---

## Larger Efforts / Research Needed

- [ ] Work order cooldown tracking
  - Work orders have a 30-day cooldown when completed. Can track via recording completions, but also via character.json — completed work orders are removed from the array when cooldown expires, giving "available"/"unavailable" state even for completions we didn't witness. Should also support alt character tracking.
  - **Effort: Large | Impact: High (multi-character workflow)**

- [ ] Repeatable quest cooldown tracking (general)
  - Same approach as work order tracking but for all repeatable quests with cooldowns. Ties into Statehelm quest tracking and work order tracking above.
  - **Effort: Large | Impact: High**

- [ ] Skillbook autowatchwords
  - Automatically watch chat for item links of skill books the player could learn but doesn't own or know. Two modes: (1) books for currently-trained skills, (2) "future skill" mode where players select skills and we watch for any skillbooks they don't already own/know, even if skill level is too low to use them yet.
  - **Effort: Large | Impact: High (proactive skill progression)**

- [ ] Kaeus tool integration
  - Make it easier for kaeus to integrate his tools with glogger. Repos: GorgonBetTracker, GorgonCraftingTools, KaeusGorgonTools (all at github.com/kaeus). Needs investigation into what integration points make sense.
  - **Effort: Large (collaboration/research) | Impact: Medium (community/ecosystem)**

- [ ] Standardize search across the app (scryfall-inspired)
  - Search is implemented differently in different places. Need a smart, unified search system. Take inspiration from Scryfall's search syntax for filtering and querying.
  - **Effort: Large | Impact: High (UX consistency, power-user feature)**

- [ ] Write 'how to use' docs for each screen
  - Even brief docs for each screen would help users understand features. Could be in-app help or docs folder.
  - **Effort: Large (breadth) | Impact: Medium (onboarding/discoverability)**

- [ ] Repeatable quests / work orders tracking
  - A lot more work needed with repeatable quests and work orders. Ties into Statehelm quest tracking below.
  - **Effort: Large | Impact: High**

- [ ] Reevaluate test suite
  - Think about what tests make sense, what isn't giving value, and how to harden against future failures.
  - **Effort: Large | Impact: Medium (reliability/confidence)**

- [ ] Analyze what should move from frontend to Rust
  - Some frontend logic may be better served on the Rust backend. Needs analysis to identify candidates.
  - **Effort: Large (research) | Impact: Medium (performance/architecture)**

- [ ] Statehelm repeatable quest tracking
  - StatehelmView already has full gift tracking. CDN quest data has `ReuseTime_*` fields and renown rewards. **Blocker:** quest events (`ProcessCompleteQuest`, `ProcessAddQuest`, etc.) are **not yet implemented** in the parser. Needs: (1) quest event parsing in `PlayerEventParser`, (2) new `game_state_quest_completions` table, (3) coordinator handler, (4) extract `ReuseTime_*` from CDN, (5) filter to Statehelm quests, (6) frontend UI with cooldown timers.
  - Sub-task: track statehelm renown possible vs earned (depends on quest tracking above).
  - **Effort: High | Impact: Medium-High**

- [x] Debug capture devtool
  - 'Start/stop debug capture' that saves: gamestate at start+stop, all player.log lines during capture, Status and Combat chat channels, character/inventory JSONs if detected. Any glogger debug should be saved as well. Save as single file with optional notes field.
  - **Effort: High | Impact: Medium (debugging/support)**

- [x] Debug capture improvements (from capture analysis)
  - [x] **Line truncation bug** — Fixed: PlayerLogWatcher now uses `read_to_end` with partial-line detection (only advances position to last complete newline), matching ChatLogWatcher's approach. Also fixed CRLF position tracking on Windows.
  - [x] **Normal vs Full capture modes** — Done: "Save (Normal)" filters engine noise, "Save (Full)" keeps everything. Filtering happens at save time so raw data is always fully captured.
  - [x] **Post-capture notes editing** — Done: Two-phase stop/save flow. Recording stops first (snapshot taken), then user reviews stats and edits notes before choosing save mode.
  - [x] **Empty lines** — Fixed by the same partial-line detection fix (lines < 4 chars also filtered in normal mode).
  - **Opaque .NET types** — Some events (ProcessSetStarredRecipes, ProcessInventoryFolderSettings) log .NET type names instead of data. Not fixable from our side.

- [ ] Shop/stall tracking (reported by Reyetta)
  - Track what you put in, what sells, trends. Player vendor events are in the parser's "low priority / future" bucket. Manual entry fallback has questionable adoption (Reyetta said it's "too much effort").
  - **Effort: Large | Impact: High (if automated), Low (if manual-only)**

- [ ] Nightmare cave challenge door tracker
  - Need to look up all the challenges and see which ones we can track. Some are easy (1200 armor) and some are harder (have 4x 10-second premonition buffs). Could also track letters of authority as alternate path. Requires research + parser work.
  - **Effort: Large (research + implementation) | Impact: Medium (niche but useful)**

- [ ] Gardening helper
  - Should be able to detect seeds, fertilizer, water in inventory. Could also track what nearby plants need. **Research resolved:** `ProcessUpdateDescription` fires for every nearby garden plot with full state (Thirsty/Hungry/Growing/Ripe + crop name + action needed). Combined with inventory data, this enables a real-time gardening dashboard. See `docs/plans/capture-results.md` for the full state machine and sample data.
  - **Effort: Medium-Large (implementation only, research done) | Impact: High**

- [ ] Macros or process interaction
  - Can we target the game process and send commands? Can we screen-read the process? Major research question with significant technical and policy implications.
  - **Effort: Large (research) | Impact: Unknown (depends on feasibility)**

- [ ] Match-3 winnings tracker
  - Track winnings from the Match-3 minigame. Needs investigation into whether relevant log events exist.
  - **Effort: Large (research) | Impact: Low-Medium (minigame niche)**

- [ ] Monsters and Mantids winnings tracker
  - Track winnings from Monsters and Mantids. Needs investigation into whether relevant log events exist.
  - **Effort: Large (research) | Impact: Low-Medium (minigame niche)**

- [ ] Track total owned quantity changes over time
  - Start with currencies, expand to other items. Currencies may be in character JSON, inventory JSON, or both. PG doesn't overwrite inventory reports in the reports folder — could backfill historical data from old reports. Character JSONs are overwritten on export though. Needs schema for time-series tracking.
  - **Effort: Large | Impact: High (trend data is very valuable)**

- [ ] Auto cleanup of old exported/saved PG data
  - Old chatlogs, reports, etc. Let the user set their retention policies. Needs UI for policy config and safe file deletion logic.
  - **Effort: Large | Impact: Low-Medium (housekeeping feature)**
