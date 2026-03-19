# Player.log Event Reference

How the game client communicates game state through Player.log, and how to decode the events.

## Event Types

### ProcessAddItem — New item enters inventory

```
[HH:MM:SS] LocalPlayer: ProcessAddItem(InternalName(instanceId), slotIndex, isNew)
```

| Field | Type | Meaning |
|---|---|---|
| `InternalName` | string | CDN internal name (e.g., `MetalSlab2`, `UnrefinedSilverOre`) |
| `instanceId` | u64 | Unique instance identifier for this specific stack/item |
| `slotIndex` | i32 | Inventory slot (-1 = auto-placed) |
| `isNew` | bool | True if newly acquired (loot, craft), False if loading inventory |

**When it fires:**
- Login (all inventory items, `isNew=False`)
- Looting items from the ground or containers
- Crafting results
- Receiving items from NPCs/quests
- Item entering inventory that creates a **new stack** (item you didn't already have a stack of)

**Key behavior:** At login, every inventory item fires a ProcessAddItem with `isNew=False`. This is how we build the **instance ID → item name mapping**. Items acquired during gameplay fire with `isNew=True`.

### ProcessUpdateItemCode — Existing stack updated

```
[HH:MM:SS] LocalPlayer: ProcessUpdateItemCode(instanceId, encodedValue, fromServer)
```

| Field | Type | Meaning |
|---|---|---|
| `instanceId` | u64 | Instance identifier (same as from ProcessAddItem) |
| `encodedValue` | u32 | Packed value: see decoding below |
| `fromServer` | bool | True = authoritative server update, False = client-side (e.g., moving between storage) |

#### Decoding `encodedValue`

The second argument packs two values into a single integer:

```
encodedValue = (stackSize << 16) | itemTypeId
```

| Bits | Mask | Value |
|---|---|---|
| High 16 bits | `value >> 16` | **Stack size** (new quantity after the update) |
| Low 16 bits | `value & 0xFFFF` | **Item type ID** (maps to CDN `items.id`) |

**Example:**
```
ProcessUpdateItemCode(136937342, 1642723, True)

  1642723 >> 16    = 25       → new stack size is 25
  1642723 & 0xFFFF = 4323     → item type ID 4323 (MetalSlab3)
```

**When it fires:**
- Adding items to an existing stack (quantity increases)
- Consuming items from a stack (crafting, using consumables — quantity decreases)
- Moving items between inventory and storage (typically `fromServer=False`)

**Tracking deltas:** By remembering the previous stack size for each instance ID, the delta between old and new stack size tells you how many items were added or removed:
```
delta = newStackSize - previousStackSize
  > 0 → items gained
  < 0 → items consumed/moved
```

### ProcessDeleteItem — Item removed from inventory

```
[HH:MM:SS] LocalPlayer: ProcessDeleteItem(instanceId)
```

| Field | Type | Meaning |
|---|---|---|
| `instanceId` | u64 | Instance identifier being removed |

**When it fires:**
- Stack fully consumed (last item used)
- Item moved to storage (paired with `ProcessAddToStorageVault`)
- Item destroyed or quest-consumed
- Motherlode map consumed on successful find

**Important:** DeleteItem fires for both "real" deletion (item consumed/destroyed) and storage transfers. To distinguish, check if a `ProcessAddToStorageVault` follows immediately — if so, the item was stowed, not destroyed.

### ProcessLoadSkills — Full skill snapshot

```
[HH:MM:SS] LocalPlayer: ProcessLoadSkills({type=SkillName,raw=R,bonus=B,xp=X,tnl=T,max=M}, ...)
```

A single line containing every skill the player has, each as a comma-separated struct:

| Field | Type | Meaning |
|---|---|---|
| `type` | string | Internal skill name (e.g., `Hammer`, `Mentalism`, `Anatomy_Cats`) |
| `raw` | u32 | Base skill level (without bonus) |
| `bonus` | u32 | Bonus levels from gear/buffs |
| `xp` | u32 | Current XP within the level |
| `tnl` | i32 | XP required to reach next level (-1 = cannot level further) |
| `max` | u32 | Maximum achievable level for this skill |

**When it fires:**
- Login — appears once alongside the ProcessAddItem inventory load
- Zone changes / reloads — fires again with a fresh snapshot

**Key behavior:** Every skill the player has ever touched is included, even at level 0. Meta-skills like `Anatomy` and `Phrenology` appear with `raw=0` and a `bonus` equal to the highest sub-skill (they are virtual roll-up skills, not directly trainable). Skills at their cap have `tnl=-1` (e.g., `Compassion` at 50/50).

**Example (abbreviated):**
```
[16:00:53] LocalPlayer: ProcessLoadSkills(
  {type=Hammer,raw=70,bonus=5,xp=0,tnl=1153715,max=70},
  {type=Mentalism,raw=76,bonus=0,xp=2353127,tnl=2502977,max=80},
  {type=Gourmand,raw=49,bonus=0,xp=835,tnl=2500,max=100},
  {type=Surveying,raw=60,bonus=3,xp=0,tnl=4000,max=60},
  ...
)
```

**Effective level** = `raw + bonus`. For Hammer above: effective level 75 (70 + 5).

### ProcessAddToStorageVault — Item moved to storage

```
[HH:MM:SS] LocalPlayer: ProcessAddToStorageVault(npcId, -1, slot, InternalName(instanceId))
```

When this follows a ProcessDeleteItem with the same instanceId, the item was **moved to storage**, not consumed.

## NPC Interaction Events

### ProcessStartInteraction — Begin interacting with an entity

```
[HH:MM:SS] LocalPlayer: ProcessStartInteraction(entityId, interactionType, distance, canInteract, "NPC_Name")
```

| Field | Type | Meaning |
|---|---|---|
| `entityId` | u32 | Entity ID (NPC, player saddlebag, interactable object) |
| `interactionType` | u32 | Type of interaction: `7` = talk/vendor, `3` = saddlebag/personal storage |
| `distance` | f32 | Distance to entity when interaction started (0 for self-interactions) |
| `canInteract` | bool | Whether the interaction is valid |
| `NPC_Name` | string | Internal NPC identifier (e.g., `"NPC_Yetta"`, `"NPC_Kalaba"`), empty string for non-NPC interactions |

**When it fires:** Player clicks on or otherwise initiates interaction with an NPC or interactable entity.

**Interaction types observed:**
- `7` — NPC talk/vendor (includes storage NPCs, hired vendors)
- `3` — Saddlebag / personal storage (entityId = player entity, NPC_Name = `""`)

### ProcessWaitInteraction — Interaction delay

```
[HH:MM:SS] LocalPlayer: ProcessWaitInteraction(entityId, delay, "", "")
```

| Field | Type | Meaning |
|---|---|---|
| `entityId` | u32 | Entity being interacted with |
| `delay` | u32 | Wait time in milliseconds (e.g., `500`) |

**When it fires:** Brief server-side delay during an NPC interaction, typically before a screen transition (e.g., opening shop logs, switching vendor tabs).

### ProcessPreTalkScreen — NPC talk screen preamble

```
[HH:MM:SS] LocalPlayer: ProcessPreTalkScreen(npcId, PreTalkScreenInfo)
```

Fires before each talk/prompt screen. Appears repeatedly during a single NPC conversation as the player navigates between screens (gift, vendor, dialogue).

### ProcessTalkScreen — NPC dialogue

```
[HH:MM:SS] LocalPlayer: ProcessTalkScreen(npcId, "", "dialogue text", "", System.Int32[], System.String[], 0, Generic)
```

| Field | Type | Meaning |
|---|---|---|
| `npcId` | u32 | NPC entity ID |
| `dialogue text` | string | The NPC's spoken text |
| `Generic` | enum | Dialogue category |

**When it fires:** NPC greets the player or responds during conversation.

### ProcessPromptForItem — NPC requests an item (gift giving)

```
[HH:MM:SS] LocalPlayer: ProcessPromptForItem(npcId, "Give Gift", "dialogue", "prompt", null, System.Int32[], System.String[], -1301, "", Error, 0, ForNpc, "NPC_Name")
```

| Field | Type | Meaning |
|---|---|---|
| `npcId` | u32 | NPC entity ID |
| `"Give Gift"` | string | Action type |
| `dialogue` | string | NPC's response text (e.g., `"A gift? For me?"`, `"Oh, thanks!"`) |
| `prompt` | string | UI prompt text (`"Choose gift"` or `"Choose another gift"`) |
| `ForNpc` | enum | Indicates this is an NPC-directed gift |
| `NPC_Name` | string | Internal NPC identifier |

**When it fires:** NPC opens the gift-giving UI. Fires once initially, then again after each gift is given (with updated dialogue like `"Choose another gift"`).

### ProcessDeltaFavor — NPC favor change

```
[HH:MM:SS] LocalPlayer: ProcessDeltaFavor(npcId, "NPC_Name", delta, isGift)
```

| Field | Type | Meaning |
|---|---|---|
| `npcId` | u32 | NPC entity ID |
| `NPC_Name` | string | Internal NPC identifier |
| `delta` | f32 | Favor amount gained (e.g., `2.8476`, `1.582`) |
| `isGift` | bool | True when favor is from a gift |

**When it fires:** After giving a gift to an NPC. The delta varies per item — different gifts are worth different amounts of favor.

### ProcessSetAttributes — Player attribute update

```
[HH:MM:SS] LocalPlayer: ProcessSetAttributes(entityId, "[ATTRIBUTE], [value]")
```

| Field | Type | Meaning |
|---|---|---|
| `entityId` | u32 | Player entity ID |
| `ATTRIBUTE` | string | Attribute key (e.g., `CUR_COMMUNITY`) |
| `value` | i32 | New attribute value |

**When it fires:** Periodically during NPC interactions. Known attributes:
- `CUR_COMMUNITY` — ticks upward as the player sells items or gives gifts (social/community currency)
- `CUR_PEACEABLENESS` — changes during storage interactions and other activities

### ProcessUpdateQuest — Quest state change

```
[HH:MM:SS] LocalPlayer: ProcessUpdateQuest(entityId, TransitionalQuestState)
```

**When it fires:** A quest objective was completed or quest state changed. In the gift-giving context, this fires after a favor threshold is crossed (e.g., giving enough gifts unlocks a quest step).

### ProcessBook — Display book or log content

```
[HH:MM:SS] LocalPlayer: ProcessBook("title", "content", "bookType", "", "", False, False, False, False, False, "")
```

| Field | Type | Meaning |
|---|---|---|
| `title` | string | Book/log title (e.g., `"Yesterday's Shop Logs"`, `"Today's Shop Logs"`) |
| `content` | string | Full text content with `\n` line breaks |
| `bookType` | string | Category (e.g., `"PlayerShopLog"`) |

**When it fires:** Player opens a readable book, scroll, or log in-game. For player shops, the hired vendor NPC provides daily shop logs via this event.

**Player shop log content** includes structured entries like:
- `"Toncom bought Thin Mesh Grate at a cost of 350 per 1 = 350"` — customer purchase
- `"Zenith collected 3800 Councils from customer purchases"` — owner collecting gold
- `"Zenith added Guava x100 to shop"` — owner stocking items
- `"Zenith made Guavax100 visible in shop at a cost of 500 per 1"` — owner setting prices
- `"Zenith paid 10900 Councils to hire Mantis Attendant for another 24 hours."` — vendor hire renewal
- `"Fidge sent a note to shop owner"` — customer message
- `"Zenith removed Basic Metal Slab x55 from shop"` — owner pulling items

### ProcessScreenText — On-screen notification

```
[HH:MM:SS] LocalPlayer: ProcessScreenText(category, "message")
```

| Field | Type | Meaning |
|---|---|---|
| `category` | enum | Notification type (e.g., `ImportantInfo`, `CraftingNotice`) |
| `message` | string | Display text |

**When it fires:** Various in-game notifications. Known categories:
- `ImportantInfo` — survey distance hints, loot bonuses (e.g., `"The treasure is 342 meters from here."`, `"Malachite collected! Also found Quartz x3"`)
- `CraftingNotice` — crafting/storage results (e.g., `"Stowed 5 items across 3 storages."`)

## Storage Events

### ProcessShowStorageVault — Open a storage vault tab

```
[HH:MM:SS] LocalPlayer: ProcessShowStorageVault(npcId, vaultId, "Storage", "label", slotCount, System.Collections.Generic.List`1[Item], System.String[], "tabName", System.Int32[], System.String[], modifier)
```

| Field | Type | Meaning |
|---|---|---|
| `npcId` | u32 | NPC entity ID providing storage (or player entity for saddlebag) |
| `vaultId` | u32 | Vault identifier (e.g., `1501`–`1507`, `114` for saddlebag) |
| `"Storage"` | string | Vault type (`"Storage"` for NPC vaults, `"Saddlebag"` for mount storage) |
| `label` | string | Description (e.g., `"Access saddlebag contents here"`, empty for NPC storage) |
| `slotCount` | u32 | Total slots in this vault tab |
| `tabName` | string | Named tab (e.g., `"Gardening and Tools"`, `"Equipment and Ammunition"`, `"Potions and Alchemy Ingredients"`, `"Gems, Crystals, and Ores"`, or empty) |

**When it fires:** Player opens or switches between storage vault tabs. Each tab is a separate vault with its own ID.

### ProcessRefreshStorageVault — Storage vault contents refreshed

```
[HH:MM:SS] LocalPlayer: ProcessRefreshStorageVault(npcId, vaultId, slotCount, System.Collections.Generic.List`1[Item])
```

| Field | Type | Meaning |
|---|---|---|
| `npcId` | u32 | NPC entity ID (0 for bulk stow operations) |
| `vaultId` | u32 | Vault identifier |
| `slotCount` | i32 | Slot count (-1 during bulk operations) |

**When it fires:** After items are added to or removed from storage. During a "stow all" operation, multiple vaults refresh simultaneously (npcId=0, slotCount=-1), followed by a `ProcessScreenText(CraftingNotice, "Stowed N items across M storages.")`.

### ProcessRemoveFromStorageVault — Take item from storage

```
[HH:MM:SS] LocalPlayer: ProcessRemoveFromStorageVault(npcId, -1, instanceId, quantity)
```

| Field | Type | Meaning |
|---|---|---|
| `npcId` | u32 | NPC entity ID (or player entity for saddlebag) |
| `instanceId` | u64 | Instance ID of the item being retrieved |
| `quantity` | u32 | Number of items taken from the stack |

**When it fires:** Player takes an item from storage into inventory. Always paired with a preceding `ProcessAddItem` — the item appears in inventory, then the storage removal is confirmed.

**Key behavior:** The inverse of `ProcessAddToStorageVault`. For AddToStorage, the sequence is `DeleteItem → AddToStorageVault`. For RemoveFromStorage, it's `AddItem → RemoveFromStorageVault`.

## Vendor Events

### ProcessVendorScreen — Open vendor shop

```
[HH:MM:SS] LocalPlayer: ProcessVendorScreen(npcId, favorLevel, currentGold, serverId, maxGold, "greeting", VendorInfo[], VendorInfo[], VendorInfo[], VendorPurchaseCap[], System.Int32[], System.String[], -1601)
```

| Field | Type | Meaning |
|---|---|---|
| `npcId` | u32 | NPC entity ID |
| `favorLevel` | enum | Favor tier with this NPC (e.g., `SoulMates`, `BestFriends`) |
| `currentGold` | u32 | Gold the vendor currently has available to buy your items |
| `serverId` | u64 | Server-side identifier |
| `maxGold` | u32 | Maximum gold the vendor can hold |
| `greeting` | string | Vendor greeting text |
| `VendorInfo[]` | array | Vendor inventory tabs (buy/sell/buyback) |
| `VendorPurchaseCap[]` | array | Per-item purchase limits |

**When it fires:** Player opens the vendor/shop UI on an NPC.

**Key behavior:** `favorLevel` reflects the player's relationship tier with that NPC, which determines available inventory and prices. `currentGold` decreases as you sell items to the vendor.

### ProcessVendorAddItem — Sell item to vendor

```
[HH:MM:SS] LocalPlayer: ProcessVendorAddItem(price, InternalName(instanceId), isFromBuyback)
```

| Field | Type | Meaning |
|---|---|---|
| `price` | u32 | Sale price in gold (councils) |
| `InternalName` | string | Item internal name (e.g., `AmuletOfCrushingMitigation5`) |
| `instanceId` | u64 | Item instance ID (same as from ProcessAddItem/DeleteItem) |
| `isFromBuyback` | bool | False = selling to vendor, True = from buyback tab |

**When it fires:** Player sells an item to the vendor. Always preceded by a `ProcessDeleteItem` with the same instance ID (item leaves player inventory).

### ProcessVendorUpdateItem — Vendor stack updated

```
[HH:MM:SS] LocalPlayer: ProcessVendorUpdateItem(instanceId, encodedValue, price)
```

| Field | Type | Meaning |
|---|---|---|
| `instanceId` | u64 | Instance ID already in vendor inventory |
| `encodedValue` | u32 | Packed value, same encoding as ProcessUpdateItemCode: `(stackSize << 16) \| itemTypeId` |
| `price` | u32 | Price per unit |

**When it fires:** Selling a stackable item that the vendor already has a stack of. Instead of creating a new entry (`VendorAddItem`), the existing vendor stack is updated.

### ProcessVendorUpdateAvailableGold — Vendor gold balance change

```
[HH:MM:SS] LocalPlayer: ProcessVendorUpdateAvailableGold(currentGold, serverId, maxGold)
```

| Field | Type | Meaning |
|---|---|---|
| `currentGold` | u32 | Vendor's gold after the transaction |
| `serverId` | u64 | Server-side identifier |
| `maxGold` | u32 | Vendor's maximum gold capacity |

**When it fires:** After every vendor buy/sell transaction. `currentGold` decreases when the vendor buys from you (pays you gold), increases when you buy from them.

## Practical Patterns — NPC Interactions

### Selling Items to a Vendor

Each sold item produces a three-event sequence:

```
[16:32:25] ProcessDeleteItem(115259296)                                    ← item leaves inventory
[16:32:25] ProcessVendorAddItem(120, AmuletOfCrushingMitigation5(115259296), False)  ← vendor receives it at 120g
[16:32:25] ProcessVendorUpdateAvailableGold(14880, ..., 15000)             ← vendor gold drops by 120
```

When selling a stackable item that already exists in the vendor's inventory:

```
[16:32:27] ProcessDeleteItem(115271948)
[16:32:27] ProcessVendorUpdateItem(115249145, 200909, 7)     ← existing vendor stack updated
[16:32:27] ProcessVendorUpdateAvailableGold(14776, ..., 15000)
```

**Tracking gold earned:** The difference between consecutive `ProcessVendorUpdateAvailableGold` values gives the sale price, or read it directly from `ProcessVendorAddItem`.

### Gift-Giving to an NPC

```
[16:33:00] ProcessPromptForItem(9618, "Give Gift", ..., ForNpc, "NPC_Kalaba")  ← gift UI opens
[16:33:03] ProcessDeltaFavor(9618, "NPC_Kalaba", 2.8476, True)                ← favor gained
[16:33:03] ProcessPromptForItem(9618, "Give Gift", "Oh, thanks!", ...)         ← ready for next gift
[16:33:03] ProcessDeleteItem(114961794)                                         ← gifted item consumed
[16:33:04] ProcessDeltaFavor(9618, "NPC_Kalaba", 1.582, True)                 ← more favor
[16:33:04] ProcessUpdateQuest(1145895, TransitionalQuestState)                 ← quest threshold crossed
```

**Detection logic:**
- `ProcessPromptForItem` with `ForNpc` → gift interaction started
- `ProcessDeleteItem` during a gift prompt → item was given as a gift (not sold or consumed)
- `ProcessDeltaFavor` → favor reward for the gift
- Different items yield different favor amounts

### Player-Owned Shop (Hired Vendor)

Interacting with a hired vendor (`NPC_HiredVendor`) reveals shop management via dialogue and book events:

```
[13:25:58] ProcessStartInteraction(9506, 7, 0, False, "NPC_HiredVendor")
[13:25:59] ProcessTalkScreen(9506, "", "<b>Hi boss, what do you need?</b>\n\n---\n
    Mantis Attendant is hired by you. Time remaining: 27 hours.\n
    Councils in cash-box: 25000", ...)
[13:26:01] ProcessTalkScreen(9506, "", "<b>You collected 25000 councils.</b>\n\n---\n
    Mantis Attendant is hired by you. Time remaining: 27 hours.\n
    Councils in cash-box: 0", ...)
[13:26:04] ProcessBook("Yesterday's Shop Logs", "...", "PlayerShopLog", ...)
[13:26:07] ProcessBook("Today's Shop Logs", "...", "PlayerShopLog", ...)
```

**Key data extractable from hired vendor dialogue:**
- **Attendant type** and **time remaining** (from `TalkScreen` text: `"Mantis Attendant is hired by you. Time remaining: 27 hours."`)
- **Cash-box balance** before/after collection (`"Councils in cash-box: 25000"` → `"Councils in cash-box: 0"`)
- **Collection amount** (`"You collected 25000 councils."`)

**Key data extractable from shop logs (`ProcessBook` with `PlayerShopLog`):**
- Customer purchases: who bought what, price, and quantity
- Owner stocking/pricing/removal actions
- Gold collection history
- Vendor hire payments and renewals
- Customer messages

### Storage Vault Interaction

Interacting with a storage NPC shows vault tabs and allows item transfers:

```
[13:26:55] ProcessStartInteraction(14804, 7, 1200, True, "NPC_Qatik")
[13:26:58] ProcessShowStorageVault(14804, 1507, "Storage", "", 15, ...)        ← first tab
[13:27:03] ProcessShowStorageVault(14804, 1506, "Storage", "", 48, ...)        ← switch tab
[13:27:07] ProcessDeleteItem(136093889)                                         ← item leaves inventory
[13:27:07] ProcessAddToStorageVault(14804, -1, 40, MapleWood(136093889))       ← into storage
[13:27:07] ProcessDeleteItem(133493941)                                         ← stackable, merged
[13:27:07] ProcessRefreshStorageVault(14804, -1, 48, ...)                      ← vault refreshed
```

Retrieving items from storage is the reverse:
```
[13:28:48] ProcessAddItem(MetalSlab4(132702881), 46, True)                     ← item enters inventory
[13:28:48] ProcessRemoveFromStorageVault(14804, -1, 132702881, 11)             ← removed 11 from storage
```

**Bulk stow** (game auto-distributes items across vaults):
```
[13:27:00] ProcessDeleteItem(136202943)
[13:27:00] ProcessDeleteItem(136202764)
[13:27:00] ProcessDeleteItem(136184812)
[13:27:00] ProcessRefreshStorageVault(0, 1505, -1, ...)    ← npcId=0, slotCount=-1 during bulk
[13:27:00] ProcessRefreshStorageVault(0, 1501, -1, ...)
    ... (multiple vaults refresh)
[13:27:00] ProcessScreenText(CraftingNotice, "Stowed 5 items across 3 storages.")
```

### Saddlebag Access

The player's mount saddlebag is accessed via self-interaction (type `3`):

```
[13:27:20] ProcessStartInteraction(4938644, 3, 0, False, "")                   ← self-interaction
[13:27:20] ProcessShowStorageVault(4938644, 114, "Saddlebag", "Access saddlebag contents here", 62, ...)
[13:27:22] ProcessAddItem(DishingHammer(136195024), 59, True)                  ← take from saddlebag
[13:27:22] ProcessRemoveFromStorageVault(4938644, -1, 136195024, 1)
```

**Detection:** `ProcessStartInteraction` with type `3` and empty NPC name → saddlebag. The `ProcessShowStorageVault` will have type `"Saddlebag"` instead of `"Storage"`.

## Instance ID → Item Identity Mapping

Instance IDs are arbitrary per-session numbers. They do **not** correspond to CDN item IDs. To know what item an instance ID refers to, you must either:

1. **Catch the ProcessAddItem at login** — every inventory item is enumerated with `InternalName(instanceId)` when the player logs in
2. **Read the low 16 bits of ProcessUpdateItemCode** — the `itemTypeId` embedded in the encoded value maps to CDN `items.id`

Both approaches should be used together. The AddItem path gives you the internal name mapping; the UpdateItemCode path gives you the numeric type ID and stack size.

## Practical Patterns

### Motherlode Survey Lifecycle

Motherlode maps are used repeatedly to get distance hints. When the correct location is found, the map is consumed and a mining node spawns:

```
[17:37:00] ProcessDoDelayLoop(1, Unset, "Using Kur Mountains Good Metal Motherlode Map", 5305, ...)
[17:37:01] ProcessScreenText(ImportantInfo, "The treasure is 342 meters from here.")
    ↑ distance hint — map NOT consumed

[17:37:03] ProcessDoDelayLoop(1, Unset, "Using Kur Mountains Good Metal Motherlode Map", 5305, ...)
[17:37:04] ProcessDeleteItem(136969636)
[17:37:05] ProcessStartInteraction(5163814, 7, 0, False, "")
[17:37:05] ProcessDoDelayLoop(6, ChopLumber, "Mining ...", 0, AbortIfAttacked, IsInteractorDelayLoop)
[17:37:11] ProcessUpdateItemCode(136937342, 1642723, True)    ← MetalSlab3, stack now 25
[17:37:11] ProcessUpdateItemCode(136807948, 3167735, True)    ← Tungsten, stack now 48
    ↑ found! map consumed → mine node → rewards via stack updates
```

**Detection logic:**
- `ProcessDoDelayLoop` with `"Motherlode Map"` followed by `ProcessScreenText` with distance → ping (searching)
- `ProcessDoDelayLoop` with `"Motherlode Map"` followed by `ProcessDeleteItem` → found (map consumed)
- `ProcessDoDelayLoop` with `"Mining ..."` shortly after → mining the spawned node
- `ProcessUpdateItemCode` calls immediately after mining completes → rewards

### Crafting Consumption

When crafting consumes materials, you see stack decreases:

```
ProcessUpdateItemCode(109085930, 200710, True)    → AdvancedInk, stack decreased
ProcessUpdateItemCode(136144120, 1574047, True)   → TundraLichen, stack decreased
ProcessUpdateItemCode(111587763, 5247202, True)   → MetalSlab2, stack decreased
```

### Storage Transfers (Not Real Deletion)

See the detailed **Storage Vault Interaction** pattern above for full stow/retrieve/bulk sequences. The short version: `ProcessDeleteItem` followed by `ProcessAddToStorageVault` with the same instance ID is a storage move, not a consumption.

## Architecture Notes

### Core Item Tracker

Item event parsing should be a **core system**, not specific to any feature. It provides:

1. **Instance registry** — maps instance IDs to item names/type IDs, built from login AddItem events
2. **Stack tracking** — current stack size per instance, updated on every UpdateItemCode
3. **Delta events** — emits item gained/lost events with item identity and quantity

Features like surveying, crafting tracking, or loot analysis subscribe to these events rather than parsing item lines themselves.

### Limitations

- **Item type ID is 16-bit** — max value 65535. Items with IDs above this would overflow. Check CDN data to confirm all item IDs fit (they should based on the game's scale).
- **Stack overflow** — when a stack hits max (typically 99), additional items of the same type create a new stack via ProcessAddItem rather than ProcessUpdateItemCode.
- **fromServer flag** — `True` on UpdateItemCode means server-authoritative (real game event). `False` typically means client-side inventory management (storage moves). Filter on `True` for tracking real gains/losses.
- **No item metadata in UpdateItemCode** — only the type ID and stack size. For item names, durability, etc., you need the instance registry or CDN lookup.
