# Devtools Capture Analysis Results

*Analysis of 3 captures from glogger v0.6.5, character Zenith on Dreva*

Captures analyzed:
- `gardening-almanac-01.json` — 3944 lines, ~81 seconds, gardening + almanac reading + area transition
- `npc-milking.json` — 486 lines, ~30 seconds, milking 3 NPC cows
- `reports-and-slash-commands.json` — 1849 lines, ~3 minutes, triggering various `/report` slash commands

Reference docs from `E:\smallProjects\GorgonLogViewer\docs\log-parsing\` were used to validate findings and fill in argument formats.

---

## High-Value Feature Opportunities

### 1. Gardening Tracker (via ProcessUpdateDescription)

**Not currently parsed.** `ProcessUpdateDescription` fires whenever a nearby entity changes state. In the gardening context, it tracks the full lifecycle of garden plants in real-time:

```
ProcessUpdateDescription(entityId, "name", "description", "action", actionType, "appearance", flags)
```

Plant state machine observed:
- **Thirsty** — `"This barley needs water!"`, action `"Water Barley"`, Scale=0.6
- **Hungry** — `"This bluebell needs fertilizer!"`, action `"Fertilize Bluebell"`, Scale=0.8
- **Growing** — `"This barley is growing nicely."`, action `"Check Barley"`, Scale=0.7-0.9
- **Ripe/Blooming** — `"This potato is fully grown and in peak condition."`, action `"Harvest Potato"`, Scale=0.85-1.0

The entity ID persists across state changes, so a garden plot can be tracked from planting through harvest. Crops observed: Barley, Carrot, Broccoli, Beet, Potato, Tundra Rye, Bluebell, Pansy, Winterhue, Dahlia.

**What this enables:** A real-time gardening dashboard showing which plants need water, which need fertilizer, which are growing, and which are ready to harvest. Combined with inventory data (seeds, fertilizer, water in inventory), this could be a full gardening assistant.

**Unblocks:** The existing TODO items for "Gardening helper" and "Garden almanac widget" — we now have confirmation that garden state IS available from the log.

### 2. Garden Almanac Parsing (via ProcessBook)

**ProcessBook is already parsed** and emits `BookOpened` events, but only `PlayerShopLog` book type is handled in the coordinator. The gardening almanac produces a `ProcessBook` with `book_type = "GardeningAlmanac"` containing structured HTML-like content:

- Current bonus event: crop name, zone, "extra yield", time remaining
- Upcoming events: crop name, zone, start time

Example content: `"Paleblood Season in Ilmari"` with extra yield, ends in 12h43m.

**What this enables:** A gardening almanac widget showing current and upcoming crop bonuses. The TODO already has this item — the blocker was "need someone to read the almanac in-game and check what the ProcessBook line looks like." The capture answers this question: it's a `GardeningAlmanac` book type with parseable HTML content.

### 3. Moon Phase from Log (via ProcessSetCelestialInfo)

**Not currently parsed.** `ProcessSetCelestialInfo(WaxingCrescentMoon)` fires on area load with the server's authoritative moon phase. Reference docs confirm the known values include `WaningCrescentMoon`, `FullMoon`, etc.

**What this enables:** Validation or replacement of the Meeus algorithm moon phase calculation. The server sends the current phase on area load, so glogger could use this as ground truth rather than computing it.

### 4. Character Statistics Dashboard (via ProcessBook reports)

The `/report` slash commands produce `ProcessBook` events with specific book types containing rich structured data:

| Book Type | Content |
|-----------|---------|
| `HelpScreen` (Behavior Report) | Lifetime kills by species, deaths, attacks, damage dealt/taken, time played, foods eaten (meat/dairy/eggs/fish), login count, group joins, buried corpses |
| `PlayerAge` | Creation date, time played, deaths, kills, VIP subscription days |
| `ServerStatus` | Server time, uptime, ping, area status, session time |
| `SkillReport` (Foods Consumed) | Every food item ever eaten with counts (hundreds of entries) |
| `SkillReport` (Sources of Lore XP) | Lore XP by source category (altars, books, quests, hangouts, recipes) with current/max |
| `SkillReport` (Explored Maps) | Per-area exploration percentage with fog block counts |
| `SkillReport` (Teleportation Status) | Bind locations, most-used destinations, unique spots per area |
| `SkillReport` (Mushroom Farming Status) | Boxes allowed vs active |
| `SkillReport` (Cheese Aging Status) | Casks allowed vs active |

**What this enables:** A character stats/achievements page showing lifetime statistics, exploration progress, food history, lore XP tracker. The book content is structured text that can be parsed with regex. Some of this data is not available from any other source.

### 5. Crafting Timer/Progress Tracking (via ProcessUpdateDescription)

Beyond gardening, `ProcessUpdateDescription` also fires for crafting items with timed stages. In the milking capture, a "Rising Simple Sourdough" item was tracked:

```
ProcessUpdateDescription(1601585, "Rising Simple Sourdough", "Proofing for 00:00:05", "Bake Simple Sourdough", UseItem, "Dough(Scale=0.36547)", 0)
```

The scale increases over time (0.365 -> 0.381 -> 0.396 -> ...) and the description shows a countdown timer. This could apply to any crafting process with timed stages (brewing, cheesemaking, etc.).

**What this enables:** Real-time progress tracking for timed crafting processes. This is directly relevant to the TODO item for "General-purpose timer system (mushroom barrels, brewing, cheesemaking, fletching)." While not a substitute for manual timers for offline processes, it provides live feedback while the player is near the crafting station.

### 6. NPC Milking / Transform-in-Place Detection

Milking NPCs produces a distinct pattern: `ProcessStartInteraction(cow)` followed by `ProcessUpdateItemCode` changes (EmptyBottle stack decreases, BottleOfMilk stack increases) with NO `ProcessAddItem`/`ProcessDeleteItem`. The game treats milking as a stack-size transform, not an item add/remove.

**What this enables:** If the interaction context (cow entity) is correlated with the item code changes, milking sessions could be tracked. More broadly, this pattern reveals that some gathering activities bypass the normal item event flow — the parser should be aware of this for inventory accuracy.

---

## Medium-Value Opportunities

### 7. Quest Event Parsing (ProcessLoadQuests, ProcessCompleteQuest, ProcessAddQuest, etc.)

These fire on session load and during gameplay. Reference docs confirm the full set:
- `ProcessLoadQuests(entityId, TransitionalQuestState[], Int32[], Int32[])` — full quest state on login
- `ProcessAddQuest(entityId, TransitionalQuestState)` — new quest acquired
- `ProcessCompleteQuest(entityId, questId)` — quest completed
- `ProcessUpdateQuest(entityId, TransitionalQuestState)` — quest state change
- `ProcessFailQuest(entityId, questId)` — quest failed/abandoned
- `ProcessSelectQuest(questId)` — tracked quest selection

Not currently parsed. **Directly unblocks** the TODO item for "Statehelm repeatable quest tracking" which lists quest event parsing as a blocker.

### 8. Guild Info (ProcessGuildGeneralInfo)

`ProcessGuildGeneralInfo(guildId, "GuildName", "motd")` — provides guild ID, name, and MOTD. Not currently parsed. Low effort to add, could display guild membership on a character info screen.

### 9. Player Notepad (ProcessSetString with NOTEPAD key)

The game sends the player's in-game notepad content via `ProcessSetString`. Reference docs note the known keys include `HUNTING_GROUP_TITLE`. The capture also revealed `NOTEPAD` as a key. Could be surfaced in glogger as a reference panel.

### 10. Vendor Transaction Tracking

Reference docs detail a complete vendor flow not seen in these captures but documented:
- `ProcessVendorScreen(npcId, favorLevel, currentGold, serverId, maxGold, greeting)` — open shop
- `ProcessVendorAddItem(price, InternalName(instanceId), isFromBuyback)` — sell to vendor
- `ProcessVendorUpdateAvailableGold(currentGold, serverId, maxGold)` — gold after transaction

This could power the "Shop/stall tracking" TODO item.

### 11. P2P Trade Tracking

Reference docs document the trade flow:
- `ProcessP2PStartInteraction(Trade, targetEntityId)` — trade begins
- `ProcessP2PSetTrade(targetEntityId, slotIndex, itemList, isConfirmed)` — items offered
- `ProcessP2PEndInteraction(Trade, targetEntityId, wasCompleted)` — trade completed/cancelled

Not seen in captures, but documented and could feed into economic tracking.

---

## State Snapshot Observations

Both `state_at_start` and `state_at_stop` snapshots are captured and contain comprehensive data:
- **800 attributes** (health, power, armor, metabolism, mount stats, NPC modifier stats, work order stats)
- **Full inventory** with instance IDs, type IDs, stack sizes
- **All skills** with levels, XP, TNL
- **386 favor entries** with NPC tiers
- **14 currencies** (including exact council count)
- **Active effects** with IDs and source entities
- **World state** (area, combat status)

The start/stop snapshot diff could be used to detect state changes that occurred during the capture window, particularly for inventory and currency changes that might not have corresponding log events.

---

## Capture Format Issues & Noise Analysis

### Line Truncation Bug
~3-5% of captured lines are truncated fragments (e.g. `"nfo)"`, `"se"`, `"o)"`). These are tail ends of lines split across read-buffer boundaries. The capture's `push_line` receives partial lines from the log file reader.

### Noise Breakdown (from gardening-almanac-01.json, representative)

| Category | Lines | % | Filter in Normal mode? |
|----------|-------|---|----------------------|
| Download appearance loop | 1912 | 48.5% | Yes |
| Asset loading (LoadAssetAsync / IsDoneLoading / Completed) | 888 | 22.5% | Yes |
| ProcessMusicPerformance (opaque arg) | 75 | 1.9% | Yes |
| OnAttackHitMe from other players | 94 | 2.4% | Yes (keep own hits) |
| Ref-count cleanup / unload cycles | 60 | 1.5% | Yes |
| Sound playing | 52 | 1.3% | Yes |
| Animation warnings (Animator.GotoState, NavMesh) | ~30 | 0.8% | Yes |
| ClearCursor | ~13 | 0.3% | Yes |
| Unity collider warnings | ~9 | 0.2% | Yes |
| **Useful Process events** | ~77 | **2.0%** | **No** |
| **Chat lines** | 3-29 | **<1%** | **No** |

In Normal mode, filtering the "Yes" categories would reduce file size by ~75-80% while retaining all meaningful gameplay data.

### Recommended Normal-Mode Filter Patterns

Based on both capture analysis and reference docs (`E:\smallProjects\GorgonLogViewer\docs\log-parsing\engine.md`):

```
# Exact prefix matches (high-confidence noise)
"Download appearance loop"
"LoadAssetAsync"
"IsDoneLoading"
"Successfully downloaded Texture"
"Cannot remove: entity doesn't have particle"
"Ref-count cleanup"
"ClearCursor"
"Animator.GotoState"
"BoxColliders created at Runtime"
"Combined Static Meshes"
"Either create the Box Collider"
"MecanimEx:"
"Told to do animation"

# Contains matches
"ProcessMusicPerformance(MusicPerformanceManager+PerformanceInfo)"
"Playing sound"
": Playing "  (sound events)

# Regex patterns
/^\d+\.\d+: Playing sound/  (timestamped sound events)
/^Shader/  (shader warnings)
```

---

## Cross-Capture Summary: All ProcessXxx Types

### Currently Parsed (seen in captures)
ProcessAddItem, ProcessCombatModeStatus, ProcessStartInteraction, ProcessEndInteraction, ProcessUpdateItemCode, ProcessSetAttributes, ProcessAddEffects, ProcessRemoveEffects, ProcessUpdateEffectName, ProcessDoDelayLoop, ProcessBook, ProcessSetWeather, ProcessLoadSkills, ProcessLoadAbilities, ProcessLoadRecipes, ProcessSetEquippedItems

### Not Parsed — High Value
| Type | Feature Value | Reference |
|------|---------------|-----------|
| ProcessUpdateDescription | **High** — gardening + crafting timers | Seen in all 3 captures |
| ProcessSetCelestialInfo | **Medium** — moon phase | Seen in gardening capture, documented in reference |
| ProcessLoadQuests | **Medium** — quest tracking blocker | Seen in gardening capture, documented in reference |
| ProcessCompleteQuest | **Medium** — quest completion | Documented in reference, not in captures |
| ProcessAddQuest | **Medium** — quest acquisition | Documented in reference |
| ProcessUpdateQuest | **Medium** — quest progress | Documented in reference |

### Not Parsed — Medium Value
| Type | Feature Value | Reference |
|------|---------------|-----------|
| ProcessSetString (NOTEPAD) | Low-Medium — notepad sync | Seen in gardening capture |
| ProcessGuildGeneralInfo | Low — guild display | Seen in gardening capture |
| ProcessVendorScreen / AddItem / UpdateGold | Medium — vendor tracking | Documented in reference |
| ProcessP2PSetTrade / EndInteraction | Medium — trade tracking | Documented in reference |
| ProcessCompleteDirectedGoals | Low-Medium — achievements | Seen in gardening capture |
| ProcessEmote | Low — event/arena | Seen in reports capture |
| ProcessMapFx | Low-Medium — survey/discovery markers | Documented in reference |
| ProcessDeltaFavor | Medium — favor gain tracking | Documented in reference |

### Not Parsed — No Value (noise or opaque)
ProcessMusicPerformance, ProcessSetStarredRecipes, ProcessInventoryFolderSettings, ProcessEnableInteractors, ProcessSetAreaSettings, ProcessMapFog, ProcessToolCommandResponse, ProcessSetExtendedGuiFeatures, ProcessRedemptionCount, ProcessSetDisabledEquipment
