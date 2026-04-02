# Character Deaths

Tracks player deaths from the `[Combat]` channel in the chat log to help identify what's killing your character so you can build mitigations.

## How It Works

The combat parser watches `[Combat]` channel messages and emits two event types:

1. **DamageOnPlayer** — non-fatal damage hits on the active player (kept in a rolling buffer of 10)
2. **PlayerDeath** — `(FATALITY!)` messages where the target is the active player

When a death is detected:

1. The killing blow details are extracted (killer name, ability, damage)
2. The current area is attached for location context
3. The death is persisted to the `character_deaths` database table
4. The recent damage buffer is saved to `death_damage_sources` for context
5. A `character-death` event is emitted to the frontend for live updates
6. The damage buffer is cleared

## Detection Pattern

Combat messages follow this format:
```
[Combat] EnemyName #EntityID: AbilityName on PlayerName! Dmg: N health, N armor. (FATALITY!)
```

The parser distinguishes player deaths from mob kills by checking whether the target matches the active character name.

## Data Stored

Each death record includes:
- **Killer name** — the enemy that dealt the killing blow
- **Killing ability** — the specific attack used (resolved against CDN for tooltips)
- **Damage type** — resolved from CDN ability data (e.g. Fire, Slashing, Crushing)
- **Health/armor damage** — damage values from the killing blow
- **Area** — where the death occurred (from the most recent area transition)
- **Timestamp** — when the death occurred
- **Damage sources** — up to 10 prior damage events leading to the death (attacker, ability, damage, crit flag)

## CDN Enrichment

When a death is recorded, the coordinator resolves the killing ability name against CDN `abilities.json` data using `GameData::resolve_ability()`. This populates the `damage_type` column so players can see what damage types are killing them and build appropriate mitigations.

## UI

The **Deaths** tab on the Character screen shows:
- Summary cards: top killers, deadliest abilities, deadliest areas, damage types
- Full death log table sorted by most recent
- Click any death row to expand and see the damage sources leading up to the killing blow
- Ability names are rendered as `AbilityInline` components with hover tooltips showing damage type, costs, keywords, and other combat details

## Key Files

- Parser: `src-tauri/src/chat_combat_parser.rs`
- Coordinator integration: `src-tauri/src/coordinator.rs` (in `process_chat_events`)
- Database: `src-tauri/src/db/death_commands.rs`, migrations v11/v13/v14 in `migrations.rs`
- Frontend store: `src/stores/deathStore.ts`
- UI: `src/components/Character/DeathsView.vue`
