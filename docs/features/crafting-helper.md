# Crafting Helper

## Overview

A full-featured crafting assistant for Project Gorgon. Helps players plan crafting projects, calculate ingredient requirements, optimize XP leveling paths, track live crafting progress, process work orders, and review crafting history — all powered by CDN recipe/item/skill data, inventory snapshots, and character exports.

## Architecture

### Files

**Backend (Rust):**
- `src-tauri/src/db/crafting_commands.rs` — all Tauri commands (project CRUD, material checks, work orders, skill levels)
- `src-tauri/src/db/migrations.rs` — `migration_v6_crafting` (2 tables)

**Frontend (Vue/TS):**
- `src/types/crafting.ts` — all TypeScript interfaces
- `src/stores/craftingStore.ts` — Pinia store with project management, ingredient resolution, leveling optimizer, live tracking, work orders
- `src/components/Crafting/` — 12 Vue components (see Component Hierarchy below)

### Database Tables

- **`crafting_projects`** — id, name, notes, created_at, updated_at
- **`crafting_project_entries`** — project_id (FK), recipe_id (FK→recipes), recipe_name, quantity, sort_order, expand_intermediates

Both cascade on delete from `crafting_projects`.

### Component Hierarchy

```
CraftingView.vue                    — 5-tab container
├── QuickCalcTab.vue                — single-recipe calculator
│   └── IngredientTreeNode.vue      — recursive ingredient display
├── ProjectsTab.vue                 — project list + active project detail
│   ├── IngredientTreeNode.vue      — per-entry ingredient tree
│   ├── MaterialSummary.vue         — aggregated needs vs available
│   ├── PickupList.vue              — vault-organized pickup list
│   ├── ShoppingList.vue            — vendor shopping with cost estimates
│   └── LiveCraftingPanel.vue       — real-time craft detection overlay
├── LevelingTab.vue                 — XP leveling optimizer
├── HistoryTab.vue                  — crafting stats from character exports
│   └── SkillCraftingProgress.vue   — per-skill completion bars
└── WorkOrdersTab.vue               — work order harvester + project builder
```

---

## Feature 1: Quick Calculator

A lightweight single-recipe calculator for quick "how much do I need?" queries.

### How it works

- Search and select a recipe by name
- Enter desired craft count or output quantity (auto-calculates from batch size)
- See full ingredient breakdown with quantities, estimated cost, and XP gained
- Toggle intermediate expansion — recursively break craftable ingredients into base materials
- One-click "Add to Project" or "Check Inventory" for deeper analysis

### Ingredient Resolution

The core resolver (`resolveRecipeIngredients` in the store) handles:
- **Recursive expansion** — walks `recipe_ingredients` → checks if ingredient has a recipe via `getRecipesForItem()` → optionally expands
- **Chance-to-consume** — expected consumption = `stack_size × chance_to_consume × craft_count`
- **Batch awareness** — if a recipe produces `stack_size > 1` per craft, adjusts craft count accordingly
- **Probabilistic outputs** — recipes with `percent_chance < 100` calculate expected crafts needed
- **Circular dependency detection** — guards against recipe loops

---

## Feature 2: Crafting Projects

Users create named projects containing multiple target crafts. The helper aggregates all ingredient requirements, deduplicating shared materials.

### Project Management

- Create, rename, delete, and duplicate projects
- Add recipes via search, set desired quantity per entry
- Reorder entries with drag-and-drop
- Per-entry toggle for intermediate expansion
- Projects persist to SQLite across sessions

### Material Breakdown

For the entire project, the material summary shows:
- **Total ingredients needed** — summed across all recipes
- **Inventory stock** — what the player has on-hand (live tracking)
- **Storage stock** — from latest inventory snapshot, broken down by vault
- **Shortfall** — items still needed, with estimated vendor cost

### Pickup List

A vault-organized "go get these items" list:
- Groups needed materials by storage vault location
- Shows quantity to pick up from each vault
- Sorted so the player can visit each vault once

### Shopping List

Items the player doesn't have anywhere:
- Estimated vendor cost per item (`value × 1.5`)
- Total gold needed for all missing materials

---

## Feature 3: XP Leveling Optimizer

Given a skill and target level, computes a crafting plan considering XP rewards, first-time bonuses, and cost efficiency.

### Inputs

- **Skill** — any crafting skill (auto-populated list from CDN)
- **Current level** — auto-filled from latest character snapshot
- **Target level** — desired level
- **Strategy** — Combined, First-Time Rush, or Cost-Efficient
- **Include unlearned recipes** — toggle to show recipes not yet known
- **Excluded recipes** — manually remove specific recipes from consideration

### XP Calculation

- XP needed is computed from CDN `xp_tables` (per-level amounts, not cumulative)
- Per-recipe XP uses `reward_skill_xp` (standard) and `reward_skill_xp_first_time` (bonus for first craft)
- Recipes are matched by `reward_skill` (not `skill`) — some recipes grant XP in a different skill than the one used to craft
- First-time bonus eligibility checked against `character_recipe_completions` from character export
- `reward_skill_xp_drop_off_level` flags recipes that become inefficient past a certain level

### Strategies

- **First-Time Rush** — craft each unlearned recipe once for bonus XP, then grind the most efficient recipe
- **Cost-Efficient** — minimize gold spent per XP gained
- **Combined** — first-time bonuses first, then cost-efficient grinding

### Output

Results are displayed grouped by level transition (e.g., "Lv 33 → 34", "Lv 34 → 35"):
- Each level shows the recipes to craft, quantities, XP gained, and estimated cost
- Summary totals for total crafts, total cost, and XP breakdown
- One-click "Create Crafting Project" to convert the plan into a project with all recipe entries

---

## Feature 4: Crafting History

Shows historical crafting data from character exports.

### Statistics

- **Top crafted recipes** — sorted by completion count, enriched with CDN recipe data
- **Per-skill progress** — completion bars showing X of Y recipes crafted, total completions, uncrafted count
- **First-time bonus opportunities** — uncrafted recipes per skill (feeds into the leveling optimizer)

---

## Feature 5: Work Orders

Loads the player's memorized work orders from character export data and optionally includes un-memorized work order scroll items from inventory/storage.

### Data Flow

1. **Active work orders** — extracted from `ActiveWorkOrders` array in character snapshot `raw_json`
2. **Completed work orders** — from `CompletedWorkOrders` array in the same snapshot
3. **Inventory scrolls** (optional toggle) — queries `character_snapshot_items` for items matching `"Work Order for %"` or `"Scroll_%"` name patterns, then resolves their `BestowQuest` field through CDN items → quest internal name

Each work order is enriched with CDN quest data:
- Quest name, crafting skill (`WorkOrderSkill`), objective item and quantity
- Matching recipe (looked up via `getRecipesForItem` on the objective item)
- Industry XP reward and gold reward from quest rewards
- Industry level requirement

### UI

- Skill filter pills for quick filtering
- Status badges: ACTIVE (green), SCROLL (blue, for inventory items), done (muted)
- Checkbox multi-select with "Select All"
- "Create Project" button to batch-create a crafting project from selected work orders
- Summary bar showing total work orders, Industry XP, and gold

---

## Feature 6: Live Crafting Detection

Real-time tracking of crafting progress against active projects or quick-calc targets.

### How it works

- Listens to `player-event` Tauri events for `ItemAdded` and `ItemStackChanged`
- Matches detected item outputs against tracked recipe entries
- Updates crafts completed and detected output quantities in real-time
- Maintains a rolling log of recent craft detections (max 100 entries)

### Tracking modes

- **Project tracking** — monitors all recipe entries in the active project
- **Quick-calc tracking** — monitors a single recipe from the quick calculator

---

## Tauri Commands

### Project CRUD
- `create_crafting_project(name, notes) → project_id`
- `get_crafting_projects() → Vec<CraftingProjectSummary>`
- `get_crafting_project(project_id) → CraftingProject`
- `update_crafting_project(project_id, name, notes)`
- `delete_crafting_project(project_id)`
- `duplicate_crafting_project(project_id) → project_id`

### Entry Management
- `add_project_entry(project_id, recipe_id, recipe_name, quantity) → entry_id`
- `update_project_entry(entry_id, quantity, expand_intermediates)`
- `remove_project_entry(entry_id)`
- `reorder_project_entries(project_id, entry_ids)`

### Data Queries
- `check_material_availability(character_name, server_name, item_type_ids) → Vec<MaterialAvailability>`
- `get_latest_recipe_completions(character_name, server_name) → Vec<RecipeCompletionEntry>`
- `get_latest_skill_level(character_name, server_name, skill_name) → Option<(level, xp, xp_needed)>`
- `get_work_orders_from_snapshot(character_name, server_name) → WorkOrderData`

### CDN Commands
- `get_quest_by_internal_name(name) → Option<QuestInfo>` — resolves quest by InternalName (used for work order enrichment)
- `get_xp_table_for_skill(skill) → Vec<number>` — per-level XP amounts for leveling optimizer

---

## Key Design Decisions

- **`reward_skill` vs `skill`**: The leveling optimizer matches recipes by `reward_skill`, not `skill`. Many recipes grant XP in a different skill than the crafting skill used (e.g., a Blacksmithing recipe that rewards Armorsmithing XP).
- **XP tables are per-level**: CDN `XpAmounts` contains per-level XP amounts (not cumulative). The store converts these to cumulative values for range calculations.
- **Skill internal names**: CDN skills use internal names (e.g., `JewelryCrafting`) that differ from display names (`Jewelry Crafting`). The optimizer resolves display → internal name for proper recipe matching.
- **Work order key formats**: Character exports store work orders by quest `InternalName` (e.g., `Carpentry_QualityMeleeStaves`), while the CDN indexes quests by numeric keys (e.g., `quest_50344`). A `quest_internal_name_index` bridges this gap.
- **Inventory scroll detection**: Work order scrolls appear in inventory exports with either display names (`Work Order for ...`) or internal names (`Scroll_...`). The SQL query matches both patterns.
- **Ingredient resolution is frontend-side**: The recursive ingredient resolver runs in the store (TypeScript), not Rust, since it needs to call multiple CDN lookup functions and handle UI-driven expansion toggles.
