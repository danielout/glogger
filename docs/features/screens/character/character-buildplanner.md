# Character Build Planner

## Overview

The build planner is a **"plan what you want"** tool for designing combat builds. In Project Gorgon, the real power of a build comes from treasure effects (mods) on gear -- picking the right mods for your skill pair across all equipment slots is the core optimization loop for endgame players.

The planner lets you pick two combat skills, set a target level/rarity, and then browse and assign treasure effects to each equipment slot. It also supports ability bar planning (primary, secondary, and sidebar), crafting point budget tracking across augments/shamanic infusion/crafting enhancements, and saving multiple named builds per character.

## Game Mechanics Reference

### Rarity and Mod Distribution

| Rarity | Total Mods | Primary Skill | Secondary Skill |
|--------|-----------|---------------|-----------------|
| Common | 0 | 0 | 0 |
| Uncommon | 3 | 1 | 0 |
| Rare | 3 | 1 | 1 |
| Exceptional | 3 | 2 | 1 |
| Epic | 4 | 2 | 2 |
| Legendary | 5 | 3 | 2 |

- **Augmentation** adds one extra mod to any item. Costs 100 CP from the item's budget.
- **Mods are unique per item** -- no duplicates of the same power on one piece of gear.
- **Mastercrafted/Foretold** items follow legendary distribution with 160 CP.
- **Crafting points**: Crafted items have 120 CP, dropped items have 100 CP, mastercrafted/foretold legendaries have 160 CP.
- **Armor types**: Armor pieces have a material type (Cloth, Leather, Metal, Organic). Equipping 3+ pieces of the same type grants a set bonus.
- **Belts**: Can only be Common (0 mods) or Uncommon (1 generic mod). Crafted via Shamanic Infusion or Toolcrafting.

### Crafting Point Budget

Three systems consume CP from an item's budget:

1. **Augmentation** (100 CP) -- adds one TSys power from any skill. Max 1 per item.
2. **Shamanic Infusion** (100 CP) -- adds fixed effects like +Armor, +Sprint Speed. Uses `AddItemTSysPower()` recipe effects. Max 1 per item.
3. **Crafting Enhancements** (5-20 CP) -- adds armor, pockets, or elemental damage via Tailoring/Blacksmithing recipes. Uses `CraftingEnhanceItem*()` recipe effects. Can stack multiple.

### Equipment Slots

- **Head, Chest, Legs, Hands, Feet** -- armor slots (participate in armor set bonuses)
- **Main Hand** -- weapon (Sword, Staff, Hammer, etc.)
- **Off Hand** -- shield, orb, bow, claw, etc.
- **Ring, Necklace** -- jewelry
- **Belt** -- limited mods (Common/Uncommon only), mapped to CDN's "Waist" slot

### Ability System

- **Primary skill bar**: 6 ability slots
- **Secondary skill bar**: 6 ability slots
- **Sidebar**: 10 slots for sidebar skills (First Aid, Armor Patching, Survival Instincts)

Mods often boost specific abilities by name, creating a link between mod and ability selection.

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

- **Header**: Build selector, skill pickers (StyledSelect), default level/rarity, build completeness indicator
- **Left pane** ("Equipment"): Tabbed between Equipment (slot grid by group) and Abilities (bar cards). Auto-switches on slot/bar click.
- **Center**: Mod browser (3-column: Primary/Secondary/CP), ability bar editor, or global mod search
- **Right pane** ("Build Summary"): Collapsible. Armor sets, CP budget, slot breakdown cards, tabbed effect views (By Skill, Effect Totals, By Ability)

### Files

| Layer | File | Purpose |
|-------|------|---------|
| Backend | `src-tauri/src/db/build_planner_commands.rs` | Persistence: presets, mods, slot items, abilities, CP recipes |
| Backend | `src-tauri/src/cdn_commands.rs` | CDN queries: combat skills, TSys powers, power info, CP recipes for slot |
| Backend | `src-tauri/src/db/migrations.rs` | v5: build tables, v6: slot items, v7: abilities, v8-v9: per-slot props, v21: CP recipes |
| Store | `src/stores/buildPlannerStore.ts` | Pinia store: build state, CRUD, mod/ability/CP recipe management |
| Types | `src/types/buildPlanner.ts` | Interfaces, constants (slots, rarities, CP types), helper functions |
| Composable | `src/composables/useBuildCrossRef.ts` | Mod-to-ability cross-referencing |
| Shared | `src/components/Shared/StyledSelect.vue` | Custom styled dropdown (dark theme, keyboard nav) |

### Components

All in `src/components/Character/BuildPlanner/`:

| Component | Purpose |
|-----------|---------|
| `BuildPlannerScreen.vue` | Top-level: PaneLayout with left/center/right panels, tab switching |
| `BuildHeader.vue` | Build CRUD, skill pickers, default level/rarity (all StyledSelect) |
| `BuildCompleteness.vue` | "X/10 slots, X/Y mods, X/3 bars" progress in header |
| `SlotGrid.vue` | Equipment slots organized by group (Armor, Weapons, Jewelry, Other) |
| `SlotCard.vue` | Per-slot: item icon, name, rarity, level, mod count, CP bar, crafted/MW toggles |
| `SlotModPicker.vue` | 3-column mod picker (Primary/Secondary/CP), compact mode, ability filter |
| `ModColumn.vue` | Reusable column: skill dropdown, assigned mods pinned top, available below |
| `ModOption.vue` | Available mod: icon, skill badge, color-coded effects, tier selector, ability labels |
| `ModAssignment.vue` | Assigned mod: icon, display name, effects, tier selector, remove button |
| `SlotItemPicker.vue` | Base item search with name, skill, armor type, effect text, and level filters |
| `AbilityBarEditor.vue` | Two-column: assigned abilities + available browser with mod boost indicators |
| `AbilityOption.vue` | Ability: icon, name, level, cooldown, costs, "X mods boost this" |
| `AbilityBarSummary.vue` | Card-style bars with skill name, fill count, ability icon strips |
| `BuildSummary.vue` | Right pane: armor sets, CP bar, slot cards, tabbed effect views |
| `SummarySlotCard.vue` | Per-slot summary: icon, resolved mod names, CP bar |
| `AbilityDamageCard.vue` | Per-ability effect card with aggregated mod effects |
| `EffectLine.vue` | Color-coded effect (green positive, red negative), raw string or structured |
| `CpProgressBar.vue` | Visual progress bar for crafting points |
| `TierSelector.vue` | Segmented control (<=5 tiers) or StyledSelect dropdown (>5 tiers) |
| `GlobalModSearch.vue` | Cross-slot search of all assigned mods |
| `CpRecipeOption.vue` | Available CP recipe: name, cost, effect description |
| `CpRecipeAssignment.vue` | Assigned CP recipe with type label and remove button |

### Database Tables

| Table | Migration | Purpose |
|-------|-----------|---------|
| `build_presets` | v5 | Build definitions: name, skills, target level/rarity, notes |
| `build_preset_mods` | v5 | Assigned treasure mods per slot (power_name, tier, is_augment) |
| `build_preset_slot_items` | v6, v8, v9 | Base item per slot with per-slot level, rarity, crafted, masterwork, skill overrides |
| `build_preset_abilities` | v7 | Ability bar assignments (bar, slot_position, ability_id) |
| `build_preset_cp_recipes` | v21 | CP-consuming recipe assignments (recipe_id, cp_cost, effect_type) |

### Data Flow

```
CDN (tsysclientinfo, tsysprofiles, abilities, skills, recipes)
  | parsed at startup
GameData (Rust, in-memory)
  | queried via Tauri commands
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
- Build-level settings: primary/secondary skill, default target level (1-125), default target rarity

### Equipment Slot Grid

- 10 slots organized by group: Armor (Head/Chest/Legs/Hands/Feet), Weapons (Main Hand/Off Hand), Jewelry (Ring/Necklace), Other (Belt)
- Each slot card shows: item icon (from CDN), slot name, rarity dropdown, level input, mod fill count, CP progress bar, item name (with tooltip), armor type badge, crafted/masterwork toggles
- Per-slot rarity and level overrides (independent of build defaults)
- Per-slot skill overrides (item can use different skills than the build default)
- Belt slot constrained to Common/Uncommon rarity only
- Status indicated via left-border accent: green for full, yellow for partial

### Mod Browser (Center Panel)

Three-column layout: Primary Skill, Secondary Skill, Craft Points

- **Skill columns**: Skill dropdown (any combat skill, Generic, Endurance), assigned mods pinned to top, available mods below
- **Each mod shows**: icon, skill badge, display name, color-coded effects (EffectLine), tier selector, ability cross-reference labels ("boosts: Thunderstrike")
- **Tier selector**: Segmented control for <=5 tiers, StyledSelect dropdown for >5 tiers
- **Compact mode**: Toggle hides effect details for denser scanning
- **"My abilities" filter**: When active, sorts ability-related mods to top
- **Text filter**: Search across mod names and effect descriptions
- **CP column**: Augment picker (1 per slot, 100 CP) + Shamanic Infusion section + Crafting Enhancement section. Available options loaded per-slot from CDN recipe data.

### Base Item Selection

- Accordion in SlotModPicker header, expands to show item search
- Filters: name text, skill requirement, armor type (Cloth/Leather/Metal/Organic), effect text, level range
- Results show: icon, name, skill requirements, armor type badge, craft points, target level
- Selected item determines armor type detection and CP budget

### Ability Bar Planning

- Equipment/Abilities tabs in left pane (equal prominence)
- Three bars: Primary (6 slots), Secondary (6 slots), Sidebar (10 slots)
- Each bar card shows: skill name, fill count, horizontal ability icon strip
- Ability browser: two-column layout with assigned abilities and available abilities
- Each ability shows: icon, name, level, cooldown, damage type, costs, "X mods boost this" indicator
- Sidebar includes abilities from First Aid, Armor Patching, Survival Instincts
- Sidebar-eligibility filtering via `CanBeOnSidebar` CDN field

### Build Summary (Right Pane)

Persistent, collapsible right pane with:

- **Skills & target**: Primary/secondary skill names, target level/rarity
- **Armor sets**: Armor type counts with 3-piece bonus indicators
- **Crafting points**: Total CP progress bar across all slots and all CP sources
- **Slot breakdown**: Per-slot summary cards (SummarySlotCard) showing icon, resolved mod names, CP bar. Only shows slots with assigned mods.
- **Tabbed effect views**:
  - **By Skill**: Mods grouped by skill with resolved display names and color-coded effects
  - **Effect Totals**: Aggregated effects across all mods, sorted by magnitude, with source counts
  - **By Ability**: AbilityDamageCard per ability showing which mod effects reference it

### Cross-Referencing

- `useBuildCrossRef.ts` composable maps powers to assigned abilities via effect text matching
- ModOption shows "boosts: X, Y" when power effects mention assigned abilities
- AbilityOption shows "X mods boost this" count (matches against presetMods power names)
- Global mod search: when no slot/bar selected, center panel shows cross-slot search of all assigned mods

### Crafting Points Budget

Three CP consumers tracked per slot:
- **Augments** (100 CP, max 1) -- standard TSys power from any skill
- **Shamanic Infusion** (100 CP, max 1) -- fixed effects, from recipe `ResultEffects: AddItemTSysPower()`
- **Crafting Enhancements** (5-20 CP, stackable) -- from recipe `ResultEffects: CraftingEnhanceItem*()`

Backend `get_cp_recipes_for_slot` command parses recipe `ResultEffects` to find applicable recipes per slot. Persisted in `build_preset_cp_recipes` table.

---

## Not Yet Implemented

- Build notes field (exists in DB schema, not exposed in UI)
- Drag-to-reorder mods within a slot
- Drag-to-reorder abilities within a bar
- Validate `tsys_profile` -- confirm selected mods are possible on the chosen item type
- Show `skill_reqs` vs character's current skill levels with pass/fail indicators
- Build sharing (export as text/image/link, import from shared format)
- Current gear comparison (compare planned vs equipped gear, generate shopping list)
