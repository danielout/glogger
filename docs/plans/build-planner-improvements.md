# Build Planner Improvement Plan

## Overview

Need to make a ton of changes to make the build planner better.

Update documentation when done. This will track our task list for now.

## Tasks

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
