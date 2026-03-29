# Gourmand Tracker

Tracks which foods a player has eaten for the Gourmand skill in Project Gorgon. Shows progress toward eating everything, highlights what's still needed, and lets players compare food buff combinations.

## Data Flow

```
CDN items (food_desc != null) → foods table (built during CDN ingestion)
Game Gourmand report (.txt)   → gourmand_eaten_foods table → Vue frontend
```

1. During CDN data refresh, items with a non-null `food_desc` field are parsed and inserted into a dedicated `foods` table (~569 items)
2. Player uses the Gourmand skill's "Request Skill Report" ability in-game, producing a `SkillReport_*.txt` file in the Books folder
3. On view mount, the app auto-imports the latest gourmand report from the Books folder, or the user can manually import via file picker
4. Eaten foods are persisted in `gourmand_eaten_foods` so data survives across sessions

## Database Schema

Two tables added in migration V3 ([`src-tauri/src/db/migrations.rs`](../../src-tauri/src/db/migrations.rs)):

**`foods`** — pre-parsed food data built during CDN ingestion:

| Column | Type | Description |
|--------|------|-------------|
| item_id | INTEGER PK | FK to items.id |
| name | TEXT | Food item name |
| icon_id | INTEGER | Icon reference |
| food_category | TEXT | `'Meal'`, `'Snack'`, or `'Instant-Snack'` |
| food_level | INTEGER | Parsed from `food_desc` (e.g., "Level 20 Meal" → 20) |
| gourmand_req | INTEGER | From `raw_json.SkillReqs.Gourmand` (nullable) |
| effect_descs | TEXT | JSON array of effect strings |
| keywords | TEXT | JSON array of keyword strings |
| value | REAL | Item gold value |

**`gourmand_eaten_foods`** — last-imported report snapshot:

| Column | Type | Description |
|--------|------|-------------|
| food_name | TEXT PK | Name as it appears in the report |
| times_eaten | INTEGER | How many times eaten |
| imported_at | TEXT | Timestamp of import |

## CDN Ingestion

Food table population is hooked into the CDN refresh cycle in [`src-tauri/src/db/cdn_persistence.rs`](../../src-tauri/src/db/cdn_persistence.rs):

- `clear_cdn_data()` drops all rows from `foods`
- `persist_cdn_data()` calls `insert_foods()` which iterates items with `food_desc IS NOT NULL`
- `parse_food_desc()` extracts category and level from the `"Level {N} {Category}"` format
- Gourmand skill requirement is extracted from the item's `SkillReqs.Gourmand` in the raw JSON

## Report Parsing

The gourmand report parser in [`src-tauri/src/db/gourmand_commands.rs`](../../src-tauri/src/db/gourmand_commands.rs) handles the report text format:

```
Gourmand Report for PlayerName
...
Foods Consumed:
  Super Fishy Surprise (HAS MEAT) (HAS DAIRY): 28
  Weird Fruit Cocktail: 25
```

- Lines after `"Foods Consumed:"` are parsed as food entries
- `strip_food_tags()` removes parenthesized tags like `(HAS MEAT)`, `(HAS DAIRY)`
- Each entry is extracted as `(name, count)` pairs
- On import, `gourmand_eaten_foods` is cleared and repopulated (snapshot model)

## Auto-Import

[`import_latest_gourmand_report`](../../src-tauri/src/db/gourmand_commands.rs) scans the Books folder for `SkillReport_*.txt` files, reads the first line to detect gourmand reports, and imports the latest one if the food count differs from the existing data.

## Backend Commands

All commands in [`src-tauri/src/db/gourmand_commands.rs`](../../src-tauri/src/db/gourmand_commands.rs):

| Command | Purpose |
|---------|---------|
| `get_all_foods` | Query all rows from the `foods` table |
| `import_gourmand_report` | Parse a user-selected report file and persist to DB |
| `get_gourmand_eaten_foods` | Return the last-imported eaten foods |
| `import_latest_gourmand_report` | Auto-import latest report from Books folder |
| `export_text_file` | Write uneaten foods list to a text file |

## Gourmand Level Resolution

The store resolves the player's Gourmand skill level with a priority chain:

1. **Manual override** — user-entered value via the header input
2. **Live session** — from `skillStore` if a Gourmand XP event has been seen
3. **Character snapshot** — from `characterStore` imported character data

This level is used to determine which foods the player can currently eat (foods with `gourmand_req > level` are marked as unusable).

## Frontend Components

```
src/components/Gourmand/
├── GourmandView.vue          # Top-level: header, progress, favorites, food buff, controls, food lists
├── FoodCategorySection.vue   # Renders a category (Meals/Snacks/Instant-Snacks) with sorting/filtering
├── FoodCard.vue              # Card view item with icon, name, level, category, eaten count
├── FoodListRow.vue           # Compact list view row
├── FoodItemWithTooltip.vue   # Wraps EntityTooltipWrapper + ItemTooltip for hover tooltips
├── FoodComparisonPanel.vue   # "Food Buff" panel: combines meal + snack effects with summed stats
└── GourmandProgressBar.vue   # Reusable progress bar (label, filled bar, X/Y percentage)
```

- [**GourmandView.vue**](../../src/components/Gourmand/GourmandView.vue) — main layout with a dynamic top row (progress bars + favorites + food buff panel), controls bar, and food category sections
- [**FoodCategorySection.vue**](../../src/components/Gourmand/FoodCategorySection.vue) — handles filtering (hide eaten, hide unusable) and sorting (gourmand level, food level, alphabetical; ascending/descending)
- [**FoodCard.vue**](../../src/components/Gourmand/FoodCard.vue) — card display with color coding: green (eaten), red (uneaten), dimmed (can't eat), gold border (selected)
- [**FoodListRow.vue**](../../src/components/Gourmand/FoodListRow.vue) — compact single-line display for list view mode
- [**FoodItemWithTooltip.vue**](../../src/components/Gourmand/FoodItemWithTooltip.vue) — lazy-loads full `ItemInfo` on hover via `getItemByName`, renders standard `ItemTooltip`
- [**FoodComparisonPanel.vue**](../../src/components/Gourmand/FoodComparisonPanel.vue) — parses effect description strings, sums matching numeric stats from selected meal + snack, deduplicates text effects

## Store

[`src/stores/gourmandStore.ts`](../../src/stores/gourmandStore.ts) — Pinia store with:

- **State:** `allFoods`, `eatenFoods` (Map), `reportLoaded`, `loading`, `error`, `manualGourmandLevel`, `selectedMeal`, `selectedSnack`, `hideEaten`, `hideUnusable`, `sortMode`, `sortAsc`, `viewMode`
- **Computed:** category splits (meals/snacks/instant-snacks), progress stats per category, favorites (top 3 most-eaten per category), uneaten foods list, combined effects
- **Actions:** `loadAllFoods`, `loadEatenFoods`, `importReport`, `tryAutoImport`, `exportUneaten`, selection management, manual level override

## UI Features

- **Progress bars** — overall and per-category (meals, snacks, instant-snacks)
- **Favorites** — top 3 most-eaten foods per category
- **Food Buff panel** — select a meal + snack to see combined buff stats with numeric values summed
- **Card and list views** — toggle between grid cards and compact three-column lists
- **Sorting** — by gourmand level, food level, or alphabetical; ascending or descending
- **Filtering** — hide eaten foods, hide unusable foods (gourmand level too low)
- **Item tooltips** — hover any food for the full item tooltip (description, effects, keywords, value)
- **Click to select** — click meals/snacks to populate the food buff comparison panel
- **Export uneaten** — save remaining uneaten foods to a text file
- **Unmatched detection** — warns when report foods don't match CDN data (renamed/removed items)
