# TODO

Small tasks and notes that don't belong in a dedicated plan.

*Last reviewed: 2026-04-19*

---

## Investigations (Completed Research, No Code Changes Needed)

These are investigated items kept for reference — the research is done but the underlying limitation or blocker remains.

- **Item mods/augments not shown in inventory** — Player.log does NOT include TSys mod/augment data. Only `ProcessAddItem` (name + instance_id) and `ProcessUpdateItemCode` (stack size + type ID) are available. TSys data is only in VIP Inventory JSON export (snapshot imports already store it). Fundamental log format limitation.
- **Equipment display limited** — Equipment IS tracked from Player.log via `ProcessSetEquippedItems`, but only provides `slot` + `appearance_key` — no item names, stats, or details. A basic display could be built showing appearance slots only. Full details require VIP JSON export.

---

## Quick Wins (Small Effort, Noticeable Value)

---

## Medium Effort, High Value

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
  - Saves almanac data when you check it in-game. The almanac shows daily bonus (item + zone for guaranteed double-yield), rotating at midnight EST. `ProcessBook` parser event already exists and emits `BookOpened { title, content, book_type }`, but only `PlayerShopLog` is handled in the coordinator. **Blocker:** need someone to read the almanac in-game and check what the `ProcessBook(...)` line looks like in Player.log. If parseable: add coordinator handler, new SQLite table, frontend widget using `ItemInline`/`AreaInline`.
  - **Effort: Medium**

- [ ] General-purpose timer system (mushroom barrels, brewing, cheesemaking, fletching)
  - All these skills share a real-time waiting pattern with no log events for the timer portion. Mushroom barrel timers, brewing cask aging (1–3h), cheesemaking aging (1–9h), and fletching drying (1–30m, daylight+sunny only) would all need manual-entry timers. Could share a single reusable timer system. Talk to buppis for brewing specifics. Worth checking if newer game versions produce aging log events.
  - **Effort: Medium-High**

- [ ] 'Package data' export feature
  - Create zip file of game state JSON, player.log, chat logs, and character/inventory exports. All data sources have path helpers already. `character_snapshots` table already stores `raw_json` for every imported snapshot (historical character JSON is preserved). Needs `zip` crate, a new Tauri command to gather files + DB snapshots, and a save dialog.
  - **Effort: Low-Medium**

---

## Larger Efforts / Research Needed

- [ ] Statehelm repeatable quest tracking
  - StatehelmView already has full gift tracking. CDN quest data has `ReuseTime_*` fields and renown rewards. **Blocker:** quest events (`ProcessCompleteQuest`, `ProcessAddQuest`, etc.) are **not yet implemented** in the parser. Needs: (1) quest event parsing in `PlayerEventParser`, (2) new `game_state_quest_completions` table, (3) coordinator handler, (4) extract `ReuseTime_*` from CDN, (5) filter to Statehelm quests, (6) frontend UI with cooldown timers.
  - Sub-task: track statehelm renown possible vs earned (depends on quest tracking above).
  - **Effort: High | Impact: Medium-High**

- [x] Debug capture devtool
  - 'Start/stop debug capture' that saves: gamestate at start+stop, all player.log lines during capture, Status and Combat chat channels, character/inventory JSONs if detected. Any glogger debug should be saved as well. Save as single file with optional notes field.
  - **Effort: High | Impact: Medium (debugging/support)**

- [ ] Shop/stall tracking (reported by Reyetta)
  - Track what you put in, what sells, trends. Player vendor events are in the parser's "low priority / future" bucket. Manual entry fallback has questionable adoption (Reyetta said it's "too much effort").
  - **Effort: Large | Impact: High (if automated), Low (if manual-only)**

- [ ] Nightmare cave challenge door tracker
  - Need to look up all the challenges and see which ones we can track. Some are easy (1200 armor) and some are harder (have 4x 10-second premonition buffs). Could also track letters of authority as alternate path. Requires research + parser work.
  - **Effort: Large (research + implementation) | Impact: Medium (niche but useful)**

- [ ] Gardening helper
  - Should be able to detect seeds, fertilizer, water in inventory. Could also track what nearby plants need. Needs investigation into what inventory/proximity data is available from logs.
  - **Effort: Large (research + implementation) | Impact: Medium-High (if feasible)**

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
