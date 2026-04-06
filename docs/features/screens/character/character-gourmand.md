# Character — Gourmand

## Overview

Tracks which foods a player has eaten for the Gourmand skill. Shows progress toward eating everything, highlights what's still needed, lets players compare food buff combinations, and supports exporting an uneaten foods list.

## Architecture

### Files

**Backend (Rust):**
- `src-tauri/src/db/gourmand_commands.rs` — report parsing, food queries, import/export
- `src-tauri/src/db/cdn_persistence.rs` — `foods` table population during CDN refresh

**Frontend (Vue/TS):**
- `src/stores/gourmandStore.ts` — Pinia store
- `src/components/Gourmand/GourmandView.vue` — main layout
- `src/components/Gourmand/FoodCategorySection.vue` — category with sorting/filtering
- `src/components/Gourmand/FoodCard.vue` — card view item
- `src/components/Gourmand/FoodListRow.vue` — compact list row
- `src/components/Gourmand/FoodItemWithTooltip.vue` — hover tooltip wrapper
- `src/components/Gourmand/FoodComparisonPanel.vue` — meal + snack buff comparison
- `src/components/Gourmand/GourmandProgressBar.vue` — reusable progress bar

## Data Flow

```
CDN items (food_desc != null) → foods table (built during CDN ingestion)
Game Gourmand report (.txt)   → gourmand_eaten_foods table → Vue frontend
```

1. During CDN refresh, items with non-null `food_desc` are parsed into a `foods` table (~569 items)
2. Player uses the Gourmand skill's "Request Skill Report" ability in-game, producing `SkillReport_*.txt`
3. On view mount, the app auto-imports the latest gourmand report from Books folder, or user imports manually
4. Eaten foods are persisted in `gourmand_eaten_foods` so data survives across sessions

## Report Parsing

The gourmand report format:

```
Gourmand Report for PlayerName
...
Foods Consumed:
  Super Fishy Surprise (HAS MEAT) (HAS DAIRY): 28
  Weird Fruit Cocktail: 25
```

- Lines after `"Foods Consumed:"` are parsed as food entries
- `strip_food_tags()` removes parenthesized tags like `(HAS MEAT)`, `(HAS DAIRY)`
- On import, `gourmand_eaten_foods` is cleared and repopulated (snapshot model)

## Gourmand Level Resolution

The store resolves the player's Gourmand level with a priority chain:
1. **Manual override** — user-entered value via header input
2. **Live session** — from skill store if a Gourmand XP event has been seen
3. **Character snapshot** — from character store imported data

This level determines which foods the player can currently eat (foods with `gourmand_req > level` are marked unusable).

## Layout

Uses `PaneLayout` (screen-key `char-gourmand`) with left + conditional right panes:
- **Left pane** — progress bars (overall + per-category) and favorites (top 3 per category)
- **Center** — header bar, controls, and food category lists (scrollable)
- **Right pane** — food buff comparison panel (only appears when a meal or snack is selected)

## UI Features

- **Progress bars** — overall and per-category (Meals, Snacks, Instant-Snacks), shown in left pane
- **Favorites** — top 3 most-eaten foods per category, shown in left pane
- **Food Buff panel** — select a meal + snack to see combined buff stats in the right pane
- **Card and list views** — toggle between grid cards and compact three-column lists
- **Sorting** — by gourmand level, food level, or alphabetical; ascending or descending
- **Filtering** — hide eaten foods, hide unusable foods (gourmand level too low)
- **Item tooltips** — hover any food for full item tooltip (description, effects, keywords, value)
- **Click to select** — click meals/snacks to populate the food buff comparison panel
- **Export uneaten** — save remaining uneaten foods to a text file
- **Unmatched detection** — warns when report foods don't match CDN data (renamed/removed items)
- **Color coding** — green (eaten), red (uneaten), dimmed (can't eat), gold border (selected)

## Database Tables

**`foods`** — pre-parsed food data built during CDN ingestion:

| Column | Type | Description |
|--------|------|-------------|
| item_id | INTEGER PK | FK to items.id |
| name | TEXT | Food item name |
| icon_id | INTEGER | Icon reference |
| food_category | TEXT | `'Meal'`, `'Snack'`, or `'Instant-Snack'` |
| food_level | INTEGER | Parsed from `food_desc` |
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

## Tauri Commands

| Command | Purpose |
|---------|---------|
| `get_all_foods` | Query all rows from `foods` table |
| `import_gourmand_report` | Parse user-selected report file, persist to DB |
| `get_gourmand_eaten_foods` | Return last-imported eaten foods |
| `import_latest_gourmand_report` | Auto-import latest report from Books folder |
| `export_text_file` | Write uneaten foods list to a text file |
