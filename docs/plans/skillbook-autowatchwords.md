# Skillbook Autowatchwords

Auto-watch chat for skill books the player doesn't own, with trained-skills and future-skills modes.

## CDN Data Chain

**Item** (keyword `AbilityRecipe` + `BestowAbility` field) → **Ability** (has `Skill` field) → **Skill**

~537 skillbook items in CDN. Primary skill mapping via `SkillReqs` keys on the item (fallback to ability lookup for edge cases like `Skillbook_FoxInABox`).

Item `Name` field (e.g., "Unarmed: Headbutt 2") is what appears in chat links as `[Item: Unarmed: Headbutt 2]`.

## Determining Known Books

Character export JSON contains abilities per skill. Cross-reference `BestowAbility` field against player's ability list to find unknown books.

**Prerequisite**: Character ability storage — current import only stores skill levels, not abilities. Need new `snapshot_skill_abilities` table.

## Watch Rule Generation

One rule per skill (not per book) using `Any` mode with `ContainsItemLink` conditions. Auto-rules tagged with `source: "auto_skillbook"` to distinguish from manual rules.

Channels: Trade + Global (configurable). Notifications: sound + toast (configurable).

## UI

"Auto Skillbooks" panel in Chat > Watchwords view:
- Master toggle, mode selector (trained/future/both)
- Skill checklist for future mode
- Channel + notification preferences
- Status: "Watching for 23 Mentalism books..."
- Auto-generated rules shown with distinct icon in sidebar

Settings stored in `skillbook_autowatch` config block.

## Phases

### Phase 1: Character Ability Storage (prerequisite)
- `snapshot_skill_abilities` table migration
- Store abilities during character import
- Tauri commands for querying abilities

### Phase 2: Skillbook Index
- Build `HashMap<String, Vec<SkillbookInfo>>` keyed by skill at CDN load
- `get_skillbooks_for_skill` and `get_missing_skillbooks` commands

### Phase 3: Auto-Rule Generation
- `generate_skillbook_watch_rules` command
- `skillbook_autowatch` settings block
- Merge auto-rules into watch_rules with `source` tag

### Phase 4: UI
- Config panel in WatchwordsView
- Skill picker, auto-rule indicators, regenerate action

### Phase 5: Reactive Updates
- Re-generate on new character import or CDN update
- Toast notification on regeneration

## Key Files

- [character_commands.rs](../../src-tauri/src/db/character_commands.rs) — ability storage
- [game_data/mod.rs](../../src-tauri/src/game_data/mod.rs) — skillbook index
- [settings.rs](../../src-tauri/src/settings.rs) — autowatch config
- [WatchwordsView.vue](../../src/components/Chat/WatchwordsView.vue) — UI integration
- [watch_rules.rs](../../src-tauri/src/watch_rules.rs) — rule evaluation
