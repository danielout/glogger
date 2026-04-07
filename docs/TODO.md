# TODO

Small tasks and notes that don't belong in a dedicated plan.

*Last reviewed: 2026-04-07*

---

## To Sort

- IMPROVEMENT: better UX for adding market prices

---

## Quick Wins (Small Effort, Noticeable Value)

- [x] BUG: Market Prices screen doesn't scroll properly
  - The main values table in `MarketView.vue` has no explicit overflow handling inside its PaneLayout parent. Likely just needs an `overflow-y-auto` on the table container.
  - **Effort: Small | Impact: Medium (unusable at scale)**

- [ ] Save page state of projects page when navigating off it
  - Active project and group selection are ephemeral reactive refs in `craftingStore`. Pane widths already persist via `useViewPrefs()`. Just need to persist `activeProject`/`activeGroupName` the same way.
  - **Effort: Small | Impact: Medium (annoyance)**

- [ ] Better formatting for older chat lines that include date
  - `ChatMessage.vue` already uses `useTimestamp` with smart formatting (time-only for today, date+time for older). The issue is likely that the date+time format for older lines needs visual distinction or better layout so the date doesn't crowd the message.
  - **Effort: Small | Impact: Low-Medium (polish)**

- [ ] Primary/Secondary naming on gear is confusing in the build planner
  - Currently uses generic "Primary"/"Secondary" labels with blue/emerald color coding. Could replace with actual skill names (e.g., "Sword / Psychology") throughout the UI — the skill names are already available on the preset.
  - **Effort: Small | Impact: Medium (reduces confusion)**

- [ ] Dashboard currency card layout improvements and a more useful header card
  - `ContextBar.vue` currently shows currencies in a simple horizontal row. Needs layout polish and a more informative header. Straightforward styling/layout work.
  - **Effort: Small | Impact: Low-Medium (polish)**

---

## Medium Effort, High Value

- [ ] Update data browser to have better, more readable layouts
  - Design/UX task. All tabs already use `PaneLayout` with consistent two-pane search/detail pattern. Improvements are incremental polish: better section headers, consistent key-value grids, spacing per `docs/architecture/ux-standards.md`. Could break into sub-tasks per browser tab.
  - **Effort: Medium (iterative) | Impact: Medium (polish)**

- [ ] Better, more detailed gear searching in both data browser and build planner
  - `ItemSearch.vue` has text search, keyword filter, slot filter, and level range. `SlotItemPicker.vue` has text search, skill filter, and level range. Neither supports armor type, weapon type, or searching within item attributes/effects. Would need new filter dropdowns and backend filtering logic.
  - **Effort: Medium | Impact: High (core workflow improvement)**

- [ ] Build planner layout needs to use PaneLayout
  - `BuildPlannerScreen.vue` uses custom flex layout (`w-80 shrink-0` + `flex-1`) instead of PaneLayout. Needs migration to match the project convention. Summary pane is a teleported slide-out overlay which further complicates layout.
  - **Effort: Medium | Impact: Medium (consistency + resizable panes)**

- [ ] Build planner summary pane needs massive formatting improvements
  - `BuildSummary.vue` is a slide-out overlay with many sections (skills, armor breakdown, CP overview, per-slot table, effects by skill/totals/ability). The information density is high and formatting is rough. May also want to reconsider whether a slide-out is the best pattern vs. a dedicated pane.
  - **Effort: Medium | Impact: Medium (usability of a core feature)**

- [ ] Better support in build planner for recipes that consume crafting points
  - System currently handles augments (100 CP each) and has per-slot CP budgets (100-160 depending on item origin). Regular recipes that consume CP from the budget aren't well represented in the mod picker. Needs investigation into what the game actually supports beyond augments.
  - **Effort: Medium | Impact: Medium (build accuracy)**

- [ ] Belts in the build planner need a systematic fix
  - Belts are in `EQUIPMENT_SLOTS` as a standard slot in the `'extra'` group but treated identically to armor/weapons. Belts likely have unique constraints (different mod pools, no rarity tiers?) that aren't modeled. Needs game-knowledge investigation.
  - **Effort: Medium | Impact: Medium (build accuracy)**

- [ ] Show critical resources on the dashboard
  - Diamonds, amethysts, aquamarines, eternal greens, salt, fire dust — inventory data is already tracked. Needs a new dashboard card that pulls specific item counts from inventory store and displays them prominently.
  - **Effort: Medium | Impact: High (at-a-glance value)**

- [ ] Show latest watchword detections on the dashboard
  - Watchword matches are already stored and viewable per-rule in `WatchwordsView.vue`. Needs a new dashboard card that aggregates recent matches across all rules into a live feed.
  - **Effort: Medium | Impact: Medium (awareness without navigating to chat)**

- [ ] Actually implement audio alerts for watchwords
  - The "Play sound" checkbox exists in the rule editor UI and the setting is stored in `WatchNotifyConfig`, but there's no audio playback implementation behind it. Toast notifications appear to be wired up. Need to add actual audio file(s) and playback logic.
  - **Effort: Medium | Impact: Medium (key alerting feature)**

- [ ] Market Prices screen needs better layout
  - Currently a simple card-based vertical layout inside EconomicsView's PaneLayout. Table columns are fixed (Item | Price | Notes | Updated | Actions). Could benefit from a two-pane layout, better spacing, and visual hierarchy. Filtering already exists.
  - **Effort: Medium | Impact: Medium (usability)**

- [ ] Bulk price setting for market values
  - Currently single-item add/edit only. Import/export exists for JSON data migration but no in-app bulk operations. Could add multi-select with batch price update, percentage adjustments, or category-based pricing.
  - **Effort: Medium | Impact: Medium (power-user workflow)**

---

## Larger Efforts / Research Needed

- [ ] Show rez timer on the dashboard
  - No rez timer tracking exists anywhere. Would need to detect the death/resurrection event in PlayerEventParser, track the cooldown start time, and display a countdown on the dashboard. Requires parser work + new dashboard card.
  - **Effort: Large | Impact: Medium**

- [ ] Detect long-cooldown activations and display timers on dashboard
  - Resuscitate, opening portals, Hoplology, etc. — no cooldown tracking infrastructure exists in game state. Would need: identifying the relevant log events, adding them to the PlayerEventParser, a new cooldown tracking system, and dashboard timer cards. Cooldown durations may not be in log data and might need manual configuration.
  - **Effort: Large | Impact: Medium-High (very useful if feasible)**

- [ ] UX for checking recipes/data without losing context
  - Brainstorm: the data browser currently lives as a full screen. Ideas include: a floating/popover data browser, a quick-peek modal, breadcrumb-style navigation history, or split-view. This is a significant UX architecture decision that affects many workflows.
  - **Effort: Large (design + implementation) | Impact: High (core UX pain point)**

- [ ] Quick reference favorites / bookmarking system
  - No favorites system exists. Would need: data model for bookmarked entities, UI for managing favorites (panel, popup, toolbar), integration with entity navigation. Related to the data browser UX question above — could be part of a unified solution.
  - **Effort: Large | Impact: High (solves the "multiple wiki tabs" problem)**

- [ ] Shop/stall tracking — track what you put in, what sells, trends (would require manual entry or future log support) (reported by Reyetta)
  - Big feature. Needs: investigation of what log events exist for stalls, schema for tracking stock/sales, analytics UI. Manual entry fallback would work but adoption is questionable (Reyetta herself said manual entry is "too much effort"). The player event parser handles ~24 of ~60 known event types — player vendor events are in the "low priority / future" bucket per the parser docs.
  - **Effort: Large | Impact: High (if automated), Low (if manual-only)**

---


## Medium Effort, High Value

- [ ] Update data browser to have better, more readable layouts
  - Design/UX task. All tabs already use `PaneLayout` with consistent two-pane search/detail pattern. Improvements are incremental polish: better section headers, consistent key-value grids, spacing per `docs/architecture/ux-standards.md`. Could break into sub-tasks per browser tab.
  - **Effort: Medium (iterative) | Impact: Medium (polish)**

---

## Larger Efforts / Research Needed

- [ ] Shop/stall tracking — track what you put in, what sells, trends (would require manual entry or future log support) (reported by Reyetta)
  - Big feature. Needs: investigation of what log events exist for stalls, schema for tracking stock/sales, analytics UI. Manual entry fallback would work but adoption is questionable (Reyetta herself said manual entry is "too much effort"). The player event parser handles ~24 of ~60 known event types — player vendor events are in the "low priority / future" bucket per the parser docs.
  - **Effort: Large | Impact: High (if automated), Low (if manual-only)**

- [ ] Gear transmutation reference tool (reported by Reyetta)
  - No transmutation data in CDN. Would need manual data entry for transmutation rules, costs, and outcomes. Code is straightforward once data exists — the data is the bottleneck.
  - **Effort: Large (data acquisition) | Impact: Low-Medium (niche use case)**
