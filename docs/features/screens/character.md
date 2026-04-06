# Character Screen

## Overview

The character screen is a 7-tab hub for all character-specific data: skill progression, character report stats, NPC relationships, quest tracking, gourmand progress, Statehelm reputation tracking, and build planning. Data comes from a mix of live session tracking, persisted game state, character report imports, and CDN enrichment.

## Architecture

### Files

**Frontend (Vue/TS):**
- `src/components/Character/CharacterView.vue` — 7-tab container
- `src/components/Character/SkillsScreen.vue` — skills tab (two-panel + tracked skills)
- `src/components/Character/NpcsScreen.vue` — NPCs tab (two-panel favor/services)
- `src/components/Character/QuestsScreen.vue` — quests tab (two-panel with eligibility)
- Stats, Gourmand, and Build Planner tabs are rendered inline in CharacterView

**Stores:**
- `characterStore` — character report import, snapshot management, recipe/NPC data
- `gameStateStore` — persisted skills, favor, active skills, session tracking
- `gameDataStore` — CDN enrichment (skill details, NPC data, quest data)

### Component Hierarchy

```
CharacterView.vue                   — 7-tab container
├── SkillsScreen.vue                — unified skill view
│   ├── TrackedSkillsBar.vue        — pinned skill cards at top
│   │   └── TrackedSkillCard.vue    — individual tracked skill
│   ├── SkillListPanel.vue          — filterable/sortable skill list (left)
│   └── SkillDetailPanel.vue        — selected skill detail (right)
├── Stats tab (inline)              — character report snapshots
│   ├── SnapshotList.vue            — snapshot selector
│   ├── SnapshotComparison.vue      — compare two snapshots
│   ├── SkillTable.vue              — skill levels from report
│   ├── StatsTable.vue              — combat/attribute stats
│   ├── CurrencyTable.vue           — currency holdings
│   └── RecipeTable.vue             — known recipes
├── NpcsScreen.vue                  — NPC relationships
│   ├── NpcListPanel.vue            — searchable NPC list (left)
│   └── NpcDetailPanel.vue          — selected NPC detail (right)
│       ├── NpcFavorSection.vue     — favor level and progress
│       ├── NpcServicesSection.vue  — vendor/training/barter/storage
│       └── NpcPreferencesSection.vue — gift preferences
├── QuestsScreen.vue                — quest reference with eligibility
│   ├── QuestListPanel.vue          — searchable quest list (left)
│   └── QuestDetailPanel.vue        — selected quest detail (right)
│       ├── QuestRequirementsSection.vue
│       ├── QuestObjectivesSection.vue
│       └── QuestRewardsSection.vue
├── Gourmand tab                    — food tracking for Gourmand skill
├── StatehelmView.vue               — Statehelm weekly gift tracker
└── Build Planner tab               — combat build planning (stub)
```

## Per-Tab Documentation

- [character-skills.md](character/character-skills.md) — Skills: two-panel layout, tracked skills, XP progression, CDN enrichment
- [character-stats.md](character/character-stats.md) — Stats: character report import, snapshot management
- [character-npcs.md](character/character-npcs.md) — NPCs: favor progression, services, gift preferences
- [character-quests.md](character/character-quests.md) — Quests: personalized quest reference with requirement eligibility
- [character-gourmand.md](character/character-gourmand.md) — Gourmand: food tracking and progress
- [character-statehelm.md](character/character-statehelm.md) — Statehelm: weekly gift tracking and NPC services
- [character-buildplanner.md](character/character-buildplanner.md) — Build Planner (stub)

## Data Sources

| Data | Source | Persistence |
|------|--------|-------------|
| Skill levels & XP | `game_state_skills` (DB) + session tracking (memory) | DB + session |
| Active combat skills | `game_state_active_skills` (DB) | DB |
| NPC favor | `game_state_favor` (DB) | DB |
| Gift log | `game_state_gift_log` (DB) | DB |
| Character report data | `character_snapshots` (DB) | DB |
| Skill/NPC/Quest details | CDN via `gameDataStore` | CDN cache |
| Tracked skills | DB (per-character) | DB |
| Session XP rates | In-memory (gameStateStore) | Session only |

## Key Design Decisions

- **Three data source merge for skills** — persisted levels from login dumps, live session XP tracking, and CDN enrichment are merged into a unified view.
- **Two-panel layout** — Skills, NPCs, and Quests all use a list-on-left, detail-on-right pattern for efficient browsing.
- **Tracked skills** — players pin important skills for quick access as rich cards, avoiding an overwhelming grid of all skills.
- **Personalized quest eligibility** — quest requirements are checked against the player's actual skill levels and favor, showing which quests they can currently accept.
