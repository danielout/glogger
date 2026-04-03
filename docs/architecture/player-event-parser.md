# Player Event Parser

The `PlayerEventParser` is a foundational module that parses all `[HH:MM:SS] LocalPlayer: ProcessXxx(...)` lines from Player.log into structured, identity-resolved events. Features subscribe to these events rather than parsing log lines themselves.

**Source:** [`src-tauri/src/player_event_parser.rs`](../../src-tauri/src/player_event_parser.rs)

## Architecture

```
Player.log line
      │
      ▼
 PlayerLogWatcher::poll()
      │
      ├── pattern matchers (login, chat path, area transition, skill update)
      ├── PlayerEventParser::process_line()   ← this module (runs first)
      │        │
      │        ▼
      │   Vec<PlayerEvent>
      │        │
      ├── SurveyParser::process_events(&player_events, raw_line)
      │        ← consumes PlayerEvents + raw line for ProcessMapFx
      │
      └── LogEvent::PlayerEventParsed(PlayerEvent)
             │
             ▼
      DataIngestCoordinator::process_player_events()
             │
             ├── accumulate into batch
             ├── flush → emit("player-events-batch", &batch)  ← frontend
             └── flush → emit("game-state-updated", &domains)
```

Every line from Player.log is fed through `process_line()`. The parser maintains internal state (instance registry, stack sizes, interaction context) so it can resolve item identities and compute deltas. It returns zero or more `PlayerEvent`s per line.

The `SurveyParser` is a downstream consumer — it receives the structured `PlayerEvent`s from this module rather than parsing raw lines itself. This avoids duplicating parse logic and ensures the survey feature benefits from identity resolution and encoded value decoding.

## Event Types

The `PlayerEvent` enum uses `#[serde(tag = "kind")]` — when serialized to the frontend, each event includes a `kind` field for discriminating the variant.

### Item Events

| Event | Log Source | Key Fields |
|---|---|---|
| `ItemAdded` | `ProcessAddItem` | `item_name`, `instance_id`, `slot_index`, `is_new` |
| `ItemStackChanged` | `ProcessUpdateItemCode` | `instance_id`, `item_type_id`, `old_stack_size`, `new_stack_size`, `delta`, `from_server`. **Only emitted when the parser has a prior stack observation** — the first `ProcessUpdateItemCode` for an existing item (loaded at session start) establishes a baseline and is suppressed. New items (`is_new=True`) seed stack=1 so their first update emits correctly. |
| `ItemDeleted` | `ProcessDeleteItem` | `instance_id`, `item_name`, `context` (see below) |

**Delete Context** — `ItemDeleted` includes a `context` field that classifies _why_ the item was removed:

| Context | Meaning | How Detected |
|---|---|---|
| `StorageTransfer` | Moved to storage vault | `ProcessDeleteItem` followed by `ProcessAddToStorageVault` with same instance ID |
| `VendorSale` | Sold to NPC vendor | `ProcessDeleteItem` followed by `ProcessVendorAddItem` or `ProcessVendorUpdateItem` |
| `Consumed` | Used up (crafting, quest, gift) | Reserved for future context detection |
| `Unknown` | Context not determined | Delete was not followed by a storage/vendor line |

### Skill Events

| Event | Log Source | Key Fields |
|---|---|---|
| `SkillsLoaded` | `ProcessLoadSkills` | `skills: Vec<SkillSnapshot>` — full snapshot of all skills |

Each `SkillSnapshot` contains: `skill_type`, `raw`, `bonus`, `xp`, `tnl` (i32, -1 = capped), `max`.

### NPC Events

| Event | Log Source | Key Fields |
|---|---|---|
| `InteractionStarted` | `ProcessStartInteraction` | `entity_id`, `interaction_type`, `npc_name` |
| `InteractionEnded` | `ProcessEndInteraction` | `entity_id` — clears `current_interaction` state |
| `FavorChanged` | `ProcessDeltaFavor` | `npc_id`, `npc_name`, `delta` (f32), `is_gift` |

### Vendor Events

| Event | Log Source | Key Fields |
|---|---|---|
| `VendorSold` | `ProcessVendorAddItem` | `price`, `item_name`, `instance_id`, `is_buyback` |
| `VendorStackUpdated` | `ProcessVendorUpdateItem` | `instance_id`, `item_type_id`, `new_stack_size`, `price` |
| `VendorGoldChanged` | `ProcessVendorUpdateAvailableGold` | `current_gold`, `server_id`, `max_gold` |

### Storage Events

| Event | Log Source | Key Fields |
|---|---|---|
| `StorageDeposit` | `ProcessAddToStorageVault` | `npc_id`, `slot`, `item_name`, `instance_id`, `vault_key` |
| `StorageWithdrawal` | `ProcessRemoveFromStorageVault` | `npc_id`, `instance_id`, `quantity`, `vault_key` |

**Vault Key Enrichment** — Both storage events include a `vault_key` field populated from `current_interaction.npc_name` (set by the preceding `ProcessVendorScreen`). This provides the CDN-compatible key (e.g., `"NPC_Joe"`) since the `npc_id` in the log line is a game entity ID that doesn't match `storagevaults.json` IDs. The `GameStateManager` uses `vault_key` to persist storage items keyed by their CDN vault.

### Action Events

| Event | Log Source | Key Fields |
|---|---|---|
| `DelayLoopStarted` | `ProcessDoDelayLoop` | `duration` (f32), `action_type`, `label`, `entity_id`, `abort_condition` |

This is a general-purpose event for any timed action (surveying, eating, crafting, using items). The `label` field contains the human-readable action name (e.g., `"Using Eltibule Green Mineral Survey"`, `"Surveying"`, `"Using Gobbledygook"`). The `action_type` distinguishes the category (e.g., `Eat`, `Unset`, `UseTeleportationCircle`).

### Screen/Book Events

| Event | Log Source | Key Fields |
|---|---|---|
| `ScreenText` | `ProcessScreenText` | `category`, `message` |
| `BookOpened` | `ProcessBook` | `title`, `content`, `book_type` |

### Skill Bar / Mount Events

| Event | Log Source | Key Fields |
|---|---|---|
| `ActiveSkillsChanged` | `ProcessSetActiveSkills` | `skill1`, `skill2` — active combat skill bar |
| `MountStateChanged` | `ProcessPlayerMount` | `entity_id`, `is_mounting` (bool) |

### World State Events

| Event | Log Source | Key Fields |
|---|---|---|
| `WeatherChanged` | `ProcessSetWeather` | `weather_name`, `is_active` (bool) |
| `CombatStateChanged` | `ProcessCombatModeStatus` | `in_combat` (bool) |

### Recipe Events

| Event | Log Source | Key Fields |
|---|---|---|
| `RecipeUpdated` | `ProcessUpdateRecipe` | `recipe_id`, `completion_count` |

### Attribute Events

| Event | Log Source | Key Fields |
|---|---|---|
| `AttributesChanged` | `ProcessSetAttributes` | `entity_id`, `attributes: Vec<AttributeValue>` — parallel arrays decoded into name/value pairs |

Each `AttributeValue` contains: `name` (String), `value` (f32). A single event can carry 1 to hundreds of attributes (large batch on login, small incremental during play).

### Login Snapshot Events

| Event | Log Source | Key Fields |
|---|---|---|
| `AbilitiesLoaded` | `ProcessLoadAbilities` | `skill1`, `skill2` — active skill pair at login (opaque `System.Int32[]` and `AbilityBarContents[]` arrays ignored) |
| `RecipesLoaded` | `ProcessLoadRecipes` | timestamp only — signal event (both `System.Int32[]` arrays are opaque C# types with no extractable data) |
| `EquipmentChanged` | `ProcessSetEquippedItems` | `entity_id`, `appearance` (full string), `equipment: Vec<EquipmentSlot>` — parsed slot assignments |

Each `EquipmentSlot` contains: `slot` (String — e.g., `"Chest"`, `"MainHand"`, `"MainHandEquip"`), `appearance_key` (String — the appearance model identifier including nested paren groups).

**C# Opaque Arrays:** Several login events log C# array types as `System.Int32[]` or `AbilityBarContents[]`. Unity's default `.ToString()` prints only the type name, not the array contents. These fields are unparseable and are ignored by the parser.

## Internal State

The parser is stateful. It tracks:

| State | Type | Purpose |
|---|---|---|
| `instance_registry` | `HashMap<u64, InstanceInfo>` | Maps instance IDs to item names and type IDs. Built from `ProcessAddItem` events at login. |
| `stack_sizes` | `HashMap<u64, u32>` | Last known stack size per instance ID. Used to compute deltas on `ProcessUpdateItemCode`. Seeded to 1 for new items (`is_new=True`); not seeded for existing items, so the first `ProcessUpdateItemCode` establishes a baseline without emitting a false delta. |
| `current_interaction` | `Option<InteractionContext>` | Tracks which NPC the player is currently interacting with. Set by `ProcessStartInteraction`. |
| `pending_deletes` | `Vec<PendingDelete>` | 1-line lookahead buffer for `ProcessDeleteItem` events awaiting context resolution. |

### Pending Delete Buffer

The game logs `ProcessDeleteItem` _before_ `ProcessAddToStorageVault` or `ProcessVendorAddItem`. Without buffering, we can't tell if a delete is a transfer or consumption. The parser holds deletes for one line:

```
Line N:   ProcessDeleteItem(12345)          → buffered
Line N+1: ProcessAddToStorageVault(... 12345 ...)  → matched! emit ItemDeleted(StorageTransfer)
```

If the next line is unrelated, the pending delete is flushed as `ItemDeleted(Unknown)`. At end of poll, `flush_all_pending()` is called to ensure nothing is left hanging.

### Encoded Value Decoding

`ProcessUpdateItemCode` and `ProcessVendorUpdateItem` use a packed 32-bit value:

```
encodedValue = (stackSize << 16) | itemTypeId

stackSize  = encodedValue >> 16      (high 16 bits)
itemTypeId = encodedValue & 0xFFFF   (low 16 bits)
```

Example: `1642723` → stack size `25`, item type ID `4323`.

## Listening to Events on the Frontend

Events are emitted via Tauri's event system as `"player-events-batch"` — an array of 1–50 `PlayerEvent` objects per emission. Each event includes a `kind` field for discriminating variants. See [live-event-streams.md](live-event-streams.md#batching-strategy) for batching details.

```typescript
import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'

// Type the event payloads
interface ItemAddedEvent {
  kind: 'ItemAdded'
  timestamp: string
  item_name: string
  instance_id: number
  slot_index: number
  is_new: boolean
}

interface ItemStackChangedEvent {
  kind: 'ItemStackChanged'
  timestamp: string
  instance_id: number
  item_name: string | null
  item_type_id: number
  old_stack_size: number
  new_stack_size: number
  delta: number
  from_server: boolean
}

// ... other event types follow the same pattern

type PlayerEvent =
  | ItemAddedEvent
  | ItemStackChangedEvent
  | { kind: 'ItemDeleted'; instance_id: number; item_name: string | null; context: string }
  | { kind: 'SkillsLoaded'; skills: SkillSnapshot[] }
  | { kind: 'InteractionStarted'; entity_id: number; interaction_type: number; npc_name: string }
  | { kind: 'FavorChanged'; npc_id: number; npc_name: string; delta: number; is_gift: boolean }
  | { kind: 'VendorSold'; price: number; item_name: string; instance_id: number; is_buyback: boolean }
  | { kind: 'VendorStackUpdated'; instance_id: number; item_type_id: number; new_stack_size: number; price: number }
  | { kind: 'StorageDeposit'; npc_id: number; slot: number; item_name: string; instance_id: number; vault_key: string | null }
  | { kind: 'StorageWithdrawal'; npc_id: number; instance_id: number; quantity: number; vault_key: string | null }
  | { kind: 'DelayLoopStarted'; duration: number; action_type: string; label: string; entity_id: number; abort_condition: string }
  | { kind: 'ScreenText'; category: string; message: string }
  | { kind: 'BookOpened'; title: string; content: string; book_type: string }

// Listen to all player events (batched — array of 1-50 events per emission)
const unlisten: UnlistenFn = await listen<PlayerEvent[]>('player-events-batch', (event) => {
  for (const e of event.payload) {
    switch (e.kind) {
      case 'ItemAdded':
        console.log(`Item added: ${e.item_name} (new: ${e.is_new})`)
        break
      case 'ItemStackChanged':
        console.log(`Stack changed: ${e.item_name} delta=${e.delta}`)
        break
      case 'FavorChanged':
        console.log(`Favor with ${e.npc_name}: +${e.delta}`)
        break
      // ... handle other events
    }
  }
})
```

## Adding Persistence for a Feature

When a feature needs to persist specific events, add a handler in the coordinator. Follow the `SurveySessionTracker` pattern:

1. Create a persistence module (e.g., `src-tauri/src/vendor_persistence.rs`)
2. Add a tracker struct to `DataIngestCoordinator`
3. Match on specific `PlayerEvent` variants in `process_player_events`

```rust
// In coordinator.rs
LogEvent::PlayerEventParsed(player_event) => {
    match &player_event {
        PlayerEvent::VendorSold { .. } => {
            self.vendor_tracker.process_sale(&player_event, &self.db_pool);
        }
        _ => {}
    }
    // player_event is accumulated into a batch and flushed periodically
    player_event_batch.push(player_event);
}
```

## Event Coverage

The parser currently handles 24 of ~60 known event types. See `docs/architecture/player-log-events.md` for the full reference of all known events with formats and examples. Events marked **NOT YET PARSED** in that doc need to be added here.

### Currently Parsed

| Event | PlayerEvent Variant |
|---|---|
| `ProcessAddItem` | `ItemAdded` |
| `ProcessUpdateItemCode` | `ItemStackChanged` |
| `ProcessDeleteItem` | `ItemDeleted` |
| `ProcessLoadSkills` | `SkillsLoaded` |
| `ProcessStartInteraction` | `InteractionStarted` |
| `ProcessEndInteraction` | `InteractionEnded` |
| `ProcessDeltaFavor` | `FavorChanged` |
| `ProcessVendorAddItem` | `VendorSold` |
| `ProcessVendorUpdateItem` | `VendorStackUpdated` |
| `ProcessVendorUpdateAvailableGold` | `VendorGoldChanged` |
| `ProcessAddToStorageVault` | `StorageDeposit` |
| `ProcessRemoveFromStorageVault` | `StorageWithdrawal` |
| `ProcessDoDelayLoop` | `DelayLoopStarted` |
| `ProcessScreenText` | `ScreenText` |
| `ProcessBook` | `BookOpened` |
| `ProcessSetActiveSkills` | `ActiveSkillsChanged` |
| `ProcessPlayerMount` | `MountStateChanged` |
| `ProcessSetWeather` | `WeatherChanged` |
| `ProcessUpdateRecipe` | `RecipeUpdated` |
| `ProcessCombatModeStatus` | `CombatStateChanged` |
| `ProcessSetAttributes` | `AttributesChanged` |
| `ProcessLoadAbilities` | `AbilitiesLoaded` |
| `ProcessLoadRecipes` | `RecipesLoaded` |
| `ProcessSetEquippedItems` | `EquipmentChanged` |
| `ProcessAddEffects` | `EffectsAdded` | Effect IDs + login batch flag |
| `ProcessRemoveEffects` | `EffectsRemoved` | Signal-only — opaque `System.Int32[]` |
| `ProcessUpdateEffectName` | `EffectNameUpdated` | Display name for effect instance |

### Planned — Medium Priority (Enrichment)

| Log Event | Proposed Variant | Notes |
|---|---|---|
| `ProcessLoadQuests` | `QuestsLoaded` | Full quest state on login |
| `ProcessAddQuest` | `QuestAdded` | New quest acquired |
| `ProcessUpdateQuest` | `QuestUpdated` | Quest progress |
| `ProcessCompleteQuest` | `QuestCompleted` | `(entityId, questId)` |
| `ProcessMountXpStatus` | `MountXpStatus` | XP eligibility per area |
| `ProcessSetStarredRecipes` | `StarredRecipesSet` | Favorited recipe IDs |
| `ProcessSetRecipeReuseTimers` | `RecipeCooldownsSet` | Recipe cooldown timers |
| `ProcessMapFx` | `MapMarkerAdded` | Survey results, resource discoveries |

### Low Priority / Future

Player vendor management, guild info, title/book lists, inventory folder settings, error messages, and other UI/system events. See `docs/architecture/player-log-events.md` for full details.

## Adding New Event Types

To parse a new `ProcessXxx` event:

1. Add a variant to `PlayerEvent` in [`player_event_parser.rs`](../../src-tauri/src/player_event_parser.rs)
2. Add a `parse_xxx` method on `PlayerEventParser`
3. Add a dispatch branch in `process_line` (check for `"ProcessXxx("`)
4. Add tests
5. Update this doc

The parser uses manual string operations (no regex). Follow the existing `parse_*` methods as templates — they all use `line.find()`, `split()`, `trim()`, and `parse()`.

## Reusable Helpers in parsers.rs

The following helpers live in [`parsers.rs`](../../src-tauri/src/parsers.rs) and are available to any module:

- `parse_timestamp(line) -> Option<String>` — extracts `[HH:MM:SS]` from line start
- `extract_field(line, key) -> Option<String>` — extracts `key=value` from `{...}` blocks
- `parse_loot_items(text) -> (Vec<LootItem>, bool)` — parses loot screen text like `"Malachite collected! Also found Quartz x3 (speed bonus!)"` into structured items
- `parse_item_with_quantity(text) -> Option<(String, u32)>` — parses `"Item Name x3"` or `"Item Name"` into (name, quantity)

## Testing

Run all parser tests:

```bash
cd src-tauri
cargo test player_event_parser
```

Tests cover: individual event parsing, encoded value decoding, pending delete resolution (storage, vendor, standalone), instance registry population, stack delta calculation, and flush behavior. See the `#[cfg(test)] mod tests` block in the source.
