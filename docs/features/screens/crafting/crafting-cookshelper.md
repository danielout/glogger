# Crafting — Cook's Helper

## Overview

A gourmand-aware recipe finder that cross-references uneaten foods with known food-producing recipes across all crafting skills. Helps players efficiently work through their gourmand food list by showing only recipes they know that produce foods they haven't eaten yet.

## Data Sources

- **Uneaten foods** — imported from a gourmand skill report (same parser as the Gourmand Tracker), or "Start Fresh" mode treats all foods as uneaten
- **Known recipes** — from `gameStateStore.knownRecipeKeys`, which tracks recipes the player has crafted at least once (keyed as `Recipe_{id}`)
- **Food-producing recipes** — loaded via `get_recipes_producing_items` backend command, which uses the `recipes_producing_item` index to find all recipes that produce any food item. Automatically covers Cooking, Sushi Preparation, Cheesemaking, Mycology, and any future food-producing skill.
- **Food items** — from the `foods` table (populated during CDN ingestion)

## How it works

1. Player imports a gourmand skill report (or clicks "Start Fresh")
2. The store loads all foods, then requests all recipes that produce any food item (food-first approach)
3. `helpfulRecipes` filters to recipes where:
   - The player knows the recipe (has crafted it before, matched via `Recipe_{id}` key)
   - The recipe produces at least one uneaten food item (matched via `result_item_ids` → food `item_id`)
4. Results can be filtered by skill (dynamically derived from loaded recipes), material availability, and search text
5. Optional "Check Materials" runs ingredient resolution + inventory availability for each recipe

## Vendor-Purchasable Awareness

When materials are checked, recipes where the only missing ingredients are vendor-purchasable (confirmed via CDN `sources_items.json` Vendor/Barter entries) are treated as craftable:

- **Filter behavior** — recipes with only vendor-purchasable shortfalls appear under "Can Craft", not "Missing Materials"
- **Status indicator** — gold dot (●) distinguishes "need to buy from vendor" from green checkmark (✓, all materials on hand) and yellow dot (●, truly missing materials)
- **Vendor price** — uses NPC vendor buy price (`value × 1.5`) only; player-set market prices do not affect vendor-purchasable classification

## UI

- **Import bar** — import gourmand report or start fresh; shows eaten/uneaten/craftable counts
- **Filter pills** — skill filter (All + one pill per skill found in results) with counts
- **Availability filter** — All / Can Craft / Missing Materials (requires material check). "Can Craft" includes recipes where missing items are vendor-purchasable.
- **Sort modes** — by name, skill level requirement, or food level
- **Search** — text filter on recipe and food names
- **Multi-select** — checkbox per recipe with select-all toggle
- **Project integration** — add selected recipes to an existing crafting project or create a new one

## Store

[`src/stores/cooksHelperStore.ts`](../../../../src/stores/cooksHelperStore.ts) — separate from the main crafting store:

- **State:** `importedEatenNames`, `allFoods`, `foodRecipes`, filters, selection, material needs
- **Computed:** `uneatenFoods`, `availableSkills` (derived from loaded recipes), `helpfulRecipes` (known + produces uneaten food), `filteredRecipes` (with filters/search/sort applied), `stats`
- **Actions:** `importFile`, `startFresh`, `checkAllMaterials`, selection management, `addToProject`, `createProjectFromSelection`
