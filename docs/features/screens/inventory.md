# Inventory Screen

## Overview

The inventory screen provides three views into what the player owns: a real-time live inventory tracker, point-in-time inventory snapshots from character exports, and a comprehensive storage vault database showing items across all vaults with capacity tracking.

## Architecture

### Files

**Backend (Rust):**
- `src-tauri/src/db/inventory_commands.rs` — snapshot import, item queries, summary stats
- `src-tauri/src/game_data/storage_vaults.rs` — CDN vault metadata parsing

**Frontend (Vue/TS):**
- `src/components/Inventory/InventoryWrapper.vue` — 3-tab container
- `src/components/Inventory/LiveInventoryTab.vue` — real-time inventory tracking
- `src/components/Character/InventoryView.vue` — snapshot-based inventory browser
- `src/components/Inventory/VaultDatabaseTab.vue` — storage vault browser
- `src/components/Inventory/VaultAreaCard.vue` — area grouping card
- `src/components/Inventory/VaultRow.vue` — individual vault with items

**Stores:**
- `gameStateStore` — live inventory events, storage items, vault metadata, favor, attributes
- `characterStore` — snapshot management, import, snapshot items

### Component Hierarchy

```
InventoryWrapper.vue                — 3-tab container
├── LiveInventoryTab.vue            — real-time inventory from player log
├── InventoryView.vue               — snapshot-based browsing (from /outputitems)
│   ├── InventoryTable.vue          — detail list view
│   ├── InventorySmallGrid.vue      — small icon grid
│   ├── InventoryLargeGrid.vue      — large icon grid
│   └── InventoryItemPanel.vue      — card view
└── VaultDatabaseTab.vue            — all vaults with capacity tracking
    ├── VaultAreaCard.vue           — area summary card
    └── VaultRow.vue                — per-vault detail with items
```

## Per-Tab Documentation

- [inventory-live.md](inventory/inventory-live.md) — Live Inventory
- [inventory-snapshots.md](inventory/inventory-snapshots.md) — Inventory Snapshots
- [inventory-vaults.md](inventory/inventory-vaults.md) — Storage Vault Database

## Tauri Commands

### Snapshot Management
- `import_inventory_report(file_path) → InventoryImportResult` — import `/outputitems` JSON, seeds game_state_storage and game_state_inventory
- `get_inventory_snapshots(character_name, server_name?) → Vec<InventorySnapshotSummary>`
- `get_snapshot_items(snapshot_id) → Vec<SnapshotItem>` — full item details (rarity, level, durability, etc.)
- `get_inventory_summary(snapshot_id) → InventorySummary` — aggregate stats

### Vault Data
- `get_storage_vault_metadata() → Vec<StorageVaultDetail>` — CDN vault definitions with capacity info

## Database Tables

- **`character_item_snapshots`** — snapshot headers (character, server, timestamp)
- **`character_snapshot_items`** — individual items within snapshots
- **`game_state_inventory`** — current inventory items (seeded from latest snapshot)
- **`game_state_storage`** — current vault items (seeded from snapshot import)
- **`game_state_favor`** — NPC favor tiers (determines unlocked vault slots)
- **`game_state_attributes`** — player attributes (for attribute-scaled vaults)

## Key Design Decisions

- **Dual inventory tracking** — Live tab shows real-time session state (in-memory, transient), Snapshots tab shows persisted point-in-time data from exports. Both serve different use cases.
- **Snapshot import seeds game state** — importing an `/outputitems` report populates both `game_state_inventory` and `game_state_storage`, making vault data available immediately.
- **Vault capacity models** — vaults use multiple capacity systems (fixed slots, favor-tiered, attribute-scaled), all unified behind `getVaultUnlockedSlots()` / `getVaultMaxPossibleSlots()` helpers.
- **Entity resolution** — all item names use `ItemInline` and NPC names use `NpcInline` for consistent tooltip/navigation behavior.
