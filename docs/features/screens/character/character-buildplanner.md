# Character Build Planner

## Overview

The build planner is a **"plan what you want"** tool for designing combat builds. In Project Gorgon, the real power of a build comes from treasure effects (mods) on gear — picking the right mods for your skill pair across all equipment slots is the core optimization loop for endgame players.

The planner lets you pick two combat skills, set a target level/rarity, and then browse and assign treasure effects to each equipment slot. It also supports ability bar planning (primary, secondary, and sidebar) and saving multiple named builds per character.

## Game Mechanics Reference

### Rarity and Mod Distribution

Each piece of equipment has mods determined by its rarity. The rarity controls how many mods are skill-specific vs generic:

| Rarity | Total Mods | Skill-Specific | Generic |
|--------|-----------|----------------|---------|
| Common | 0 | 0 | 0 |
| Uncommon | 3 | up to 1 | 2+ |
| Rare | 3 | up to 2 (across 2 skills) | 1+ |
| Exceptional | 3 | 2 + 1 (two skills) | possible replacements |
| Epic | 4 | 2 + 2 (two skills) | possible replacements |
| Legendary | 5 | 3 + 2 (two skills) | possible replacements |

- **Augmentation** adds one extra mod to any item (via the augmentation crafting system), without changing rarity. Costs 100 crafting points from the item's budget.
- **Mods are unique per item** — no duplicates of the same power on one piece of gear.
- **Mastercrafted/Foretold** items follow legendary distribution but with 160 crafting points.
- **Crafting points**: Crafted items have 120 CP, dropped items have 100 CP. Mastercrafted/Foretold legendaries have a flat 160 CP.
- **Armor types**: Each armor piece has a material type (Cloth, Leather, Metal, Organic). Equipping 3+ armor pieces of the same type grants a 3-piece set bonus.

### Treasure Effect (TSys) System

Each mod (treasure effect) in the CDN has:
- **Skill**: which combat skill it belongs to (e.g. "Sword", "Psychology"), or generic
- **Slots**: which equipment slots it can appear on (e.g. `["Head", "OffHand"]`)
- **Tiers**: level-gated variants with escalating effects. Each tier specifies:
  - `MinLevel` / `MaxLevel` — item level range
  - `MinRarity` — minimum rarity required
  - `SkillLevelPrereq` — player skill level needed
  - `EffectDescs` — the actual effect text (resolved via attributes)

The data chain for browsing mods:
```
Player picks two skills →
  For each equipment slot →
    Filter TSysClientInfo where (skill matches OR generic) AND slot matches →
      Select tier based on target level →
        Display resolved effect descriptions
```

### Equipment Slots

Main equipment slots for planning:
- **Main Hand** — weapon (Sword, Staff, Hammer, etc.)
- **Off Hand** — shield, orb, bow, claw, etc.
- **Head, Chest, Legs, Hands, Feet** — armor slots
- **Ring, Necklace** — jewelry
- **Belt** — extra slot (limited mods available)
- **Earring, Navel** — less common, may be added later

Some skills use unique items that replace standard slots (e.g. cow barding in chest slot) — these are just alternate items for the same slot.

### Ability System

Each combat skill has abilities unlocked by level. In combat:
- **Primary skill bar**: 6 ability slots
- **Secondary skill bar**: 6 ability slots
- **Sidebar**: 6-12 slots for sidebar skills (First Aid, Armor Patching, Survival Instincts, temporary abilities, etc.)

Mods often boost specific abilities by name (e.g. "AimedShotBoost"), creating a direct link between mod selection and ability selection.

## References

- https://wiki.projectgorgon.com/wiki/Items
- https://wiki.projectgorgon.com/wiki/Augmentation
- https://wiki.projectgorgon.com/wiki/Treasure_Effects
- https://wiki.projectgorgon.com/wiki/Shamanic_Infusion
- https://wiki.projectgorgon.com/wiki/Transmutation
- https://wiki.projectgorgon.com/wiki/Combat
- https://wiki.projectgorgon.com/wiki/Armor
- https://www.gorgonexplorer.com/

---

## Current Implementation Status

### Phase 1: Core Treasure Effect Planner — Complete

The full mod planning system is implemented and functional:

**Backend (Rust):**
- `src-tauri/src/db/build_planner_commands.rs` — All CRUD commands for build presets and mods
- `src-tauri/src/cdn_commands.rs` — `get_combat_skills`, `get_tsys_powers_for_slot`, `get_tsys_power_info` commands
- Database migration v5 in `src-tauri/src/db/migrations.rs` — `build_presets` and `build_preset_mods` tables
- All 6 Tauri commands registered in `src-tauri/src/lib.rs`

**Frontend:**
- `src/stores/buildPlannerStore.ts` — Pinia store with full state management, CRUD, mod assignment with validation
- `src/types/buildPlanner.ts` — TypeScript interfaces, equipment slot and rarity constants
- Components in `src/components/Character/BuildPlanner/`:

| Component | Status | Notes |
|-----------|--------|-------|
| `BuildPlannerScreen.vue` | Done | Two-panel layout, auto-selects first preset |
| `BuildHeader.vue` | Done | Build selector, skill pickers, default level/rarity |
| `SlotGrid.vue` | Done | Vertical equipment slot list with per-slot level/rarity/crafted/masterwork controls |
| `SlotModPicker.vue` | Done | 3-column layout (Primary Skill, Secondary Skill, Craft Points) with per-slot skill pickers |
| `ModColumn.vue` | Done | Reusable mod column with skill dropdown, assigned mods pinned to top, available mods below |
| `ModOption.vue` | Done | Available mod with icon, skill badge, effects, +/+A buttons |
| `ModAssignment.vue` | Done | Assigned mod with icon, resolved display name and effects |
| `BuildSummary.vue` | Done | Slide-out panel with armor sets, CP overview, slot breakdown, and tabbed effect views (By Skill, Effect Totals, By Ability) |

**Working features:**
- Build CRUD (create, rename, delete, select)
- Two combat skill selection from CDN skill list
- Default target level (1-125) and rarity (Uncommon–Legendary) with per-slot overrides
- Per-slot skill selection: each slot can independently choose primary/secondary skills (overriding build defaults)
- Per-slot mod browsing filtered by slot's skills + slot + level tier
- Mod assignment with duplicate prevention and slot capacity enforcement
- Augment support (1 per slot, from any skill, costs 100 CP from slot budget)
- Mod/augment icons displayed from CDN attribute data
- Crafting points tracking: crafted items = 120 CP, dropped = 100 CP, mastercrafted/foretold = 160 CP flat
- Armor type detection from item keywords (Cloth, Leather, Metal, Organic) with 3-piece set bonus tracking
- Build summary with armor type breakdown, CP budget overview, and collapsible all-effects view grouped by skill
- Text filter/search across mod names and effect descriptions
- 3-column mod layout: Primary Skill, Secondary Skill, and Craft Points columns with per-column skill dropdowns
- Column skill defaults based on slot's assigned skills, can be freely changed for browsing
- Selected mods pinned to top of their respective column (no separate "selected mods" panel)
- Generic and Endurance available as column skill options alongside all combat skills
- Filterable equipment browser for base item selection (browse by slot, skill requirement, level range)
- Persistence to SQLite with replace-all save strategy
- Character-scoped builds (`characterName@serverName`)

**Not yet implemented from Phase 1 design:**
- Build notes field (exists in DB, not exposed in UI)
- Drag-to-reorder mods within a slot

### Phase 2: Ability Selection — Complete

Ability bar planning for primary, secondary, and sidebar bars.

**Backend:**
- Database migration v7: `build_preset_abilities` table with `(preset_id, bar, slot_position, ability_id, ability_name)`
- `set_build_preset_abilities` — replace-all per bar, `get_build_preset_abilities` — load all for preset
- Uses existing `get_abilities_for_skill` CDN command for ability browsing

**Frontend components:**

| Component | Status | Notes |
|-----------|--------|-------|
| `AbilityBarEditor.vue` | Done | Two-column: assigned bar + available abilities browser |
| `AbilityOption.vue` | Done | Ability display with icon, level, cooldown, damage type, costs |
| `AbilityBarSummary.vue` | Done | Accordion-style bar buttons with fill counts and ability icons |

**Working features:**
- Primary (6 slots), secondary (6 slots), and sidebar (10 slots) ability bars
- Browse abilities by skill, filtered and sorted by level
- Sidebar bar includes abilities from First Aid, Armor Patching, and Survival Instincts
- Sidebar-eligibility filtering via `CanBeOnSidebar` CDN field
- Text filter/search across ability names and descriptions
- Duplicate prevention per bar
- Persistence to SQLite with per-bar replace-all strategy
- Assigned abilities displayed with `AbilityInline` (tooltip + navigation)

**Not yet implemented from Phase 2 design:**
- Cross-referencing between mods and abilities (highlighting which mods boost selected abilities)
- Drag-to-reorder abilities within a bar

### Phase 3: Base Item Selection — Complete

Per-slot item selection for choosing base items in each equipment slot.

**Backend:**
- Database migration v6: `build_preset_slot_items` table with `(preset_id, equip_slot)` → `item_id`
- `set_build_preset_slot_item`, `clear_build_preset_slot_item`, `get_build_preset_slot_items` commands
- Uses existing `search_items` CDN command with `equip_slot` filter

**Frontend:**

| Component | Status | Notes |
|-----------|--------|-------|
| `SlotItemPicker.vue` | Done | Search-and-select UI with debounced search, slot-filtered results |

**Working features:**
- Per-slot item search filtered by equipment slot
- Item results show icon, name, skill requirements, and craft points
- Selected item shown with `ItemInline` (tooltip + navigation)
- Item name shown in SlotGrid slot buttons
- Remove/change item support
- Persistence to SQLite with upsert strategy

**Not yet implemented (future extensions):**
- Validate `tsys_profile` — confirm selected mods are possible on the chosen item type
- Show `skill_reqs` vs character's current levels with pass/fail indicators

### Phases 4–5: Not Started

See design sections below for planned work on sharing and gear comparison.

---

## Design

### Layout

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Build: [Sword/Psych Dungeon ▾]   [+ New] [Rename] [Delete]            │
│  Skills: ⚔ Sword  +  🧠 Psychology    Target: Lv [90]  Rarity [Epic ▾] │
├────────────────────────────┬─────────────────────────────────────────────┤
│                            │                                             │
│  ── Equipment Slots ──     │  SLOT DETAIL / MOD BROWSER                  │
│                            │                                             │
│  ┌──────┐ ┌──────┐        │  Head — Epic (4 mods + 1 augment)           │
│  │ Head │ │ Neck │        │                                             │
│  │ 4/4  │ │ 2/4  │        │  ┌─ Skill Mods ─────────────────────────┐   │
│  └──────┘ └──────┘        │  │ 1. [Sword Boost]     +12% Sword dmg  │   │
│  ┌──────┐ ┌──────┐        │  │ 2. [Finishing Blow+]  +18 dmg to FB  │   │
│  │Chest │ │ Main │        │  │ 3. [Psych Heal Boost] +15% heal      │   │
│  │ 4/4  │ │ Hand │        │  │ 4. [Psych Armor+]    +10 armor       │   │
│  └──────┘ │ 3/4  │        │  ├─ Augment ────────────────────────────┤   │
│  ┌──────┐ └──────┘        │  │ 5. [Generic Max Health] +30 health   │   │
│  │ Legs │ ┌──────┐        │  └──────────────────────────────────────┘   │
│  │ 4/4  │ │ Off  │        │                                             │
│  └──────┘ │ Hand │        │  ── Available Mods ──────── [Filter: ____]  │
│  ┌──────┐ │ 0/4  │        │                                             │
│  │Hands │ └──────┘        │  ⚔ Sword Mods (for Head slot)               │
│  │ 2/4  │ ┌──────┐        │    Sword Boost — +12% base dmg              │
│  └──────┘ │ Ring │        │    Finishing Blow Boost — +18 dmg            │
│  ┌──────┐ │ 1/4  │        │    Parry Boost — +8% parry chance            │
│  │ Feet │ └──────┘        │    ...                                       │
│  │ 4/4  │                  │  🧠 Psychology Mods (for Head slot)          │
│  └──────┘                  │    Psych Heal Boost — +15% healing           │
│                            │    ...                                       │
│  ── Abilities ──           │  ⚙ Generic Mods (for Head slot)             │
│  Primary (Sword): 4/6      │    Max Health — +30 health                   │
│  Secondary (Psych): 6/6    │    ...                                       │
│  Sidebar: 3/6-12           │                                             │
│                            │                                             │
├────────────────────────────┴─────────────────────────────────────────────┤
│  ── Build Summary ──                                                     │
│  Sword effects: +12% base dmg, +18 Finishing Blow, ...                   │
│  Psychology effects: +15% heal, +10 armor, ...                           │
│  Generic effects: +30 health, +15 power, ...                             │
└──────────────────────────────────────────────────────────────────────────┘
```

### Build Header

Top bar with:
- **Build selector** — dropdown of saved builds for this character
- **New / Rename / Delete** — build management
- **Skill pickers** — two dropdowns filtered to combat skills (from CDN `combat: true`)
- **Target level** — numeric input, determines which mod tier to display
- **Target rarity** — dropdown (Uncommon through Legendary), determines mod slot count per equipment piece

### Left Panel — Slot Grid & Ability Summary

**Equipment Slots:**
- Visual grid of all equipment slots
- Each slot shows: slot name, fill count (e.g. "3/4" mods assigned)
- Color coding: fully filled = green, partially filled = yellow, empty = neutral
- Click a slot to open the mod browser in the right panel

**Ability Summary:**
- Compact display of ability bar status
- Click to switch right panel to ability picker mode
- Shows: "Primary (Sword): 4/6", "Secondary (Psych): 6/6", "Sidebar: 3/8"

### Right Panel — Mod Browser (default view)

When an equipment slot is selected, the right panel shows:

**Slot Header:**
- Slot name, target rarity, mod slot breakdown
- Based on rarity: how many skill-specific vs generic slots are available
- Augment slot shown separately

**Selected Mods:**
- List of currently assigned mods for this slot
- Each shows: power name, skill, resolved effect text at target tier
- Remove button on each
- Drag to reorder / reassign between skill-specific and generic slots

**Available Mods Browser:**
- Grouped by skill: Primary skill mods, Secondary skill mods, Generic mods
- Each entry shows: power name, effect description at target tier level
- Searchable/filterable by effect text
- Click to add to the slot (respecting uniqueness — no duplicates, and slot count limits)
- Mods already used in this slot are visually marked
- Mods used on OTHER slots in the build could be dimmed or flagged (since mods are unique per item, not per build — but this is a planning tool so duplicates across slots are valid)

**Data source:** New backend command that filters `TsysClientInfo` entries by skill + slot + target level tier range, returns resolved effect text.

### Right Panel — Ability Picker (alternate view)

When ability bars are selected in the left panel:

**Primary / Secondary Skill Bar:**
- Shows all abilities for the selected skill from CDN, sorted by level
- 6 slots per bar — drag or click to assign
- Each ability shows: name, icon, level requirement, cooldown, damage type, resource costs
- Abilities the character can't use yet (based on current skill levels from game state) shown dimmed with "Requires Level X"
- Highlight which selected mods boost each ability (cross-reference mod names containing ability names)

**Sidebar:**
- Configurable slot count: 6 to 12 (slider or input)
- Browse sidebar-eligible abilities (from First Aid, Armor Patching, Survival Instincts, and any other sidebar-flagged skills)
- Same assign/display pattern as primary/secondary

### Build Summary

Collapsible footer section aggregating all selected mods across the entire build:
- Grouped by skill, then by effect type
- Shows total effect budget at a glance
- Useful for checking overall build balance

---

## Implementation Phases

### Phase 1: Core Treasure Effect Planner

The highest-value feature — browse and assign mods to equipment slots for a skill pair.

**Backend (Rust):**

New file: `src-tauri/src/build_planner_commands.rs`

- `get_tsys_powers_for_slot(skill_primary, skill_secondary, equip_slot, target_level)` — Returns all eligible TSys powers for a slot, filtered by skill match (primary, secondary, or generic) and tier level range. Returns resolved effect descriptions at the appropriate tier.
- `get_all_combat_skills()` — Returns skills where `combat: true`, for the skill picker dropdowns.
- `create_build_preset(character_id, name, skill_primary, skill_secondary, target_level, target_rarity)` — Create a new build.
- `update_build_preset(id, ...)` — Update build metadata.
- `delete_build_preset(id)` — Delete a build and its mods (cascade).
- `get_build_presets(character_id)` — List all builds for a character.
- `set_build_preset_mods(preset_id, mods: Vec<{equip_slot, power_name, tier, is_augment, sort_order}>)` — Replace all mods for a build (simpler than individual CRUD).
- `get_build_preset_mods(preset_id)` — Get all mods for a build.

**Database migration** (next version after current):

```sql
CREATE TABLE build_presets (
    id INTEGER PRIMARY KEY,
    character_id TEXT NOT NULL,
    name TEXT NOT NULL,
    skill_primary TEXT,
    skill_secondary TEXT,
    target_level INTEGER DEFAULT 90,
    target_rarity TEXT DEFAULT 'Epic',
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE build_preset_mods (
    id INTEGER PRIMARY KEY,
    preset_id INTEGER NOT NULL REFERENCES build_presets(id) ON DELETE CASCADE,
    equip_slot TEXT NOT NULL,
    power_name TEXT NOT NULL,
    tier INTEGER,
    is_augment INTEGER DEFAULT 0,
    sort_order INTEGER DEFAULT 0
);

CREATE INDEX idx_build_presets_character ON build_presets(character_id);
CREATE INDEX idx_build_preset_mods_preset ON build_preset_mods(preset_id);
```

**Frontend:**

New store: `src/stores/buildPlannerStore.ts`
- Active build state, CRUD operations, mod assignment logic
- Computed properties for slot fill counts, mod availability, build summary
- Follows patterns from `src/stores/craftingStore.ts`

New components in `src/components/Character/BuildPlanner/`:

| Component | Purpose |
|-----------|---------|
| `BuildPlannerScreen.vue` | Top-level screen, mounted in CharacterView's Build Planner tab |
| `BuildHeader.vue` | Build selector, skill pickers, target level/rarity |
| `SlotGrid.vue` | Visual equipment slot layout with fill indicators |
| `SlotModPicker.vue` | Right panel: selected mods + available mods browser for a slot |
| `ModOption.vue` | Single mod in the available list — name, skill badge, effect text, add button |
| `ModAssignment.vue` | Single assigned mod — name, effect, remove button |
| `BuildSummary.vue` | Slide-out overlay panel with build overview and tabbed effect views |

Reuses existing:
- `ModPowerInline.vue` — mod display with tooltip
- `SkillInline.vue` — skill references
- Effect resolution pipeline (`resolve_effect_descs`, `get_tsys_power_info`)
- Two-panel layout pattern from Skills/NPCs/Quests screens

### Phase 2: Ability Selection

Add ability bar planning alongside the mod planner.

**Backend:**
- `get_sidebar_abilities()` — abilities from sidebar-eligible skills (First Aid, Armor Patching, Survival Instincts, etc.)
- Ability data already available via `get_abilities_for_skill()`

**Database:**
- New table `build_preset_abilities` — preset_id, bar (primary/secondary/sidebar), ability_id, slot_position
- Migration added to the same version or next

**Frontend:**

| Component | Purpose |
|-----------|---------|
| `AbilityBarEditor.vue` | Ability slot picker for one bar (6 slots) |
| `SidebarEditor.vue` | Sidebar ability picker (6-12 configurable slots) |
| `AbilityOption.vue` | Single ability in the browse list — icon, name, level req, cooldown, costs |

**Cross-referencing:** When an ability is selected, highlight mods in the build that boost it. When a mod is selected, show which ability it boosts. This connection is derivable from mod internal names (e.g. "AimedShotBoost" → "Aimed Shot" ability).

### Phase 3: Base Item Selection (in progress)

Per-slot item selection so users can choose which base item they want for each equipment slot.

**Backend:**
- Database migration v6: `build_preset_slot_items` table mapping `(preset_id, equip_slot)` → `item_id`
- Database migration v8: added `slot_level`, `slot_rarity`, `is_crafted`, `is_masterwork` columns to `build_preset_slot_items`
- Database migration v9: added `slot_skill_primary`, `slot_skill_secondary` columns for per-slot skill overrides
- New commands: `set_build_preset_slot_item`, `clear_build_preset_slot_item`, `get_build_preset_slot_items`, `update_build_preset_slot_props`
- Item search already available via existing `search_items` command with `equip_slot` filter

**Frontend:**
- `SlotItemPicker.vue` — search-and-select UI for choosing a base item per slot
- Items displayed in SlotGrid and SlotModPicker header
- Uses existing `searchItems()` from `gameDataStore` with slot filtering
- Shows item name, icon, skill requirements, and craft points

**Future extensions (not in scope yet):**
- Validate `tsys_profile` — confirm selected mods are possible on the chosen item type
- Show `skill_reqs` vs character's current levels with pass/fail indicators
- Show item stats, effect descriptions

### Phase 4: Sharing (future, shape TBD)

- Export build as formatted text (mod list per slot)
- Export as image (screenshot-style, like skills screen sharing)
- Compact shareable code/link encoding
- Import from shared format

### Phase 5: Current Gear Comparison (future)

- Compare planned build against actual equipped gear (from game state / inventory snapshots)
- Highlight differences: what you have vs what you want
- "Shopping list" of mods you still need to find/craft/augment

---

## Architecture

### Files

| Layer | File | Purpose |
|-------|------|---------|
| Backend | `src-tauri/src/db/build_planner_commands.rs` | Build planner persistence (presets, mods, slot items, abilities) |
| Backend | `src-tauri/src/cdn_commands.rs` | CDN queries (combat skills, TSys powers, power info, abilities) |
| Backend | `src-tauri/src/db/migrations.rs` | v5: build tables, v6: slot items, v7: abilities, v8: per-slot level/rarity, v9: per-slot skills |
| Store | `src/stores/buildPlannerStore.ts` | Pinia store for build state + CRUD |
| Types | `src/types/buildPlanner.ts` | TypeScript interfaces for builds, mods, slot items, abilities |
| Screen | `src/components/Character/BuildPlanner/BuildPlannerScreen.vue` | Top-level screen |
| Components | `src/components/Character/BuildPlanner/*.vue` | All sub-components (10 files) |

### Data Flow

```
CDN (tsysclientinfo, tsysprofiles, abilities, skills)
  ↓ parsed at startup
GameData (Rust, in-memory)
  ↓ queried via Tauri commands
buildPlannerStore (Pinia)
  ↓ reactive state
BuildPlanner components (Vue)
  ↓ user edits
Tauri commands → SQLite (build_presets, build_preset_mods, build_preset_slot_items, build_preset_abilities)
```

### Key Design Decisions

1. **Gear-first, not ability-first.** Treasure effects are the primary optimization vector. Abilities come second because mod selection drives ability effectiveness.

2. **"Plan what you want" over "see what you have."** The planner is aspirational — it doesn't require inventory data. Current-gear comparison is a future enhancement.

3. **Target level + rarity as build parameters.** Rather than picking tiers manually, the user sets a target level and rarity for the build. The system automatically selects the appropriate mod tier and enforces the correct mod slot distribution.

4. **Multiple builds per character.** Players swap skills, run different content, and experiment. Named builds with full CRUD support this.

5. **Mod browsing is the core interaction.** The right panel mod browser — filtered by skill, slot, and level — is where users spend most of their time. It needs to be fast, searchable, and clear about what each mod does.

6. **Sidebar abilities are first-class.** The 6-12 configurable sidebar slots are included from the ability planning phase, not treated as an afterthought.
