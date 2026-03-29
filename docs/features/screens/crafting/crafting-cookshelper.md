# Crafting — Cook's Helper

## Overview

A gourmand-aware recipe finder that cross-references uneaten foods with known Cooking/Sushi Making recipes. Helps players efficiently work through their gourmand food list by showing only recipes they know that produce foods they haven't eaten yet.

## Data Sources

- **Uneaten foods** — imported from a gourmand skill report (same parser as the Gourmand Tracker), or "Start Fresh" mode treats all foods as uneaten
- **Known recipes** — from `gameStateStore.knownRecipeKeys`, which tracks recipes the player has crafted at least once (keyed as `Recipe_{id}`)
- **Cooking/Sushi recipes** — loaded from CDN via `gameDataStore.getRecipesForSkill()` for both `Cooking` and `Sushi Making`
- **Food items** — from the `foods` table (populated during CDN ingestion)

## How it works

1. Player imports a gourmand skill report (or clicks "Start Fresh")
2. The store loads all foods and all Cooking + Sushi Making recipes from CDN
3. `helpfulRecipes` filters to recipes where:
   - The player knows the recipe (has crafted it before, matched via `Recipe_{id}` key)
   - The recipe produces at least one uneaten food item (matched via `result_item_ids` → food `item_id`)
4. Results can be filtered by skill (Cooking / Sushi Making), material availability, and search text
5. Optional "Check Materials" runs ingredient resolution + inventory availability for each recipe

## UI

- **Import bar** — import gourmand report or start fresh; shows eaten/uneaten/craftable counts
- **Filter pills** — skill filter (All / Cooking / Sushi Making) with counts
- **Availability filter** — All / Can Craft / Missing Materials (requires material check)
- **Sort modes** — by name, skill level requirement, or food level
- **Search** — text filter on recipe and food names
- **Multi-select** — checkbox per recipe with select-all toggle
- **Project integration** — add selected recipes to an existing crafting project or create a new one

## Store

[`src/stores/cooksHelperStore.ts`](../../../../src/stores/cooksHelperStore.ts) — separate from the main crafting store:

- **State:** `importedEatenNames`, `allFoods`, `cookingRecipes`, `sushiRecipes`, filters, selection, material needs
- **Computed:** `uneatenFoods`, `helpfulRecipes` (known + produces uneaten food), `filteredRecipes` (with filters/search/sort applied), `stats`
- **Actions:** `importFile`, `startFresh`, `checkAllMaterials`, selection management, `addToProject`, `createProjectFromSelection`
