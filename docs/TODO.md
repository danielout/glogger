# TODO

Small tasks and notes that don't belong in a dedicated plan.

*Last reviewed: 2026-04-07*

---

## To Sort

- IMPROVEMENT: Found these lines in the player.log, probably something we can work in to our parsing to improve things:
    ```
    [16:00:51] New Network State: Picking Character... (GotCharacters -> PickingCharacter)
    [16:00:51] Vivox - LoginAsync(Zenith)
    [16:00:51] Logged in as character Zenith. Time UTC=03/06/2026 16:00:51. Timezone Offset -08:00:00
    EVENT(Ok): loginCharacter, numChars=2
    [16:00:51] New Network State: Joined Area... Initializing Scene (PickingCharacter -> JoinedArea)
    [16:00:51] LOADING LEVEL AreaCasino
    Unloading 3 Unused Serialized files (Serialized files now loaded: 8)
    [16:00:51] Logging chat to C:/Users/danie/AppData/LocalLow/Elder Game/Project Gorgon/ChatLogs/Chat-26-03-06.log
    UnloadTime: 4.145100 ms
    ```

- BUG: Cook's Helper needs to make sure it accounts for ice conjuration.

---

## Quick Wins (Small Effort, Noticeable Value)

- [x] BUG: Market Prices screen doesn't scroll properly
  - The main values table in `MarketView.vue` has no explicit overflow handling inside its PaneLayout parent. Likely just needs an `overflow-y-auto` on the table container.
  - **Effort: Small | Impact: Medium (unusable at scale)**

- [x] BUG: Crafting project delete fires at the same time the confirmation dialog appears
  - The delete action triggers immediately rather than waiting for user confirmation. Need to gate the delete behind the dialog result.
  - **Effort: Small | Impact: Medium (destructive action without confirmation)**

- [ ] Save page state of projects page when navigating off it
  - Active project and group selection are ephemeral reactive refs in `craftingStore`. Pane widths already persist via `useViewPrefs()`. Just need to persist `activeProject`/`activeGroupName` the same way.
  - **Effort: Small | Impact: Medium (annoyance)**

- [x] Better formatting for older chat lines that include date
  - Fixed: timestamp column had fixed `w-15` width too narrow for date+time format; switched to `whitespace-nowrap` with auto width.

- [x] Character->Skills: PaneLayout, rewards, bonus sources, alignment, and layout improvements
  - Converted to PaneLayout with resizable/collapsible left pane. Rewards now show attained (with checkmarks) and upcoming. Bonus level sources section shows which skills grant bonuses with achieved status. Fixed level alignment. XP/Session stats use responsive grid layout.

- [ ] Primary/Secondary naming on gear is confusing in the build planner
  - Currently uses generic "Primary"/"Secondary" labels with blue/emerald color coding. Could replace with actual skill names (e.g., "Sword / Psychology") throughout the UI — the skill names are already available on the preset.
  - **Effort: Small | Impact: Medium (reduces confusion)**

---

## Medium Effort, High Value

- [ ] Update data browser to have better, more readable layouts
  - Design/UX task. All tabs already use `PaneLayout` with consistent two-pane search/detail pattern. Improvements are incremental polish: better section headers, consistent key-value grids, spacing per `docs/architecture/ux-standards.md`. Could break into sub-tasks per browser tab.
  - **Effort: Medium (iterative) | Impact: Medium (polish)**


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

- [ ] Book viewer in the data browser
  - We have the book data already. Would make books easier to read than the in-game UI. Needs a new detail view/tab in the data browser.
  - **Effort: Medium | Impact: Medium (nice quality-of-life feature)**


- [ ] Hot tips tracker
  - Track hot tips in the game. Needs investigation into what data is available.
  - **Effort: Medium | Impact: Medium**

- [ ] Investigate seedling/plant/milling product linkage in CDN data
  - Right now in the data browser, items show up as "gift from grindstone" or "gift from ripesunflowerplant" with no direct linkage. Need to check if CDN data has a way to link seedling → plant → milling product.
  - **Effort: Medium | Impact: Low-Medium (data browser completeness)**

- [x] Pinned tooltips in a bottom tray
  - Implemented as the Reference Shelf — see `docs/plans/quick-reference-system.md`. Pin entities from tooltips, hover shelf chips to peek, click to navigate.

---

## Larger Efforts / Research Needed

- [x] UX for checking recipes/data without losing context
  - Addressed by Reference Shelf — hover pinned chips to see full tooltips without leaving current screen.

- [x] Quick reference favorites / bookmarking system
  - Implemented as the Reference Shelf pin system. Pins persist across screens and sessions.

- [ ] Shop/stall tracking — track what you put in, what sells, trends (would require manual entry or future log support) (reported by Reyetta)
  - Big feature. Needs: investigation of what log events exist for stalls, schema for tracking stock/sales, analytics UI. Manual entry fallback would work but adoption is questionable (Reyetta herself said manual entry is "too much effort"). The player event parser handles ~24 of ~60 known event types — player vendor events are in the "low priority / future" bucket per the parser docs.
  - **Effort: Large | Impact: High (if automated), Low (if manual-only)**

- [ ] Gear transmutation reference tool (reported by Reyetta)
  - No transmutation data in CDN. Would need manual data entry for transmutation rules, costs, and outcomes. Code is straightforward once data exists — the data is the bottleneck.
  - **Effort: Large (data acquisition) | Impact: Low-Medium (niche use case)**

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
