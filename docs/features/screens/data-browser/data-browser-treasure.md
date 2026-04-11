# Data Browser — Treasure (TSys)

## Overview

Browse treasure system (TSys) mod definitions — the crafting/transmutation power entries that define what equipment mods are available, which slots they apply to, and their tier progression.

## Search

- Text search by internal name, CDN key, skill, prefix, suffix, or slot name (debounced 250ms)
- Skill dropdown filter narrows results to mods for a specific skill
- All entries loaded on mount; search refines within them

## Detail View

- Internal name and CDN key
- Linked skill (via `SkillInline`)
- Availability flags (Unavailable, Hidden from Transmutation)
- **Naming** — prefix and suffix used on equipment
- **Equipment Slots** — which gear slots this mod can appear on
- **Tiers** — sorted by tier ID, showing typed fields: level range (`min_level`–`max_level`), min rarity, skill level prereq, and effect descriptors (`effect_descs` array)
- **Related Abilities** — abilities whose PvE/PvP attribute tokens match this mod's effect tokens, deduplicated by base ability name (highest-level version shown), displayed via `AbilityInline` with hover tooltips
- **Raw JSON**

## Data Source

- `tsysclientinfo.json` — keyed by `power_XXXXX`, parsed into `TsysClientInfo` structs
- `tsysprofiles.json` — profile definitions (available via `get_tsys_profiles` command)

## Tauri Commands

- `get_all_tsys()` — all entries sorted by internal name
- `search_tsys(query, limit?)` — text search across key, name, skill, prefix, suffix, slots
- `get_tsys_profiles()` — raw profiles JSON
- `get_abilities_for_tsys(tsys_key)` — abilities affected by this mod (via attribute token matching)
