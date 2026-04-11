# Character Build Planner

## Overview

The build planner is a **"plan what you want"** tool for designing combat builds. In Project Gorgon, the real power of a build comes from treasure effects (mods) on gear -- picking the right mods for your skill pair across all equipment slots is the core optimization loop for endgame players.

The planner lets you pick two combat skills, set a target level/rarity, and then browse and assign treasure effects to each equipment slot. It also supports ability bar planning (primary, secondary, and sidebar), crafting point budget tracking across augments/shamanic infusion/crafting enhancements, and saving multiple named builds per character.

## Game Mechanics Reference

### Rarity and Mod Distribution

Items have a "main" skill (whichever ends up with more mods) and an "auxiliary" skill. Which skill is main vs auxiliary is determined dynamically by what the player puts on the item — the first skill to claim more slots becomes main.

| Rarity | Total Mods | Valid Configurations |
|--------|-----------|---------------------|
| Common | 0 | — |
| Uncommon | 3 | 1 main + 2 generic, OR 3 generic |
| Rare | 3 | 2 main + 1 generic, 1 main + 2 generic, OR 3 generic |
| Exceptional | 3 | 2 main + 1 aux, 2 main + 1 generic, 1 main + 1 aux + 1 generic, OR 3 generic |
| Epic | 4 | 2 main + 2 aux, 2 main + 1 aux + 1 generic, 2 main + 2 generic, OR 4 generic |
| Legendary | 5 | 3 main + 2 aux, 3 main + 1 aux + 1 generic, 3 main + 2 generic, OR 5 generic |

The planner uses a **constraint solver** (`computeSlotConstraints` in `buildPlanner.ts`) that tracks which configurations are still reachable as mods are added. Empty slots dynamically show what types of mods can fill them (e.g., "Ice Magic, Knife Fighting, or Generic" or "Ice Magic only").

- **Augmentation** adds one extra mod to any item (except belts). Costs 100 CP from the item's budget.
- **Mods are unique per item** -- no duplicates of the same power on one piece of gear.
- **Mastercrafted/Foretold** items follow legendary distribution with 160 CP.
- **Crafting points**: Crafted items have 120 CP, dropped items have 100 CP, mastercrafted/foretold legendaries have 160 CP.
- **Armor types**: Armor pieces have a material type (Cloth, Leather, Metal, Organic). Equipping 3+ pieces of the same type grants a set bonus.
- **Belts**: Can only be Common (0 mods) or Uncommon (1 generic-only mod). No CP budget, no augments, no skill-specific mods.

### Crafting Point Budget

Three systems consume CP from an item's budget:

1. **Augmentation** (100 CP) -- adds one TSys power from any skill. Max 1 per item. Not available on belts.
2. **Shamanic Infusion** (100 CP) -- adds fixed effects like +Armor, +Sprint Speed. Uses `AddItemTSysPower()` recipe effects. Max 1 per item.
3. **Crafting Enhancements** (5-20 CP) -- adds armor, pockets, or elemental damage via Tailoring/Blacksmithing recipes. Uses `CraftingEnhanceItem*()` recipe effects. Can stack multiple of the same recipe as long as CP remains. Filtered by slot body part.

### Equipment Slots

- **Head, Chest, Legs, Hands, Feet** -- armor slots (participate in armor set bonuses)
- **Main Hand** -- weapon (Sword, Staff, Hammer, etc.)
- **Off Hand** -- shield, orb, bow, claw, etc.
- **Ring, Necklace** -- jewelry
- **Belt** -- 1 generic mod max (Uncommon), no CP, no augments, mapped to CDN's "Waist" slot

### Ability System

- **Primary skill bar**: 6 ability slots
- **Secondary skill bar**: 6 ability slots
- **Sidebar**: Configurable 6-12 slots (default 6) for sidebar skills (First Aid, Armor Patching, Survival Instincts)

Abilities are assigned per-slot (discrete positions matching the in-game bar). Only one tier of each ability family can be on a bar at a time. Mods often boost specific abilities by name, creating a link between mod and ability selection.

### References

- https://wiki.projectgorgon.com/wiki/Items
- https://wiki.projectgorgon.com/wiki/Augmentation
- https://wiki.projectgorgon.com/wiki/Treasure_Effects
- https://wiki.projectgorgon.com/wiki/Shamanic_Infusion
- https://wiki.projectgorgon.com/wiki/Belt
- https://wiki.projectgorgon.com/wiki/Combat
- https://wiki.projectgorgon.com/wiki/Armor
- https://www.gorgonexplorer.com/

---

## Architecture

### Layout

Three-panel PaneLayout with resizable/collapsible panes:

- **Header**: Build completeness indicator
- **Left pane** ("Equipment"): Paper doll layout with build selector, default settings (rarity/level), two-column equipment grid, aggregated stats, and ability bar summaries with inline skill selectors
- **Center**: Slot detail panel (item card + mod slots on left, tabbed mod browser / CP recipes on right) or global mod search
- **Right pane** ("Build Summary", 550px default, up to 900px): Tabbed effect views (By Skill, Effect Totals, By Ability)
- **Popups**: Ability picker dialog (per-slot), item picker dialog (with preview panel)

### Files

| Layer | File | Purpose |
|-------|------|---------|
| Backend | `src-tauri/src/db/build_planner_commands.rs` | Persistence: presets, mods, slot items, abilities, CP recipes |
| Backend | `src-tauri/src/cdn_commands.rs` | CDN queries: combat skills, TSys powers, power info, CP recipes for slot, item search with effect resolution, batch TSys↔ability lookup |
| Backend | `src-tauri/src/game_data/mod.rs` | GameData struct with precomputed TSys↔Ability index, constraint solver data |
| Backend | `src-tauri/src/db/migrations.rs` | v5: build tables, v6: slot items, v7: abilities, v8-v9: per-slot props, v21: CP recipes |
| Store | `src/stores/buildPlannerStore.ts` | Pinia store: build state, CRUD, mod/ability/CP recipe management, slot-addressed ability bars |
| Types | `src/types/buildPlanner.ts` | Interfaces, constants, rarity configs (`RARITY_CONFIGS`), constraint solver (`computeSlotConstraints`), helper functions |
| Composable | `src/composables/useBuildCrossRef.ts` | Mod↔ability cross-referencing via precomputed backend index (single batch call) |
| Composable | `src/composables/useBuildStats.ts` | Shared item attribute aggregation, CP budget totals |

### TSys ↔ Ability Cross-Reference

A precomputed bidirectional index built once at CDN load time (`build_tsys_ability_index` in `game_data/mod.rs`). Uses three matching strategies:

1. **Attribute token matching** (most reliable): Extracts `{TOKEN}` keys from TSys effect descriptions, matches against ability combat stats (`attributes_that_delta_damage`, `attributes_that_mod_base_damage`, etc., including DoT sub-arrays)
2. **Icon ID matching**: Checks if an ability's `icon_id` appears in `<icon=NNNN>` tags in TSys text effect descriptions
3. **Text name matching** (fallback): Searches for ability display names in text effect descriptions, with prefix disambiguation to avoid "Pound" matching "Pound To Slag" (longer names get priority, only names 4+ chars)

Stored as two `HashMap`s in `GameData`:
- `tsys_to_abilities: HashMap<String, Vec<u32>>` — TSys key → all ability IDs (all tiers of matched families)
- `ability_to_tsys: HashMap<u32, Vec<String>>` — ability ID → TSys keys

Frontend access via `get_tsys_ability_map` Tauri command (batch lookup, O(1) per key). `useBuildCrossRef.ts` makes one batch call when slot powers change, caches the result, and provides `isAbilityRelated()` / `getAbilitiesForPower()` as pure in-memory lookups.

### Components

All in `src/components/Character/BuildPlanner/`:

| Component | Purpose |
|-----------|---------|
| `BuildPlannerScreen.vue` | Top-level: PaneLayout with left/center/right panels |
| `BuildCompleteness.vue` | "X/10 slots, X/Y mods, X/3 bars" progress in header |
| `PaperDollLayout.vue` | Left pane: build selector, defaults (rarity/level), paper doll grid, ability bars |
| `PaperDollSlot.vue` | Compact slot indicator in the paper doll grid |
| `PaperDollStats.vue` | Center stats column: aggregated item attributes, armor set bonuses |
| `SlotDetailPanel.vue` | Center pane: slot header, item card, skill overrides, two-column mod/CP layout |
| `ModSlotList.vue` | Left column: mod slots with constraint-based labels, augment slot, CP bar + assigned recipes (grouped with counters), "Slot Rules" tooltip |
| `ModSlotBar.vue` | Single mod slot: filled (with effects/tier/remove) or empty (shows allowed mod types) |
| `ModBrowser.vue` | Right column (Mods tab): skill filter (defaults to build skills + generic), ability filter, text search, compact mode. Click-to-add with constraint validation |
| `ModBrowserItem.vue` | Mod card: icon, skill badge, effects, equip slot chips, tier selector, "+ Mod" and "+ Aug" buttons, ability cross-reference labels |
| `CpRecipePanel.vue` | Right column (Craft Points tab): available CP recipes for the slot |
| `CpRecipeOption.vue` | Available CP recipe: name, cost, effect description. Disabled when insufficient CP |
| `CpProgressBar.vue` | Visual progress bar for crafting points |
| `ItemPickerDialog.vue` | Large modal: two-panel item browser (search/filter on left, item preview with full stats on right, "Select" button) |
| `SlotItemPicker.vue` | Item search with name, skill, armor type, resolved effect text search, and level filters. Preview mode for dialog |
| `AbilityPickerDialog.vue` | Modal popup: per-slot ability selection, search, tier constraint toggles, two-column family-grouped browser. Closes on selection |
| `AbilityBarSummary.vue` | Per-bar cards with inline skill selector dropdown, fixed-position ability grid, clickable slots, tooltip on hover, configurable sidebar slots |
| `AbilityFamilyOption.vue` | Clickable ability family card: icon, base name, tier selector buttons, selected tier stats, mod boost count |
| `BuildSummary.vue` | Right pane: tabbed effect views (By Skill, Effect Totals, By Ability). Uses precomputed TSys↔Ability index for "By Ability" tab |
| `AbilityDamageCard.vue` | Per-ability effect card with aggregated mod effects and source labels |
| `EffectLine.vue` | Color-coded effect (green positive, red negative), raw string or structured |
| `TierSelector.vue` | Segmented control (<=5 tiers) or StyledSelect dropdown (>5 tiers) |
| `GlobalModSearch.vue` | Cross-slot search of all assigned mods |

### Database Tables

| Table | Migration | Purpose |
|-------|-----------|---------|
| `build_presets` | v5 | Build definitions: name, skills, target level/rarity, notes |
| `build_preset_mods` | v5 | Assigned treasure mods per slot (power_name, tier, is_augment) |
| `build_preset_slot_items` | v6, v8, v9 | Base item per slot with per-slot level, rarity, crafted, masterwork, skill overrides |
| `build_preset_abilities` | v7 | Ability bar assignments (bar, slot_position, ability_id). Slot positions are preserved |
| `build_preset_cp_recipes` | v21 | CP-consuming recipe assignments. Same recipe can appear multiple times (stackable) |

### Data Flow

```
CDN (tsysclientinfo, tsysprofiles, abilities, skills, recipes)
  | parsed at startup
GameData (Rust, in-memory)
  | builds precomputed indices including TSys↔Ability cross-reference
  | queried via Tauri commands (batch lookups, O(1) per key)
buildPlannerStore (Pinia)
  | reactive state
BuildPlanner components (Vue)
  | user edits
Tauri commands -> SQLite (5 tables with cascade deletes)
```

---

## Feature Details

### Build Management

- Multiple named builds per character (scoped by `characterName@serverName`)
- Create, rename, delete builds via modal dialogs
- Auto-selects first build on load
- Build-level settings: default target level (1-125), default target rarity

### Paper Doll Equipment Layout (Left Pane)

- Build selector (dropdown + New/Rename/Delete) at top of left pane
- Collapsible "Set Defaults" section for rarity and level defaults
- 10 slots arranged in a paper doll grid with two columns:
  - **Left column**: Armor slots (Head, Chest, Legs, Feet, Hands)
  - **Right column**: Non-armor slots (Main Hand, Off Hand, Necklace, Ring, Belt)
  - **Center**: Aggregated stat bonuses from base items, armor set bonus indicators
- Each slot indicator shows: item icon, slot name, mod fill count, completion status (border color)
- Clicking a slot opens it in the center panel
- Ability bar summaries always visible below the equipment grid (no tabs)

### Slot Detail Panel (Center Panel)

When a slot is selected, the center panel shows:

**Header row**: Slot name, mod count badge, augment indicator

**Per-slot skill overrides**: Primary and secondary skill dropdowns that override the build defaults for this slot

**Two-column layout**:

**Left column** -- "What's on the item":
- **Item card**: Icon, name, level (editable inline), vendor value, rarity selector, crafted checkbox, masterwork/foretold checkbox (legendary only), resolved effect descriptions, change/clear buttons. Clicking "Change" opens the item picker dialog. Empty state shows "Select Base Item..." placeholder
- **Mod slots**: Vertical stack of 0-5 slots. Empty slots show what types of mods are allowed based on the constraint solver (e.g., "Ice Magic, Knife Fighting, or Generic", "Ice Magic only", "Generic / Endurance only"). "Slot Rules" hover tooltip shows the rarity's valid configurations, current skill counts, and remaining slot allowances
- **Augment slot**: Single augment slot (100 CP). Not shown for belts
- **Craft Points**: CP progress bar, assigned CP recipes grouped with counters (e.g., "Enhance Add Pockets To Cloth Shirt 12 CP x3"), minus button to remove one at a time. Not shown for belts

**Right column** -- "What's available" (tabbed):
- **Mods tab**: Skill filter dropdown (defaults to build skills + generic + endurance), text search, "My abilities" filter (uses precomputed TSys↔Ability index to show only ability-related mods), compact mode toggle. Click a mod row to add as regular mod; "+ Aug" button to add as augment (replaces existing). Each mod shows: icon, skill badge, display name, effects, equip slot chips, tier selector, ability boost labels
- **Craft Points tab**: Available CP recipes for this slot (shamanic infusions filtered by slot compatibility; crafting enhancements filtered by slot body part via recipe name matching). Recipes disabled when insufficient CP. Same recipe can be added multiple times

### Mod Distribution Enforcement

The planner enforces valid mod configurations using a constraint solver:
- Each rarity has a set of valid configurations (`RARITY_CONFIGS` in `buildPlanner.ts`)
- As mods are added, impossible configurations are eliminated
- The constraint solver (`computeSlotConstraints`) determines: which existing skills can grow, whether generic mods can be added, whether new skills can be introduced
- Empty slot labels update dynamically (e.g., after 3 Ice Magic + 1 Generic on Legendary, remaining slot shows "Knife Fighting only" if only one config remains valid)
- "Slot Rules" tooltip shows all remaining valid configurations

**Belt special handling**: Belts only accept generic mods (no skill-specific, not even endurance), have no CP budget, no augment slot, and no crafting point section.

### Base Item Selection

- **Item Picker Dialog** (large modal popup, ~90vw wide, 85vh tall):
  - **Left panel**: Full search with name text, skill filter, armor type filter, level range, effect text search (backend resolves `{TOKEN}{VALUE}` format effects before matching)
  - **Right panel** (384px): Item preview showing full ItemTooltip (icon, name, price, description, keywords, resolved effects, vendors, stack size), skill requirements, armor type badge, CP info
  - Click an item in the list to preview it, then click "Select [name]" to confirm

### Ability Bar Planning

- Three bars: Primary (6 slots), Secondary (6 slots), Sidebar (6-12 configurable slots, default 6)
- **Skill selector**: Each primary/secondary bar has a `StyledSelect` dropdown in the bar header (full width, shows full skill names). Fake combat skills (`IsFakeCombatSkill: true`) filtered out. Changing the skill clears all abilities on that bar
- **Per-slot assignment**: Each grid position is a fixed slot. Click any slot (filled or empty) to open the ability picker for that specific position
- **Filled slots**: Show ability icon with tooltip on hover (EntityTooltipWrapper + AbilityTooltip). Click to open picker and replace
- **Empty slots**: Dashed outline with "+" icon. Click to open picker and fill
- **Ability Picker Dialog** (modal popup, opens for a specific slot):
  - Title shows bar name + slot number + current ability name (if replacing)
  - "Clear Slot" button to empty the slot, "Clear All" to reset the bar
  - Search filter for ability names
  - Toggle: "Hide unlearned abilities" -- hides tiers above the character's current skill level (default off)
  - Toggle: "Limit tier by skill level" -- auto-selects the highest tier the character can use (default off)
  - Shows character's current skill level when available
  - Two-column grid of ability families. Families already assigned to other slots are hidden
  - **Family cards**: Clickable — whole row is the button. Icon, base name, mod boost count. Multi-tier: tier selector buttons. Single-tier: add on click
  - Picking an ability sets it at the exact slot position and closes the dialog
  - Sidebar loads abilities from primary + secondary skills (sidebar-eligible only) plus FirstAid, ArmorPatching, SurvivalInstincts

### Build Summary (Right Pane)

Persistent right pane (550px default, resizable to 900px) with tabbed effect views:

- **By Skill**: Mods grouped by skill with resolved display names, slot labels, and color-coded effects
- **Effect Totals**: Aggregated effects across all mods, sorted by magnitude, with source counts
- **By Ability**: Uses the precomputed TSys↔Ability index (single batch call, no text matching). Groups all mod effects by the abilities they affect. Shows ability name, effect count, individual effects with sources

### Cross-Referencing (TSys ↔ Ability)

Built on the precomputed backend index (see Architecture section above):
- `useBuildCrossRef.ts` composable makes one `getTsysAbilityMap()` batch call when slot powers change, caches the result
- `isAbilityRelated()` and `getAbilitiesForPower()` are pure in-memory lookups after the initial call
- ModBrowserItem shows "boosts: X, Y" labels for ability-related mods
- "My abilities" checkbox filters the mod list to only show ability-related mods
- Build Summary "By Ability" tab uses the same precomputed index
- Global mod search: when no slot selected, center panel shows cross-slot search of all assigned mods

### Crafting Points Budget

Three CP consumers tracked per slot:
- **Augments** (100 CP, max 1) -- not available on belts
- **Shamanic Infusion** (100 CP, max 1) -- fixed effects, from recipe `ResultEffects: AddItemTSysPower()`
- **Crafting Enhancements** (5-20 CP, stackable) -- from recipe `ResultEffects: CraftingEnhanceItem*()`

Backend `get_cp_recipes_for_slot` parses recipe `ResultEffects` to find applicable recipes. Crafting enhancements filtered by slot body part (recipe name matching: "Pants"→Legs, "Shirt"→Chest, etc.). CP recipes shown in a tab on the right column; assigned recipes displayed as grouped counters with x-multiplier on the left. Belts have 0 CP regardless of crafted/masterwork status.

---

## Not Yet Implemented

- Build notes field (exists in DB schema, not exposed in UI)
- Drag-and-drop for mods (native HTML5 DnD has issues in Tauri webview; click-to-add used instead)
- Drag-to-reorder abilities within a bar
- Validate `tsys_profile` -- confirm selected mods are possible on the chosen item type
- Show `skill_reqs` vs character's current skill levels with pass/fail indicators
- Build sharing (export as text/image/link, import from shared format)
- Current gear comparison (compare planned vs equipped gear, generate shopping list)
