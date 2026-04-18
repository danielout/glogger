# TODO

Small tasks and notes that don't belong in a dedicated plan.

*Last reviewed: 2026-04-17*

---

## To Sort


- investigate: we don't show the mods/augments/etc on items in the inventory. can we? should see if this data is anywhere in the log
  - **Investigated:** Player.log does NOT include TSys mod/augment data. Only `ProcessAddItem` (name + instance_id) and `ProcessUpdateItemCode` (stack size + type ID) are available. TSys data is only available through VIP Inventory JSON export (snapshot imports already store it). This is a fundamental log format limitation.
- investigate: we don't show current equipment anywhere - is this in the log?
  - **Investigated:** Equipment IS tracked from Player.log via `ProcessSetEquippedItems`, but it only provides `slot` + `appearance_key` — no item names, stats, or details. The data is stored in `game_state_equipment` and exposed to the frontend. A basic "current equipment" display could be built but would only show appearance slots, not full item info. Full equipment details would require the VIP JSON export.

- ~~feat: migrate the teleport machine codes off the wiki and in to a widget.~~ **Done.** `TeleportCodesWidget` — searchable lookup with zone filtering, ~190 codes across 14 zones.
- feat: "garden almanac" widget that saves almanac data when you check it in game.
  - **Investigated:** The almanac shows daily bonus (item + zone for guaranteed double-yield), rotating at midnight EST. `ProcessBook` parser event already exists and emits `BookOpened { title, content, book_type }`, but only `PlayerShopLog` is handled in the coordinator. The almanac likely fires as a different book type. **Blocker:** need someone to read the almanac in-game and check what the `ProcessBook(...)` line looks like in Player.log. If parseable: add coordinator handler, new SQLite table for entries, frontend widget using `ItemInline`/`AreaInline`. Widget can only show data after the player reads the almanac — can't predict future bonuses unless the rotation algorithm is reverse-engineered. **Effort: Medium.**
- ~~feat: "mushroom farming" widget — lookup tool with moon phase integration.~~ **Done.** `MushroomFarmingWidget` — 22 varieties with level, grow time, substrates, and live moon phase highlighting for robust/poor. Sortable and filterable.
  - feat: barrel timers for mushroom farming. No log events for barrel placement/harvest, so timers would need manual user input. Needs a general-purpose timer system (reusable with brewing/cheesemaking/fletching). **Effort: Medium-High.**
- feat: fletching/cheesemaking/brewing timer widget? what can we do there?
  - **Investigated:** All three skills share a real-time waiting pattern. **Brewing:** un-aged casks age in caves, 1–3 hours by level, 2–3 concurrent slots. **Cheesemaking:** cheese ages in cave containers, 1–9 hours, 1–3 slots. **Fletching:** arrow shafts dry in boxes, 1–30 min, but only during daylight + sunny weather, 1–7 slots. The parser recognizes `Brew`/`Distill` as `DoDelayLoop` action types but only for the short crafting animation — the multi-hour aging/drying timers are **not in Player.log**. No fletching or cheesemaking recognition in the parser either. Without log events, these are manual-entry timer widgets. Could share a general-purpose timer system with mushroom barrel timers. Worth checking if newer game versions produce aging log events. **Effort: Medium-High.** Talk to buppis for more info on brewing specifics.
- feat: add repeatable quest tracking for statehelm quests to the statehelm character page.
  - **Investigated:** StatehelmView already has full gift tracking (NPC cards, favor tiers, progress dots). CDN quest data has `ReuseTime_Days/Hours/Minutes` fields for repeatable quest cooldowns and `DeltaScriptAtomicInt` renown rewards. **Blocker:** quest events (`ProcessCompleteQuest`, `ProcessAddQuest`, etc.) are listed in the parser docs as **not yet implemented** — no `QuestCompleted` variant exists in `PlayerEvent`. Needs: (1) implement quest event parsing in `PlayerEventParser`, (2) new `game_state_quest_completions` table, (3) coordinator handler, (4) extract `ReuseTime_*` from CDN quest JSON, (5) filter to Statehelm quests, (6) frontend UI with cooldown timers. **Effort: High.**
  - impv: track count of statehelm renown possible vs earned in the statehelm widget. requires the above statehelm work first.
    - **Investigated:** Statehelm Renown is already tracked as a currency in `game_state_currencies`. "Possible vs earned" requires summing renown rewards from completed repeatable quests against total available — fully depends on the quest tracking above being built first.
- feat: 'package data' feature to create zip file of game state json, player.log, chat logs, and any character/inventory exports that occurred during the session. we probably should start saving the last X historical character jsons locally once we detect them to help with this, since (unlike inventory exports) they get overwritten
  - **Investigated:** All data sources have path helpers already (`get_player_log_path()`, `get_chat_logs_dir()`, Reports/ directory). `character_snapshots` table already stores `raw_json` for every imported snapshot — **historical character JSON is already preserved in the database**, so the "save last X" concern is partially solved. Inventory snapshots also store `raw_json`. No zip/export code exists — needs `zip` crate added to Cargo.toml, a new Tauri command to gather files + DB snapshots into a zip, and a save dialog. **Effort: Low-Medium.**

- ~~fix/impovement/idunno ChatLog search doesn't work at all~~ — **Done:** Added unified Search tab as default chat tab with `from:` and `in:` operator support
- ~~impv: need to support "from:user" in chat log search and "in:channel"~~ — **Done:** `parseSearchQuery()` utility parses `from:player` and `in:channel` operators
- ~~impv: need to be able to quickly jump to a specific chat search result to see it in context~~ — **Done:** Click a search result to view surrounding messages via `get_chat_messages_around` with highlighted target
- ~~fix: our dynamic loading of chat messages seems to get stuck?~~ — **Done:** Fixed race condition — added loading guard in `loadMore()`, moved offset increment to after response, added emit guard in scroll handler

---

## Quick Wins (Small Effort, Noticeable Value)


- [ ] Investigate and fix project tracking in Crafting -> Projects
  - The "Start Tracking" button was removed from `ProjectMaterialsPanel.vue` because it wasn't working well. The `startProjectTracking()` method still exists in `craftingStore`. Need to investigate what's broken and re-implement properly.
  - **Effort: Medium | Impact: Medium**

- [ ] Save page state of projects page when navigating off it
  - Active project and group selection are ephemeral reactive refs in `craftingStore`. Pane widths already persist via `useViewPrefs()`. Just need to persist `activeProject`/`activeGroupName` the same way.
  - **Effort: Small | Impact: Medium (annoyance)**


---

## Medium Effort, High Value


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

- [x] Book viewer in the data browser
  - Lorebooks tab added to the data browser. Browse by category, search by title/text/location, read formatted book content.

- [ ] Hot tips tracker
  - Track hot tips in the game. Needs investigation into what data is available.
  - **Effort: Medium | Impact: Medium**

- [ ] Investigate seedling/plant/milling product linkage in CDN data
  - Right now in the data browser, items show up as "gift from grindstone" or "gift from ripesunflowerplant" with no direct linkage. Need to check if CDN data has a way to link seedling → plant → milling product.
  - **Effort: Medium | Impact: Low-Medium (data browser completeness)**



---

## Larger Efforts / Research Needed


- [ ] Shop/stall tracking — track what you put in, what sells, trends (would require manual entry or future log support) (reported by Reyetta)
  - Big feature. Needs: investigation of what log events exist for stalls, schema for tracking stock/sales, analytics UI. Manual entry fallback would work but adoption is questionable (Reyetta herself said manual entry is "too much effort"). The player event parser handles ~24 of ~60 known event types — player vendor events are in the "low priority / future" bucket per the parser docs.
  - **Effort: Large | Impact: High (if automated), Low (if manual-only)**

- [ ] Nightmare cave challenge door tracker
  - Need to look up all the challenges and see which ones we can track. Some are easy (1200 armor) and some are harder (have 4x 10-second premonition buffs). Could also track how many letters of authority the player has as an alternate path. Requires research + parser work.
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
