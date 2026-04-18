# Widget: Recipe Items

**ID:** `recipe-items` | **Default size:** Medium | **Component:** `widgets/RecipeItemsWidget.vue`

Shows recipe-teaching items (recipe scrolls, skill books) found in the player's inventory and storage, classified by learnability:

- **Already Known** — all recipes bestowed by the item are already learned. Shown with a green "safe to sell" label. These are duplicates and can be safely sold or traded.
- **Ready to Learn** — the player hasn't learned the recipes yet but meets all skill requirements. Shown with a blue "can learn" label.
- **Missing Requirements** — the player doesn't meet the skill prerequisites. Each entry lists the unmet skills with current/required levels (e.g. "Toolcrafting 12/25").

Each section shows a scrollable list (max 160px) of `ItemInline` entries with stack counts when > 1. A footer summarizes total recipe items found.

## Data flow

1. Calls the `find_recipe_items_in_inventory` Tauri command with the active character/server
2. Backend aggregates items from `game_state_inventory` + `game_state_storage`, summing stack sizes by item type
3. For each item with a `BestowRecipes` field, resolves recipe internal names via `GameData::resolve_recipe()`
4. Checks each resolved recipe ID against `game_state_recipes` (known if `completion_count > 0`)
5. Checks item `SkillReqs` against `game_state_skills` for unmet requirements
6. Results sorted: known first, then learnable, then missing requirements, alphabetical within each group

Reloads automatically when inventory or storage state changes.

**Data source:** `find_recipe_items_in_inventory` Tauri command. Combines CDN item/recipe data with persisted inventory, storage, recipe, and skill state from SQLite.
