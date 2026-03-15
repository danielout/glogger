# Inventory Import & Display

Imports player inventory data from the game's `/outputitems` JSON export, stores snapshots in SQLite, and displays items in a filterable table enriched with CDN data (base item tooltips, resolved effect descriptions, TSys mod tooltips).

## Data Flow

```
Game /outputitems → JSON file in Reports/ → Rust parser → SQLite → Vue frontend
                                                 ↓
                                          CDN enrichment
                                    (items, attributes, TSys)
```

1. Player runs `/outputitems` in-game, producing `{CharName}_items_{timestamp}.json` in `Reports/`
2. Rust reads the JSON, validates `Report == "Storage"`, and inserts into `character_item_snapshots` + `character_snapshot_items`
3. Frontend queries snapshots and items, enriching display with CDN data on hover

## Auto-Import & Polling

Inventory import piggybacks on the existing character report polling system:

- On character activation, [`characterStore.initInventoryForActiveCharacter()`](../../src/stores/characterStore.ts) imports the latest `*_items_*.json` from the Reports folder via `import_latest_inventory_for_character`
- [`pollForNewReports()`](../../src/stores/characterStore.ts) checks for new inventory files on each poll cycle
- Deduplication uses `ON CONFLICT DO NOTHING` on the `(character_id, snapshot_date)` unique constraint

## Backend Commands

All inventory commands live in [`src-tauri/src/db/inventory_commands.rs`](../../src-tauri/src/db/inventory_commands.rs):

| Command | Purpose |
|---------|---------|
| `import_inventory_report` | Parse a JSON file path and insert snapshot + items |
| `get_inventory_snapshots` | List all snapshots for a character (id, date, item count) |
| `get_snapshot_items` | Return all items for a given snapshot |
| `get_inventory_summary` | Aggregate stats: total items, value, breakdowns by vault/rarity |
| `import_latest_inventory_for_character` | Scan Reports folder, find latest inventory file, import it |

CDN enrichment commands in [`src-tauri/src/cdn_commands.rs`](../../src-tauri/src/cdn_commands.rs):

| Command | Purpose |
|---------|---------|
| `resolve_effect_descs` | Convert `{ATTRIBUTE_TOKEN}{VALUE}` placeholders to human-readable text using attributes.json |
| `get_tsys_power_info` | Look up a TSys power by internal name + tier, returning skill, prefix/suffix, and resolved tier effects |

## Database Schema

Two tables in [`src-tauri/src/db/migrations.rs`](../../src-tauri/src/db/migrations.rs):

- **`character_item_snapshots`** — one row per import (character_id, snapshot_date, unique constraint for dedup)
- **`character_snapshot_items`** — one row per item stack, storing all fields from the game export: type_id, storage_vault, stack_size, value, item_name, rarity, slot, level, is_crafted, crafter, durability (REAL — game outputs floats), craft_points, uses_remaining, transmute_count, attuned_to, tsys_powers (JSON string), tsys_imbue_power, tsys_imbue_power_tier, pet_husbandry_state

## Deserialization Notes

The game JSON uses non-standard casing that requires special handling:

- Structs use `#[serde(rename_all = "PascalCase")]` for most fields
- `TypeID` (all-caps ID) needs an explicit `#[serde(rename = "TypeID")]` since `rename_all = "PascalCase"` would produce `TypeId`
- `Durability` is `Option<f64>` — the game outputs fractional values like `0.44006`

## Frontend Components

```
src/components/Character/
├── InventoryView.vue       # Top-level: snapshot selector, import button, summary stats
├── InventoryTable.vue      # Filterable/sortable item table
├── ItemPowerList.vue       # Parses TSysPowers JSON, renders ModPowerInline per power
└── ModPowerInline.vue      # Hover tooltip for a single TSys mod (skill, prefix/suffix, tier effects)
```

- [**InventoryView.vue**](../../src/components/Character/InventoryView.vue) — snapshot dropdown, manual import button, summary stat bar (total items, stacks, value, unique items)
- [**InventoryTable.vue**](../../src/components/Character/InventoryTable.vue) — text search + dropdown filters (vault, rarity, slot), sortable columns, rarity color coding, vault name formatting (`"CouncilVault"` → `"Council Vault"`, `"NPC_Ashk"` → `"Ashk's Storage"`)
- [**ItemPowerList.vue**](../../src/components/Character/ItemPowerList.vue) — parses the `tsys_powers` JSON string into `ModPowerInline` components
- [**ModPowerInline.vue**](../../src/components/Character/ModPowerInline.vue) — on hover, calls `get_tsys_power_info` to show skill, prefix/suffix, and resolved tier effects

Items use [`ItemInline`](../../src/components/Shared/Item/ItemInline.vue) with the `item-id` prop so the tooltip shows the CDN base item data even when the displayed name is the enchanted name (e.g., "Pierce-Resistant Thorian Coat of Armored Leaping" → tooltip shows base "Thorian Coat" stats).

## Effect Description Resolution

Item tooltips resolve raw effect placeholders like `{MAX_ARMOR}{197}` into readable text like `Max Armor +197`:

1. [`ItemTooltip.vue`](../../src/components/Shared/Item/ItemTooltip.vue) passes `effect_descs` to the `resolve_effect_descs` backend command
2. Backend looks up each `{TOKEN}` in attributes.json to get `Label` and `DisplayType`
3. `DisplayType` determines formatting: `AsPercent` → `+X%`, `AsBuffDelta` → `+X`, `AsBuffMod` → `+X%`, `AsInt` → raw integer, etc.

## Tooltip Rendering

[`EntityTooltipWrapper.vue`](../../src/components/Shared/EntityTooltipWrapper.vue) uses `Teleport to="body"` with `position: fixed` positioning computed from `getBoundingClientRect()`. This prevents tooltips from being clipped by `overflow-auto` containers like the inventory table's scrollable area.
