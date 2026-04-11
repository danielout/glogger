# CDN Data Parsing Reference

How we parse Project: Gorgon CDN JSON files into typed Rust structs and persist them to SQLite.

## Architecture

Every CDN parser follows the **raw-JSON-first** pattern:

1. Deserialize the entire JSON file as `HashMap<String, serde_json::Value>` (or `HashMap<u32, Value>` for ID-keyed files)
2. Extract typed fields from each `Value` using helper functions
3. Store the original `Value` as `raw_json` on the struct — no data is ever lost
4. Persist typed fields as indexed DB columns and `raw_json` as a TEXT column

This means any CDN field is always accessible via `raw_json`, and we promote fields to typed struct members as features need them.

## File Locations

| Layer | Location |
|-------|----------|
| Rust parser | [`src-tauri/src/game_data/<type>.rs`](../../src-tauri/src/game_data/) |
| Rust struct re-exports | [`src-tauri/src/game_data/mod.rs`](../../src-tauri/src/game_data/mod.rs) |
| DB schema | [`src-tauri/src/db/migrations.rs`](../../src-tauri/src/db/migrations.rs) |
| DB persistence | [`src-tauri/src/db/cdn_persistence.rs`](../../src-tauri/src/db/cdn_persistence.rs) |
| TypeScript types | [`src/types/gameData/<type>.ts`](../../src/types/gameData/) |
| Tauri commands | [`src-tauri/src/cdn_commands.rs`](../../src-tauri/src/cdn_commands.rs) |

## How to Add a New Typed Field

When a feature needs a CDN field that's currently only in `raw_json`:

### 1. Find the CDN field name

Check [`docs/architecture/cdn-field-schemas.json`](cdn-field-schemas.json) for the exact field name, type, and coverage percentage. Field names are PascalCase in the CDN JSON (e.g., `FoodDesc`, `SkillReqs`).

### 2. Add to Rust struct

In the parser file (e.g., `src-tauri/src/game_data/items.rs`), add the field to the struct:

```rust
pub struct ItemInfo {
    // ... existing fields ...
    pub my_new_field: Option<String>,  // Use Option for non-100% coverage
    pub raw_json: Value,
}
```

### 3. Extract from Value in parse function

Use the helper functions at the bottom of each parser file:

```rust
// In the parse loop:
my_new_field: str_field(&value, "MyNewField"),
```

Available helpers (defined per parser file):
- `str_field(&value, "Key")` → `Option<String>`
- `u32_field(&value, "Key")` → `Option<u32>`
- `f32_field(&value, "Key")` → `Option<f32>`
- `bool_field(&value, "Key")` → `Option<bool>`
- `str_array_field(&value, "Key")` → `Vec<String>`
- For complex types: `value.get("Key").cloned()` → `Option<Value>`
- For typed sub-objects: define a struct (e.g., `CombatStats`, `TsysTierInfo`) and a parse function that extracts typed fields, with an `extra: Value` catch-all for the long tail of rare fields. See `abilities.rs::parse_combat_stats` and `tsys.rs::parse_tiers` for examples.

### 4. Add DB column

In [`migrations.rs`](../../src-tauri/src/db/migrations.rs), add the column to the table definition. Add an index if you'll query/filter on this field.

### 5. Update persistence

In [`cdn_persistence.rs`](../../src-tauri/src/db/cdn_persistence.rs), add the field to the INSERT statement and the `params![]` call.

### 6. Update TypeScript type

In the corresponding `src/types/gameData/<type>.ts`, add the matching field.

### 7. Verify

```bash
cd src-tauri && cargo check
npx vue-tsc --noEmit
```

## How to Identify Missing Fields

### Quick check: browse raw_json

In the app or via SQLite, inspect `raw_json` for any entry to see all available CDN fields:

```sql
SELECT raw_json FROM items WHERE id = 1234;
```

### Systematic: use the field schemas

[`docs/architecture/cdn-field-schemas.json`](cdn-field-schemas.json) contains every field, its type(s), frequency (% of entries that have it), and sample values. Compare against the typed fields in the Rust struct to find gaps.

### Re-extract schemas after CDN update

When the CDN version bumps, the schemas should be re-extracted from fresh data. The extraction process:

1. Download the latest CDN data (the app does this automatically to `{app_data_dir}/data/`)
2. Run extraction against the JSON files to produce updated schemas
3. Compare against previous schemas to identify new/removed/changed fields

## Precomputed Indices

After parsing all CDN JSON files, `load_from_cache()` in `game_data/mod.rs` builds in-memory indices for fast lookups. These are `HashMap` fields on the `GameData` struct, computed once at startup:

- **Name/ID indices**: `item_name_index`, `skill_name_index`, `ability_name_index`, etc. — map display names to IDs for entity resolution
- **Relationship indices**: `recipes_by_skill`, `recipes_producing_item`, `vendors_for_item`, etc. — precomputed joins
- **Ability families**: `ability_families` groups ability tiers by `upgrade_of` chains. `ability_to_family` provides reverse lookup
- **TSys ↔ Ability cross-reference**: `tsys_to_abilities` and `ability_to_tsys` map treasure mods to the abilities they affect and vice versa. Built using three matching strategies (attribute token overlap, icon ID matching, text name matching with prefix disambiguation). See the build planner docs for details.

## CDN JSON Key Patterns

Files use two key formats:

- **ID-keyed**: Keys like `"Item_1234"`, `"Ability_5678"` — parsed by splitting on `_` and taking the last segment as `u32`
- **String-keyed**: Keys like `"Sword"`, `"AreaCasino"` — used as-is

## Type Mapping

| CDN JSON type | Rust type | SQLite type | TypeScript type |
|---------------|-----------|-------------|-----------------|
| `string` | `Option<String>` | `TEXT` | `string \| null` |
| `integer` | `Option<u32>` | `INTEGER` | `number \| null` |
| `number` (float) | `Option<f32>` | `REAL` | `number \| null` |
| `boolean` | `Option<bool>` | `BOOLEAN` | `boolean \| null` |
| `array<string>` | `Vec<String>` | `TEXT` (JSON) | `string[]` |
| `array<integer>` | `Vec<u32>` | `TEXT` (JSON) | `number[]` |
| `array<object>` | `Vec<Value>` or `Option<Vec<Value>>` | `TEXT` (JSON) | `unknown[]` or `unknown[] \| null` |
| `object` (complex) | `Option<Value>` or typed struct | `TEXT` (JSON) | `unknown \| null` or typed interface |
| Full entry | `Value` (always present) | `TEXT NOT NULL` | `Record<string, unknown>` |

Use `Option<T>` for fields with < 100% coverage. Use bare `Vec<T>` (defaulting to empty) for arrays even if coverage is < 100%.

---

## CDN File Field Reference

Complete field inventory for all 27 CDN JSON files. Coverage indicates what percentage of entries contain the field.

### abilities.json
- **Entries:** 5,877 | **Key format:** ID (`Ability_1234`) | **Total fields:** 72
- **Parser:** [`abilities.rs`](../../src-tauri/src/game_data/abilities.rs) | **DB table:** `abilities`

| Field | Type | Coverage | Typed? |
|-------|------|----------|--------|
| Name | string | 100% | yes |
| Description | string | 100% | yes |
| IconID | integer | 100% | yes |
| Skill | string | 100% | yes |
| Level | integer | 100% | yes |
| Keywords | array\<string\> | 97.7% | yes |
| DamageType | string | 100% | yes |
| ResetTime | integer, number | 100% | yes |
| Target | string | 100% | yes |
| Animation | string | 100% | yes |
| Prerequisite | string | 62.8% | yes |
| IsHarmless | boolean | 32.2% | yes |
| SpecialInfo | string | 34% | yes |
| WorksUnderwater | boolean | 23.2% | yes |
| WorksWhileFalling | boolean | 17.4% | yes |
| PvE | object | 100% | yes (`CombatStats` struct: damage, power_cost, range, rage_cost, accuracy, attribute modifier arrays, plus `extra` catch-all) |
| PvP | object | varies | yes (`CombatStats` struct, same shape as PvE) |
| InternalName | string | 100% | yes |
| SharesResetTimerWith | string | 66.4% | raw_json |
| UpgradeOf | string | 66.2% | raw_json |
| CausesOfDeath | array\<string\> | 91.5% | raw_json |
| AttributesThatDeltaResetTime | array\<string\> | 97.9% | raw_json |
| AttributesThatDeltaPowerCost | array\<string\> | 75.4% | raw_json |
| AttributesThatModPowerCost | array\<string\> | 68.8% | raw_json |
| TargetParticle | string | 62% | raw_json |
| ItemKeywordReqs | array\<string\> | 61.3% | raw_json |
| ItemKeywordReqErrorMessage | string | 39% | raw_json |
| Projectile | string | 17.6% | raw_json |
| ConditionalKeywords | array\<object\> | 15.7% | raw_json |
| AmmoDescription | string | 10.4% | raw_json |
| AmmoKeywords | array\<object\> | 10.4% | raw_json |
| SpecialCasterRequirements | array\<object\>, object | 8.6% | raw_json |
| SelfParticle | string | 8.1% | raw_json |
| CanBeOnSidebar | boolean | 7.8% | raw_json |
| SpecialCasterRequirementsErrorMessage | string | 6.3% | raw_json |
| CombatRefreshBaseAmount | integer | 6.4% | raw_json |
| SelfPreParticle | string | 5.8% | raw_json |
| CanSuppressMonsterShout | boolean | 5.2% | raw_json |
| AmmoStickChance | integer, number | 4.5% | raw_json |
| InternalAbility | boolean | 4.2% | raw_json |
| AbilityGroup | string | 3.6% | raw_json |
| PetTypeTagReq | string | 3.7% | raw_json |
| PetTypeTagReqMax | integer | 3.7% | raw_json |
| DelayLoopTime | integer | 2.8% | raw_json |
| DelayLoopMessage | string | 2.6% | raw_json |
| WorksInCombat | boolean | 2.4% | raw_json |
| AbilityGroupName | string | 2.4% | raw_json |
| DelayLoopIsAbortedIfAttacked | boolean | 2.2% | raw_json |
| Rank | string | 2.2% | raw_json |
| EffectKeywordsIndicatingEnabled | array\<string\> | 2% | raw_json |
| AmmoConsumeChance | integer, number | 1.6% | raw_json |
| AttributesThatDeltaDelayLoopTime | array\<string\> | 1.4% | raw_json |
| ExtraKeywordsForTooltips | array\<string\> | 1.3% | raw_json |
| WorksWhileStunned | boolean | 1.1% | raw_json |
| AttributesThatDeltaWorksWhileStunned | array\<string\> | 1.1% | raw_json |
| IgnoreEffectErrors | boolean | 1% | raw_json |
| AttributesThatDeltaCritChance | array\<string\> | 0.9% | raw_json |
| AttributeThatPreventsDelayLoopAbortOnAttacked | string | 0.7% | raw_json |
| IsCosmeticPet | boolean | 0.7% | raw_json |
| DelayLoopIsOnlyUsedInCombat | boolean | 0.7% | raw_json |
| UseAbilitiesWithoutEnemyTarget (via AI) | boolean | 0.7% | raw_json |
| SpecialTargetingTypeReq | integer | 0.5% | raw_json |
| AoEIsCenteredOnCaster | boolean | 0.5% | raw_json |
| AttributesThatModAmmoConsumeChance | array\<string\> | 0.4% | raw_json |
| IsTimerResetWhenDisabling | boolean | 0.3% | raw_json |
| CanTargetUntargetableEnemies | boolean | 0.2% | raw_json |
| WorksWhileMounted | boolean | 0.2% | raw_json |
| InventoryKeywordReqs | array\<string\> | 0.2% | raw_json |
| InventoryKeywordReqErrorMessage | string | 0.2% | raw_json |
| TargetEffectKeywordReq | string | 0.2% | raw_json |
| Costs | array\<object\> | 0.1% | raw_json |
| EffectKeywordReqs | array\<string\> | 0.1% | raw_json |
| EffectKeywordReqErrorMessage | string | 0.1% | raw_json |
| TargetTypeTagReq | string | <0.1% | raw_json |

### items.json
- **Entries:** 10,714 | **Key format:** ID (`Item_1234`) | **Total fields:** 45
- **Parser:** [`items.rs`](../../src-tauri/src/game_data/items.rs) | **DB table:** `items`

| Field | Type | Coverage | Typed? |
|-------|------|----------|--------|
| Name | string | 100% | yes |
| Description | string | 100% | yes |
| IconId | integer | 100% | yes |
| Value | integer, number | 100% | yes |
| MaxStackSize | integer | 100% | yes |
| Keywords | array\<string\> | 99.9% | yes |
| InternalName | string | 100% | yes |
| EffectDescs | array\<string\> | 40.4% | yes |
| FoodDesc | string | 5.4% | yes |
| EquipSlot | string | 27.6% | yes |
| NumUses | integer | 66.9% | yes |
| SkillReqs | object | 54.9% | yes |
| Behaviors | array\<object\> | 75.1% | yes |
| BestowRecipes | array\<string\> | 6.4% | yes |
| BestowAbility | string | 5% | yes |
| BestowQuest | string | 12.4% | yes |
| BestowTitle | integer | 5.2% | yes |
| CraftPoints | integer | 23.1% | yes |
| CraftingTargetLevel | integer | 23% | yes |
| TSysProfile | string | 23% | yes |
| DroppedAppearance | string | 37.5% | raw_json |
| EquipAppearance | string | 21.5% | raw_json |
| EquipAppearance2 | string | 13.5% | raw_json |
| StockDye | string | 13.9% | raw_json |
| IsSkillReqsDefaults | boolean | 12.7% | raw_json |
| Lint_VendorNpc | string | 12.1% | raw_json |
| AllowPrefix | boolean | 10.8% | raw_json |
| IsCrafted | boolean | 10.2% | raw_json |
| AllowSuffix | boolean | 4.4% | raw_json |
| MaxCarryable | integer | 2.9% | raw_json |
| RequiredAppearance | string | 2.4% | raw_json |
| MacGuffinQuestName | string | 2.1% | raw_json |
| DynamicCraftingSummary | string | 1.8% | raw_json |
| MaxOnVendor | integer | 1% | raw_json |
| DyeColor | string | 0.9% | raw_json |
| AllowInstallInGuildHalls | boolean | 0.6% | raw_json |
| AllowInstallInHomes | boolean | 0.6% | raw_json |
| MountedAppearance | string | 0.6% | raw_json |
| AttuneOnPickup | boolean | 0.5% | raw_json |
| DestroyWhenUsedUp | boolean | 0.4% | raw_json |
| DroppedAppearanceLookup | string | 0.4% | raw_json |
| BestowLoreBook | integer | 0.2% | raw_json |
| IgnoreAlreadyKnownBestowals | boolean | 0.1% | raw_json |
| IsTemporary | boolean | <0.1% | raw_json |
| SelfDestructDuration_Hours | integer | <0.1% | raw_json |

### skills.json
- **Entries:** 182 | **Key format:** String (skill name) | **Total fields:** 29
- **Parser:** [`skills.rs`](../../src-tauri/src/game_data/skills.rs) | **DB table:** `skills`

| Field | Type | Coverage | Typed? |
|-------|------|----------|--------|
| Id | integer | 100% | yes |
| Name | string | 78% | yes |
| Description | string | 100% | yes |
| IconId | integer | 100% | yes (via Id lookup) |
| XpTable | string | 100% | yes |
| Keywords | array\<string\> | varies | yes |
| Combat | boolean | 100% | yes |
| MaxBonusLevels | integer | 100% | yes |
| Parents | array\<string\> | 63.2% | yes |
| AdvancementTable | string | varies | yes |
| GuestLevelCap | integer | 100% | yes |
| HideWhenZero | boolean | 4.4% | yes |
| AdvancementHints | object | 51.1% | yes |
| Rewards | object | 100% | yes |
| Reports | object | 5.5% | yes |
| ActiveAdvancementTable | string | 16.5% | raw_json |
| PassiveAdvancementTable | string | 18.7% | raw_json |
| IsFakeCombatSkill | boolean | 7.1% | raw_json |
| AuxCombat | boolean | 6.6% | raw_json |
| AssociatedItemKeywords | array\<string\> | 7.7% | raw_json |
| AssociatedAppearances | array\<string\> | 4.4% | raw_json |
| RecipeIngredientKeywords | array\<string\> | 3.3% | raw_json |
| IsUmbrellaSkill | boolean | 2.7% | raw_json |
| XpEarnedAttributes | array\<string\> | 23.1% | raw_json |
| ProdigyEnabledInteractionFlag | string | 19.2% | raw_json |
| TSysCompatibleCombatSkills | array\<string\> | 17% | raw_json |
| InteractionFlagLevelCaps | object | 51.1% | raw_json |
| SkillLevelDisparityApplies | boolean | 0.5% | raw_json |
| DisallowedItemKeywords | array\<string\> | 1.1% | raw_json |

### recipes.json
- **Entries:** 4,424 | **Key format:** ID (`Recipe_1234`) | **Total fields:** 42
- **Parser:** [`recipes.rs`](../../src-tauri/src/game_data/recipes.rs) | **DB table:** `recipes`

| Field | Type | Coverage | Typed? |
|-------|------|----------|--------|
| Name | string | 100% | yes |
| Description | string | 100% | yes |
| InternalName | string | 100% | yes |
| IconId | integer | 100% | yes |
| Skill | string | 100% | yes |
| SkillLevelReq | integer | 100% | yes |
| Ingredients | array\<object\> | 100% | yes (parsed) |
| ResultItems | array\<object\> | 100% | yes (parsed) |
| ProtoResultItems | array\<object\> | 41.8% | yes (parsed) |
| Keywords | array\<string\> | 68.6% | yes |
| RewardSkill | string | 100% | yes |
| RewardSkillXp | integer | 100% | yes |
| RewardSkillXpFirstTime | integer | 100% | yes |
| PrereqRecipe | string | 45.3% | yes |
| ResultEffects | array\<string\> | 59.1% | yes |
| UsageDelay | integer, number | 65.9% | yes |
| RewardSkillXpDropOffLevel | integer | 84.1% | yes |
| SortSkill | string | 8.1% | yes |
| ActionLabel | string | 60.9% | yes |
| SharesNameWithItemId | integer | varies | yes |
| UsageDelayMessage | string | 65.9% | raw_json |
| UsageAnimation | string | 66.1% | raw_json |
| UsageAnimationEnd | string | 45.3% | raw_json |
| RewardSkillXpDropOffPct | number | 84.1% | raw_json |
| RewardSkillXpDropOffRate | integer | 84.1% | raw_json |
| LoopParticle | string | 6.1% | raw_json |
| Particle | string | 5.9% | raw_json |
| ItemMenuLabel | string | 4.6% | raw_json |
| ItemMenuKeywordReq | string | 4.1% | raw_json |
| IsItemMenuKeywordReqSufficient | boolean | 2.6% | raw_json |
| MaxUses | integer | 2.1% | raw_json |
| OtherRequirements | array\<object\>, object | 1.7% | raw_json |
| DyeColor | string | 1.7% | raw_json |
| Costs | array\<object\> | 1.2% | raw_json |
| ItemMenuCategory | string | 1.2% | raw_json |
| ItemMenuCategoryLevel | integer | 1.2% | raw_json |
| ResetTimeInSeconds | integer | 1.2% | raw_json |
| SharesResetTimerWith | string | 0.5% | raw_json |
| ValidationIngredientKeywords | array\<string\> | 0.5% | raw_json |
| RewardAllowBonusXp | boolean | 0.2% | raw_json |
| NumResultItems | integer | 0.1% | raw_json |
| RequiredAttributeNonZero | string | <0.1% | raw_json |

### npcs.json
- **Entries:** 337 | **Key format:** String (NPC key) | **Total fields:** 8
- **Parser:** [`npcs.rs`](../../src-tauri/src/game_data/npcs.rs) | **DB table:** `npcs`

| Field | Type | Coverage | Typed? |
|-------|------|----------|--------|
| Name | string | 100% | yes |
| Desc | string | 100% | yes |
| AreaName | string | 100% | yes |
| AreaFriendlyName | string | 100% | yes |
| Services | array\<object\> | 87.5% | yes |
| Preferences | array\<object\> | 69.1% | yes (parsed) |
| Pos | string | 91.4% | yes |
| ItemGifts | array\<string\> | 12.8% | yes |

### effects.json
- **Entries:** 23,056 | **Key format:** ID (`Effect_1234`) | **Total fields:** 11
- **Parser:** [`effects.rs`](../../src-tauri/src/game_data/effects.rs) | **DB table:** none (in-memory only)
- **Status:** Raw pass-through (`raw: Value`)

| Field | Type | Coverage |
|-------|------|----------|
| Name | string | 100% |
| Desc | string | 100% |
| IconId | integer | 100% |
| DisplayMode | string | 100% |
| Duration | integer, string | 99.6% |
| Keywords | array\<string\> | 100% |
| AbilityKeywords | array\<string\> | 81.3% |
| StackingType | string | 7.2% |
| StackingPriority | integer | 7.5% |
| Particle | string | 4.8% |
| SpewText | string | 0.1% |

### xptables.json
- **Entries:** 53 | **Key format:** ID (`Level_1`) | **Total fields:** 2
- **Parser:** [`xp_tables.rs`](../../src-tauri/src/game_data/xp_tables.rs) | **DB table:** `xp_tables`

| Field | Type | Coverage | Typed? |
|-------|------|----------|--------|
| InternalName | string | 100% | yes |
| XpAmounts | array\<integer\> | 100% | yes |

### tsysclientinfo.json
- **Entries:** 1,946 | **Key format:** String | **Total fields:** 8
- **Parser:** [`tsys.rs`](../../src-tauri/src/game_data/tsys.rs) | **DB table:** `tsys_client_info`

| Field | Type | Coverage | Typed? |
|-------|------|----------|--------|
| InternalName | string | 100% | yes |
| Skill | string | 100% | yes |
| Slots | array\<string\> | 100% | yes |
| Tiers | object | 100% | yes (`HashMap<String, TsysTierInfo>` with typed fields: effect_descs, min_level, max_level, min_rarity, skill_level_prereq) |
| Suffix | string | 46.6% | yes |
| Prefix | string | 16.4% | yes |
| IsUnavailable | boolean | 1% | yes |
| IsHiddenFromTransmutation | boolean | 0.6% | yes |

### tsysprofiles.json
- **Entries:** 40 | **Key format:** String | **Structure:** map of arrays
- **Parser:** [`tsys.rs`](../../src-tauri/src/game_data/tsys.rs) (stored as raw `Value` on `TsysData.profiles`)
- **DB table:** none (in-memory only)

### itemuses.json
- **Entries:** 1,121 | **Key format:** String | **Total fields:** 1
- **Parser:** [`item_uses.rs`](../../src-tauri/src/game_data/item_uses.rs) | **DB table:** `item_uses`

| Field | Type | Coverage | Typed? |
|-------|------|----------|--------|
| RecipesThatUseItem | array\<integer\> | 100% | yes |

### areas.json
- **Entries:** 36 | **Key format:** String | **Total fields:** 2
- **Parser:** [`areas.rs`](../../src-tauri/src/game_data/areas.rs) | **DB table:** `areas`

| Field | Type | Coverage | Typed? |
|-------|------|----------|--------|
| FriendlyName | string | 100% | yes |
| ShortFriendlyName | string | 88.9% | yes |

### quests.json
- **Entries:** 2,969 | **Key format:** String | **Total fields:** 45
- **Parser:** [`quests.rs`](../../src-tauri/src/game_data/quests.rs) | **DB table:** `quests`
- **Status:** Raw pass-through (`raw: Value`), persisted as `raw_data TEXT`

### attributes.json
- **Entries:** 2,121 | **Key format:** String | **Total fields:** 7
- **Parser:** [`attributes.rs`](../../src-tauri/src/game_data/attributes.rs)
- **DB table:** none (in-memory only) | **Status:** Raw pass-through

| Field | Type | Coverage |
|-------|------|----------|
| DisplayRule | string | 100% |
| DisplayType | string | 100% |
| Label | string | 90.2% |
| IconIds | array\<integer\> | 87.6% |
| DefaultValue | integer, number | 68.8% |
| Tooltip | string | 2.1% |
| IsHidden | boolean | 1.2% |

### storagevaults.json
- **Entries:** 92 | **Key format:** String | **Total fields:** 15
- **Parser:** [`storage_vaults.rs`](../../src-tauri/src/game_data/storage_vaults.rs)
- **DB table:** none (in-memory only) | **Status:** Raw pass-through

### playertitles.json
- **Entries:** 677 | **Key format:** ID (`Title_1234`) | **Total fields:** 5
- **Parser:** [`player_titles.rs`](../../src-tauri/src/game_data/player_titles.rs)
- **DB table:** none (in-memory only) | **Status:** Raw pass-through

| Field | Type | Coverage |
|-------|------|----------|
| Title | string | 100% |
| Tooltip | string | 99% |
| Keywords | array\<string\> | 83.5% |
| AccountWide | boolean | 5.3% |
| SoulWide | boolean | 5% |

### ai.json
- **Entries:** 441 | **Key format:** String | **Total fields:** 12
- **Parser:** [`ai.rs`](../../src-tauri/src/game_data/ai.rs)
- **DB table:** none (in-memory only) | **Status:** Raw pass-through

### advancementtables.json
- **Entries:** 440 | **Key format:** String | **Total fields:** 250 (Level_01 through Level_250)
- **Parser:** [`advancement_tables.rs`](../../src-tauri/src/game_data/advancement_tables.rs)
- **DB table:** none (in-memory only) | **Status:** Raw pass-through

### lorebooks.json / lorebookinfo.json
- **Parser:** [`lorebooks.rs`](../../src-tauri/src/game_data/lorebooks.rs) (multi-file aggregate)
- **DB table:** none (in-memory only) | **Status:** Raw pass-through

### sources_abilities.json / sources_items.json / sources_recipes.json
- **Parser:** [`sources.rs`](../../src-tauri/src/game_data/sources.rs) (multi-file aggregate)
- **DB table:** none (in-memory only) | **Status:** Raw pass-through

### abilitykeywords.json / abilitydynamicdots.json / abilitydynamicspecialvalues.json
- **Parser:** [`ability_keywords.rs`](../../src-tauri/src/game_data/ability_keywords.rs) / [`ability_dynamic.rs`](../../src-tauri/src/game_data/ability_dynamic.rs)
- **DB table:** none (in-memory only) | **Status:** Raw pass-through

### directedgoals.json
- **Parser:** [`directed_goals.rs`](../../src-tauri/src/game_data/directed_goals.rs)
- **DB table:** none (in-memory only) | **Status:** Raw pass-through

### landmarks.json
- **Entries:** 36 | **Structure:** map of arrays
- **Parser:** [`landmarks.rs`](../../src-tauri/src/game_data/landmarks.rs)
- **DB table:** none (in-memory only) | **Status:** Raw pass-through
