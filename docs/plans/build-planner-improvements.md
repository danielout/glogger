# Build Planner Improvement Plan

## Overview

Comprehensive improvements to make the build planner best-in-class for Project Gorgon build planning. Our primary competitor is the web-based [GorgonExplorer Build Planner](https://www.gorgonexplorer.com/build-planner). We already have key advantages (ability bar planning, augment/CP budgets, per-slot rarity/level overrides) — these improvements aim to polish the UX and fill gaps.

Update documentation when done.

## Implementation Phases

Phases are ordered by dependency. Phases 2 and 4 can proceed in parallel after Phase 1. Phase 8 is independent after Phase 1.

```
Phase 1 (Layout Foundation)
  |
  +---> Phase 2 (Grid + Icons + Completeness)
  |       |
  |       +---> Phase 3 (Ability Bar Prominence)
  |
  +---> Phase 4 (Styled Dropdowns + Visual Hierarchy)
  |       |
  |       +---> Phase 5 (Effect Formatting + CP Bars)
  |               |
  |               +---> Phase 6 (Cross-Referencing)
  |               |       |
  |               |       +---> Phase 7 (Summary Overhaul)
  |               |
  |               +---> Phase 9 (Mod Column Layout)
  |
  +---> Phase 8 (Gear Search -- independent, backend work)

Phase 10 (Investigation -- anytime)
```

---

## Phase 1: Layout Foundation (PaneLayout Migration + Summary Pane) -- DONE

- [x] Migrate to PaneLayout
  - `BuildPlannerScreen.vue` now uses `PaneLayout` with left pane (Equipment), center (mod picker/ability editor), and right pane (Build Summary). Removed custom flex layout and `showSummary` toggle button.

- [x] Make Build Summary a persistent pane instead of slide-out overlay
  - `BuildSummary.vue` is now a persistent right pane (collapsible via PaneLayout, defaultCollapsed: true). Removed Teleport/overlay/backdrop/slide-transition CSS. Effects load eagerly on mount and when the active preset changes.

**Files changed:** `BuildPlannerScreen.vue`, `BuildSummary.vue`

---

## Phase 2: Equipment Grid Visual Upgrade + Completeness Indicator -- DONE

Redesigned the left pane content for scannability.

- [x] Visual equipment grid / paper doll layout
  - `SlotGrid.vue` now organizes slots by equipment group (Armor, Weapons, Jewelry, Other) with labeled sections. Per-slot rendering extracted into `SlotCard.vue` component.

- [x] Add item icons to slot grid
  - Each `SlotCard` shows the item's `GameIcon` (from `resolvedSlotItems`) next to the slot name. Empty slots show a placeholder.

- [x] Build completeness indicator in header
  - `BuildCompleteness.vue` shows "X/10 slots, X/Y mods, X/3 bars" in the header area with green highlights when sections are complete.

**Files changed:** `SlotGrid.vue`, `BuildPlannerScreen.vue`
**Files created:** `SlotCard.vue`, `BuildCompleteness.vue`

---

## Phase 3: Elevate Ability Bars to Equal Prominence -- DONE

Made ability bars peer-level with equipment in the left pane.

- [x] Elevate ability bars to equal prominence with equipment
  - Left pane now has **Equipment / Abilities tabs** so each section gets the full pane height. `AbilityBarSummary.vue` redesigned from accordion rows to card-style bars with skill name, fill count, and always-visible ability icon strips. Tabs auto-switch when user clicks a slot (Equipment) or ability bar (Abilities).

**Files changed:** `BuildPlannerScreen.vue`, `AbilityBarSummary.vue`

---

## Phase 4: Styled Dropdowns + Visual Hierarchy + Tier Selection -- DONE

Biggest single visual consistency win. Replaced all native `<select>` elements and reduced color overload.

- [x] Replace native `<select>` elements with styled dropdowns
  - Created `StyledSelect.vue` shared component (props: options, modelValue, placeholder, size, colorClass, fullWidth). Teleport-based dropdown with keyboard nav, auto-positioning, and dark theme styling. Replaced all native selects across BuildHeader, SlotCard, SlotModPicker, and ModColumn.

- [x] Improve tier selection UX
  - Created `TierSelector.vue` segmented control showing level ranges as clickable segments with gold highlight for active tier. Replaced tier selects in ModOption and ModAssignment. Removed `.tier-select` scoped CSS hacks.

- [x] Reduce color overload / improve visual hierarchy
  - Replaced green/yellow background tinting on slot and bar cards with subtle left-border accent (green-500/50 for full, yellow-500/40 for partial). Selected state uses softer gold tint. Reduces the number of competing background colors while preserving status at a glance.

**Files created:** `src/components/Shared/StyledSelect.vue`, `TierSelector.vue`
**Files changed:** `BuildHeader.vue`, `SlotCard.vue`, `SlotModPicker.vue`, `ModColumn.vue`, `ModOption.vue`, `ModAssignment.vue`, `AbilityBarSummary.vue`

---

## Phase 5: Mod Effect Formatting + CP Progress Bars -- DONE

Color-coded effects and visual CP budgets.

- [x] Color-code and visually format mod effect values
  - Created `EffectLine.vue` with two modes: **raw string mode** (parses +/- numbers from formatted text and colorizes green/red) and **structured mode** (accepts label, formattedValue, numericValue, iconId for pre-parsed data). Replaced raw `{{ effect }}` patterns in ModOption, ModAssignment, and all three BuildSummary views (By Skill, Effect Totals, By Ability), plus Item Attributes.

- [x] CP budget as visual progress bars
  - Created `CpProgressBar.vue` with animated progress bar, color states (amber for partial, green for full, red for over-budget), and xs/sm size variants. Replaced text CP display in SlotCard and BuildSummary.

**Files created:** `EffectLine.vue`, `CpProgressBar.vue`
**Files changed:** `ModOption.vue`, `ModAssignment.vue`, `BuildSummary.vue`, `SlotCard.vue`

---

## Phase 6: Cross-Referencing + Filtering Intelligence -- DONE

Added mod-to-ability cross-references and global search.

- [x] Ability-to-mod cross-referencing during editing
  - Created `useBuildCrossRef.ts` composable that maps powers to assigned abilities via effect text matching. `AbilityOption` shows "X mods boost this" indicators. `ModOption` shows "boosts: Thunderstrike, Pound To Slag" labels when a power's effects reference assigned abilities.

- [x] Ability to filter out mods for unused abilities
  - Added "My abilities" checkbox to `SlotModPicker.vue` filter bar. When active, sorts ability-related mods to the top of each column. Applied to both skill columns and augment list.

- [x] Global mod search across all slots
  - Created `GlobalModSearch.vue` with text search across all assigned mods in the build, results grouped by slot with match counts. Shows in the center area when no slot or ability bar is selected (replaces the empty state placeholder when mods exist).

**Files created:** `src/composables/useBuildCrossRef.ts`, `GlobalModSearch.vue`
**Files changed:** `AbilityBarEditor.vue`, `AbilityOption.vue`, `ModOption.vue`, `SlotModPicker.vue`, `BuildPlannerScreen.vue`

---

## Phase 7: Build Summary Formatting Overhaul -- DONE

Major presentation pass on the now-persistent right pane.

- [x] Massive formatting improvements to Build Summary content
  - Replaced flat grid Slot Breakdown with `SummarySlotCard.vue` cards showing item icon, mod count, CP progress bar, and mod list per slot. Only slots with mods are shown. Totals moved to section header.

- [x] Better correlation of mod to ability
  - Created `AbilityDamageCard.vue` for the "By Ability" tab. Each card shows ability name, effect count, and all mod effects referencing that ability with EffectLine formatting and source labels.

**Files created:** `SummarySlotCard.vue`, `AbilityDamageCard.vue`
**Files changed:** `BuildSummary.vue`

---

## Phase 8: Improved Gear Search

Needs backend work for new filter dimensions.

- [ ] Better, more detailed gear searching in both data browser and build planner
  - `SlotItemPicker.vue` has text search, skill filter, and level range. Add armor type dropdown, weapon type dropdown, and attribute/effect text search. Needs backend support for the new filter dimensions.

**Files to modify:** `SlotItemPicker.vue`
**Backend changes:** Extend item search in `src-tauri/src/cdn_commands.rs` to support filtering by armor type, weapon type, and effect text.

---

## Phase 9: Mod Column Layout Improvements -- DONE

Refined center area mod picker for better space utilization.

- [x] Improve mod column layout for long content
  - Added "Compact" toggle to `SlotModPicker.vue` filter bar. When active, `ModOption.vue` hides effect details, prereq, ability labels, and disabled reasons -- showing only the mod name, skill badge, tier selector, and add button. Dramatically reduces per-mod height, allowing many more mods to be visible at once. Compact prop flows through `ModColumn.vue` to `ModOption.vue`.

**Files changed:** `SlotModPicker.vue`, `ModColumn.vue`, `ModOption.vue`

---

## Phase 10: Investigation & Deferred Tasks

Require game knowledge investigation before implementation can be designed. Can start anytime.

- [ ] Belts need a systematic fix
  - Belts are in `EQUIPMENT_SLOTS` as a standard slot in the `'extra'` group but treated identically to armor/weapons. Belts likely have unique constraints (different mod pools, no rarity tiers?) that aren't modeled. Needs game-knowledge investigation.

- [ ] Better support for recipes that consume crafting points
  - System currently handles augments (100 CP each) and has per-slot CP budgets (100-160 depending on item origin). Regular recipes that consume CP from the budget aren't well represented in the mod picker. Needs investigation into what the game actually supports beyond augments.
