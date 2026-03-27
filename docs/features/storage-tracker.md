# Storage Vault Database

## Overview

A CDN-driven database of ALL storage vault locations in the game, accessible as the "Vaults" tab under the Inventory section. Shows every vault organized by area with capacity tracking, NPC names, item restrictions, and per-area aggregate summaries. Tracks vault contents live via Player.log events and seeds from inventory snapshots.

## Location

**Inventory > Vaults tab** — alongside the "Inventory" (live tracking) and "Storage" (snapshot browser) tabs in `InventoryWrapper`.

## How It Works

Storage in Project Gorgon is NPC-based — players unlock vault slots by gaining favor with NPCs. All vaults within a zone are accessible from any storage NPC in that zone. The vault database shows ALL ~92 vaults from CDN data grouped by area, regardless of whether the player has items stored there.

### Data Sources

| Source | What It Provides |
|--------|-----------------|
| **CDN `storagevaults.json`** | Vault metadata: NPC names, areas, slot tiers, item restrictions (~92 vaults) |
| **Player.log events** | Real-time `StorageDeposit` / `StorageWithdrawal` events |
| **Inventory snapshot** (`/outputitems`) | Full vault contents as point-in-time seed data |
| **NPC favor** (game state) | Current favor tier per NPC, used to calculate unlocked slot count |
| **Attributes** (game state) | Attribute-based slot counts for special vaults |

### Vault Key Resolution

The critical mapping problem: `ProcessAddToStorageVault(14804, ...)` log lines contain a game **entity ID** (e.g., 14804) that does NOT match `storagevaults.json` `ID` fields (which max around ~2100).

**Solution:** The parser uses `current_interaction.npc_name` — set by the `ProcessVendorScreen` event that always fires before storage operations — to get the CDN-compatible vault key (e.g., `"NPC_Joe"`). This `vault_key` field is added to both `StorageDeposit` and `StorageWithdrawal` events at parse time.

### Slot Capacity Calculation

Vault capacity is computed on the frontend by cross-referencing vault metadata with the player's current game state. Two functions in `gameStateStore` handle this:

**`getVaultMaxPossibleSlots(vault)`** — The theoretical maximum slots if the player had max favor:
1. **`num_slots`** — Static slot count (skips 0, since NPC vaults have `num_slots: 0` in CDN data).
2. **`slot_attribute`** — Falls back to `num_slots_script_atomic_max` for the theoretical max.
3. **`levels`** — Returns the highest slot count from any favor tier.
4. **`num_slots_script_atomic_max`** — Maximum possible slots from a scripted source.

**`getVaultUnlockedSlots(vault)`** — The player's currently unlocked slots based on favor/attributes:
1. **`num_slots`** — Static slot count (always fully unlocked, skips 0).
2. **`slot_attribute`** — Current attribute value from `attributesByName`.
3. **`levels`** — Looks up the player's favor tier for the NPC via `vault.key` (e.g., `"NPC_Joe"`) in `favorByNpc`. Character reports and log events both store favor keyed by this same NPC key format. Finds the highest qualifying tier's slot count using `FAVOR_TIER_ORDER`.
4. **`num_slots_script_atomic_max`** — Fallback.

**`getVaultFavorTier(vault)`** — Returns the player's current favor tier with the vault's NPC (null for non-NPC vaults).

**Important:** NPC vaults in CDN data have `num_slots: 0` (not null). Both functions skip `num_slots` when it's 0 to avoid short-circuiting before the `levels` check.

### Storage Persistence

Unlike inventory (which clears on login), storage persists across sessions. The `GameStateManager` does NOT clear storage on `CharacterLogin`. Items are only removed by explicit `StorageWithdrawal` events or full snapshot replacement.

## Architecture

### Files

**Backend (Rust):**
- [`src-tauri/src/game_data/storage_vaults.rs`](../../src-tauri/src/game_data/storage_vaults.rs) — CDN `storagevaults.json` parsing with typed `StorageVaultInfo` struct
- [`src-tauri/src/player_event_parser.rs`](../../src-tauri/src/player_event_parser.rs) — `vault_key` enrichment on `StorageDeposit` / `StorageWithdrawal`
- [`src-tauri/src/game_state.rs`](../../src-tauri/src/game_state.rs) — `StorageDeposit` → UPSERT, `StorageWithdrawal` → DELETE
- [`src-tauri/src/db/migrations.rs`](../../src-tauri/src/db/migrations.rs) — `game_state_storage` table
- [`src-tauri/src/db/game_state_commands.rs`](../../src-tauri/src/db/game_state_commands.rs) — `get_game_state_storage` query command
- [`src-tauri/src/db/character_commands.rs`](../../src-tauri/src/db/character_commands.rs) — Snapshot seeding for storage
- [`src-tauri/src/cdn_commands.rs`](../../src-tauri/src/cdn_commands.rs) — `get_storage_vault_metadata` command with area/grouping name resolution

**Frontend (Vue/TS):**
- [`src/stores/gameStateStore.ts`](../../src/stores/gameStateStore.ts) — `storage`, `storageVaults`, `storageByVault`, `storageVaultsByKey`, `getVaultMaxPossibleSlots()`, `getVaultUnlockedSlots()`, `getVaultFavorTier()`
- [`src/types/gameState.ts`](../../src/types/gameState.ts) — `GameStateStorageItem`, `StorageVaultDetail` interfaces
- [`src/components/Inventory/VaultDatabaseTab.vue`](../../src/components/Inventory/VaultDatabaseTab.vue) — Main vault database tab with search, card grid, and detail panel
- [`src/components/Inventory/VaultAreaCard.vue`](../../src/components/Inventory/VaultAreaCard.vue) — Compact area summary card for the grid
- [`src/components/Inventory/VaultRow.vue`](../../src/components/Inventory/VaultRow.vue) — Individual vault display with capacity bar and expandable item list

### Data Flow

```
Player.log → LogWatcher → PlayerEventParser (adds vault_key)
                                    ↓
                          DataIngestCoordinator
                               ↓              ↓
                    GameStateManager     emit("player-event")
                    (UPSERT/DELETE)
                           ↓
               emit("game-state-updated", ["storage"])
                           ↓
                    gameStateStore.refreshDomain("storage")
                           ↓
                    VaultDatabaseTab (reactive update)
```

Snapshot seeding follows a separate path:
```
/outputitems → character_snapshot_items (DB)
                        ↓
            seed_game_state_from_snapshot()
                        ↓
            DELETE all storage for character → INSERT from snapshot
```

## Database Schema

```sql
CREATE TABLE game_state_storage (
    character_name TEXT NOT NULL,
    server_name TEXT NOT NULL,
    vault_key TEXT NOT NULL,
    instance_id INTEGER NOT NULL,
    item_name TEXT NOT NULL,
    item_type_id INTEGER,
    stack_size INTEGER NOT NULL DEFAULT 1,
    slot_index INTEGER NOT NULL DEFAULT -1,
    last_confirmed_at TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'log',
    PRIMARY KEY (character_name, server_name, vault_key, instance_id)
);
```

- **`vault_key`** — Maps to `storagevaults.json` keys (e.g., `"NPC_Joe"`)
- **`source`** — Either `'log'` (from live events) or `'snapshot'` (from `/outputitems`)
- **`instance_id`** — Real IDs from log events; synthetic negative IDs (`-(i+1)`) from snapshots (avoids collisions)

## Frontend Components

### VaultDatabaseTab

Main tab container mounted under Inventory > Vaults. Features:
- **Search bar** — Filters vaults by NPC name, vault key, area name, or stored item names
- **Summary** — Total stored item count and vault count
- **Card grid** — Responsive `auto-fill` grid of `VaultAreaCard` components (one per area, ~220px min width)
- **Detail panel** — Clicking an area card opens a detail panel below the grid showing all vaults in that area via `VaultRow` components
- Areas with stored items sort first, then alphabetical. Location-independent vaults (`"*"` area) sort last.
- Vault metadata loaded on mount via `gameState.loadStorageVaults()`

### VaultAreaCard

Compact area summary card in the grid:
- Area name, vault count, stored item count
- Capacity bar showing aggregate used/total slots
- Click to select and show detail panel
- Highlighted border when selected

### VaultRow

Individual vault display:
- **NPC name** via `NpcInline` with full NPC data (tooltip + click-to-navigate) for NPC vaults, plain text for chests/other. NPC data loaded via `gameDataStore.getNpcByKey()`.
- **Favor tier badge** — Shows the player's current favor level with the NPC (e.g., "Friends", "Comfortable")
- **Capacity bar** — Color-coded: green (<70%), yellow (70-90%), red (>90%). Shows "used / unlocked (max)" format.
- **Item restriction** from `requirement_description`
- **Expandable item list** — Alphabetically sorted, each row uses `ItemInline` with stack counts
- Auto-expands if vault has ≤10 items

## CDN Vault Metadata

Key fields from `storagevaults.json` (~92 vaults):

| Field | Purpose | Vault Count |
|-------|---------|-------------|
| `NpcFriendlyName` | Display name for the vault NPC | ~57 |
| `Area` / `Grouping` | Geographic grouping (24 unique areas) | All |
| `Levels` | Favor tier → slot count map | ~57 (NPC vaults) |
| `NumSlots` | Static slot count | ~27 |
| `SlotAttribute` | Attribute-based slot count | ~5 |
| `RequiredItemKeywords` | Item type restrictions | ~11 |
| `RequirementDescription` | Human-readable restriction text | ~11 |

## Event Handling

| PlayerEvent | DB Operation | Notes |
|-------------|-------------|-------|
| `StorageDeposit` | UPSERT by (character, server, vault_key, instance_id) | Updates item_name, stack_size, slot_index |
| `StorageWithdrawal` | DELETE by (character, server, vault_key, instance_id) | Removes item from storage |
| `CharacterLogin` | *(no-op for storage)* | Storage persists across sessions |

## Snapshot Seeding

Storage is seeded from both import paths:

**Inventory import** (`/outputitems` → `import_inventory_report`): Seeds `game_state_storage` directly from the imported snapshot items. The frontend `characterStore` triggers a `refreshDomain('storage')` after import.

**Character report import** (`/outputcharacter` → `import_character_report`): Calls `seed_game_state_from_snapshot()` which includes storage seeding from the latest inventory snapshot.

Both paths follow the same logic:
1. All existing `game_state_storage` rows for the character+server are deleted
2. Items from `character_snapshot_items` where `storage_vault != ''` and `is_in_inventory = 0` are inserted
3. Synthetic negative instance IDs are used (`-(row_index + 1)`) since snapshot items don't carry real instance IDs
4. Source is set to `'snapshot'`

Live log events will gradually replace snapshot data with real instance IDs as the player interacts with storage.
