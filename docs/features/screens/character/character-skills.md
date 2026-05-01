# Character Skills Screen

## Overview

The skills screen is the primary view for understanding a character's skill progression. It combines persisted skill data (from login dumps and character reports) with live session tracking to give a unified picture of where you are and how you're progressing.

## Layout

Three-pane layout using `PaneLayout`:

```
┌─────────────────────────────────────────────────────────────────────────┐
│  CharacterName · ServerName              Total Levels: 1,247  (+83)    │
├──────────────────────┬──────────────────────────┬───────────────────────┤
│  Filter: [________]  │                          │  Tracked Skills       │
│  Group: [By Type ▾]  │  SKILL DETAIL PANEL      │  ┌─────────────────┐ │
│  Sort:  [Level   ▾]  │                          │  │ Sword           │ │
│  ☐ Hide maxed        │  ⚔ Sword                 │  │ Lv 70 (+5)      │ │
│  ☐ Hide zero         │  Level 70 (+5 bonus)     │  │ ████████░░      │ │
│                       │  [Combat] [Active]       │  │ +12,450 XP      │ │
│  ► Combat Skills (12) │  [Track]                 │  │ 8.2k/hr         │ │
│    ⚔ Sword  ● Lv 70  │                          │  └─────────────────┘ │
│    🛡 Shield   Lv 65  │  ████████████░░░ 45,230  │  ┌─────────────────┐ │
│    🗡 Knife    Lv 50  │                          │  │ Alchemy         │ │
│    ...                │  ── Session ──           │  │ Lv 60           │ │
│                       │  XP Gained: 12,450       │  │ ████░░░░░░      │ │
│  ► Non-Combat (8)     │  XP/Hour: 8,200          │  │ idle            │ │
│    ⚗ Alchemy   Lv 60 │  Levels Gained: 2        │  └─────────────────┘ │
│    ...                │  Next Level: ~18 min     │                       │
│                       │                          │  "Track skills you    │
│  ► Other (5)          │  ── Description ──       │   want to watch       │
│    ...                │  ── Bonus Levels ──      │   closely..."         │
│                       │  ── Advancement Hints ── │                       │
│                       │  ── Related Abilities ── │                       │
│                       │  ── Keywords ──          │                       │
│                       │                ── Rewards │                       │
└──────────────────────┴──────────────────────────┴───────────────────────┘
```

- **Left pane:** Filterable, grouped skill list (`SkillListPanel`)
- **Center pane:** Skill detail panel (`SkillDetailPanel`)
- **Right pane:** Tracked skills cards (`TrackedSkillsBar`, collapsed by default if none tracked)

### Summary Bar

A header bar at the top showing character identity and aggregate skill stats:

- **Character name and server** — from active character context
- **Total Combined Skill Levels** — sum of all base skill levels
- **Total Bonus Levels** — sum of all bonus levels, shown as `(+N)`

### Tracked Skills (Right Pane)

A vertical card stack in the right pane showing skills the player has chosen to watch closely. Each `TrackedSkillCard` shows:

- Skill name
- Current level (with bonus)
- XP progress bar toward next level
- Session stats if active (XP gained, XP/hour)
- "idle" indicator if no session XP

**Tracking behavior:**
- Players add/remove skills via the Track/Untrack button on the detail panel
- Tracked skills persist across sessions (stored in the database, per-character)
- Reasonable default: no skills tracked initially — let the player curate their own list
- Clicking a tracked card selects it in the detail panel

### Left Panel — Skill List

A filterable, sortable, grouped list of **all known skills** for the character. Data comes from `game_state_skills` (persisted from login dumps) merged with `sessionSkills` (live XP tracking).

**Grouping options:**
- **By type** (default) — Combat / Non-Combat / Other, using the CDN `combat` flag
- **By level range** — 81+, 61-80, 41-60, 21-40, 1-20
- **None** — flat list

Groups are collapsible with expand/collapse controls.

**Sorting options:**
- Level (descending, default)
- Name (alphabetical)
- Session XP (descending) — highlights active training

**Filters:**
- Text filter on skill name
- Hide maxed skills (where `tnl == -1`)
- Hide zero-level skills (level 0, never trained)

**Each row shows:**
- Skill name
- Level (with bonus levels via `SkillLevelDisplay`)
- Compact XP progress bar
- Session activity indicator (gold dot) if XP was gained this session
- Active skill badge (sword icon) if this is one of the two equipped combat skills

### Center Panel — Skill Detail

Clicking a skill in the list (or a tracked skill card) opens the detail panel. This merges persisted data, session data, and CDN data into a grid layout.

**Sections:**

1. **Header** — Skill name, level (with bonus), Combat/Non-Combat badge, Active badge, Track/Untrack button

2. **XP Progress** — Visual progress bar, `X / Y XP` (with TNL), or "MAX" at max level

3. **Session Stats** (only shown if session data exists) — XP Gained, XP/Hour, Levels Gained, Time to Next Level

4. **Description** — from CDN `description` field

5. **Bonus Level Sources** — Reverse lookup showing which other skills grant bonus levels to this skill. Each entry shows a checkmark (achieved) or empty box, the source skill name (via `SkillInline`), and the required level.

6. **Advancement Hints** — All hints from CDN `advancement_hints`, sorted by level, with attained levels marked with a checkmark.

7. **Rewards** — All rewards from CDN `rewards`, sorted by level, with attained rewards dimmed. Shows ability rewards (via `AbilityInline`), bonus-to-skill rewards (via `SkillInline`), and recipe rewards (via `RecipeInline`).

8. **Related Abilities** — Abilities belonging to this skill from `getAbilitiesForSkill()`, sorted by level. Each shows an `AbilityInline` and a source label (Trainer, Quest, Skill level-up, etc.).

9. **Keywords** — Tag badges from CDN `keywords` array.

## Data Merging Strategy

The screen unifies three data sources:

| Source | When Available | What It Provides |
|--------|---------------|-------------------|
| `game_state_skills` (DB) | After first login dump or character import | All skill levels, bonus levels, XP, TNL |
| `sessionSkills` (in-memory) | After gaining XP in current session | XP gained, levels gained, timestamps for rate calc |
| CDN `SkillInfo` | Always (after CDN load) | Names, descriptions, icons, combat flag, hints, rewards |

**Merge logic:**
- Start with all skills from `game_state_skills` for the active character
- For each skill, overlay session data from `sessionSkills` if present (matched by skill name)
- Enrich with CDN data via `resolveSkill()` for icons, descriptions, grouping
- Skills in `sessionSkills` that aren't in `game_state_skills` still appear (edge case: XP gain before login dump arrives)

## Skill Grouping Logic

CDN skills don't have an explicit "category" field, but we derive groups:

- **Combat** — `combat === true` in CDN data
- **Non-Combat** — `combat === false`
- **Other** — combat flag not set / unknown

## Active Skills

The `game_state_active_skills` table tracks the two skills currently equipped on the combat bar (from `ProcessSetActiveSkills`). These are highlighted with a sword badge in both the skill list and detail panel.

## Empty States

- **No character data at all** — "No skill data loaded."
- **Filters eliminate all skills** — "No skills match your filters."
- **No skill selected** — "Select a skill to inspect"
- **No tracked skills** — "Track skills you want to watch closely. Select a skill and click Track."

## Components

- `SkillsScreen.vue` — top-level screen (summary bar + three-pane layout)
- `SkillListPanel.vue` — left panel with grouped/filtered/sorted skill list
- `SkillDetailPanel.vue` — center panel detail view
- `TrackedSkillsBar.vue` — right pane, vertical card stack
- `TrackedSkillCard.vue` — individual tracked skill card

Legacy `SkillCard.vue` and `SkillGrid.vue` remain in use on the Dashboard for live session summaries.

## Not Yet Implemented

- **Copy to clipboard** — formatted text export of skill list for pasting into Discord/forums
- **Share as image** — rendered PNG skill card for sharing
- **Drag-to-reorder tracked skills** — tracked skills currently use insertion order only
