# Data Browser — Enemies (AI)

## Overview

Browse enemy AI templates from the CDN `ai.json` data. Each entry represents an enemy type with its combat strategy, mobility, and ability list.

## Search & Filter

- Text search by key name, comment, or ability name (computed, no debounce)
- Strategy filter dropdown (Melee, Ranged)
- Result count displayed

## Detail View

- Enemy name (formatted from CamelCase key)
- CDN key
- **Comment** (if present)
- **Properties** — strategy (Melee/Ranged), mobility type (Immobile/Turret), swimming, uncontrolled pet
- **Abilities** — list of ability internal names as badges
- **Raw JSON** (opt-in via settings)

## Data Source

- Backend: `src-tauri/src/game_data/ai.rs` — parses `ai.json` into `AiInfo` structs
- Commands: `get_all_enemies()`, `search_enemies(query)`, `get_enemy(key)`
- Frontend: `src/components/DataBrowser/EnemyBrowser.vue`
