# Data Browser — Abilities

## Overview

Browse abilities organized by skill, with tier progression, combat details, and flag inspection. Abilities are grouped into **families** — all tiers of the same ability (e.g., Punch 1, Punch 2, Punch 3) are shown together as a single entry.

## Data Model

### AbilityFamily

Abilities are grouped using the CDN `UpgradeOf` field. Each family has:
- `base_internal_name` — the internal name of the tier 1 ability (used as family key)
- `base_name` — display name without tier number
- `tier_ids` — ordered list of ability IDs (ascending by level)
- Shared properties from the base tier: `icon_id`, `skill`, `damage_type`
- `is_monster_ability` — true if the base tier has the `Lint_MonsterAbility` keyword

Standalone abilities (no `UpgradeOf`, nothing upgrades to them) appear as single-tier families.

Family indices are built in Rust at CDN load time (`GameData.ability_families` and `GameData.ability_to_family`).

### Tauri Commands

- `get_ability_families_for_skill(skill, include_monster?)` — returns `Vec<AbilityFamily>` sorted by base level
- `search_ability_families(query, skill?, limit?, include_monster?)` — search across families by name/description
- `get_ability_family(ability_id)` — given any tier's ID, returns its family
- `get_skills_with_ability_counts(include_monster?)` — skill list with family counts

The existing `get_abilities_for_skill` and `resolve_ability` commands still work for individual tier lookups.

## Search & Filters

- **Skill filter dropdown** — only shows skills that have abilities
- **Text search** — client-side within selected skill (matches base name), or global search across all skills (checks tier names and descriptions, debounced 250ms)
- **Monster abilities toggle** — hidden by default; when enabled, includes abilities tagged with `Lint_MonsterAbility`. Monster abilities also skip treasure mod lookups since tsys effects don't apply to them.

## Left Panel (Family List)

Each row shows:
- Tier count indicator (e.g., "3T" for 3 tiers, "1T" for standalone)
- Base ability name

## Detail View (Right Panel)

When a family is selected, all tier `AbilityInfo` objects are resolved in parallel.

### Shared Info (top)
- Icon (from base tier)
- Base ability name, skill link, damage type, tier count
- Description (from base tier)

### Combat Details
- Target, cooldown (if same across all tiers), animation

### Tier Progression Table (`AbilityTierTable` component)
- Columns shown dynamically based on which stats have data: Tier #, Level, Damage, Power, Mana, Armor, Health, Range, Cooldown, Rage
- Click a tier row to expand and see: full name, description, prerequisites, special info, and **sources** (where to train that tier)

### Other Sections
- **Flags** — extracted from base tier raw JSON (Harmless, Works Underwater, etc.)
- **Treasure Mods** — TSys mods matching the base ability (skipped for monster abilities)
- **PvE/PvP details** — from highest tier, showing full combat stats
- **Keywords** — from base tier
- **Raw JSON** — base tier (when enabled in settings)
