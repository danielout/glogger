# Character Skills Screen

## Overview

The skills screen is the primary view for understanding a character's skill progression. It combines persisted skill data (from login dumps and character reports) with live session tracking to give a unified picture of where you are and how you're progressing.

## Current State

The skills tab currently shows only live session tracking — a grid of `SkillCard` components for skills that have gained XP since the session started. If no XP has been gained, it shows "No skill updates yet." Persisted skill levels (from `ProcessLoadSkills` and character imports) are buried on the separate Stats tab as a plain table.

**Problems:**
- Opening the skills tab with no active session shows nothing useful
- No way to see all your skills at a glance with their levels
- Session tracking and persisted data are split across two tabs
- No CDN enrichment (descriptions, combat/non-combat, advancement hints, rewards)
- No way to see which skills are active (equipped on the combat bar)
- No skill icons

## Proposed Design

### Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  CharacterName · ServerName          Total Levels: 1,247  (+83)│
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ── Tracked Skills ──────────────────────────────── [+ Track]  │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐                  │
│  │ ⚔ Sword    │ │ 🛡 Shield  │ │ ⚗ Alchemy  │                  │
│  │ Lv 70 (+5) │ │ Lv 65      │ │ Lv 60      │                  │
│  │ ████████░░ │ │ ██████░░░░ │ │ ████░░░░░░ │                  │
│  │ +12,450 XP │ │ +8,200 XP  │ │ idle       │                  │
│  │ 8.2k/hr    │ │ 5.1k/hr    │ │            │                  │
│  └────────────┘ └────────────┘ └────────────┘                  │
│                                                                 │
├──────────────────────────────────┬──────────────────────────────┤
│  Filter: [________]  [Combat ▾] │                              │
│  ☐ Hide maxed   ☐ Hide zero     │  SKILL DETAIL PANEL          │
│                                  │                   [+ Track]  │
│  ► Combat Skills (12)            │  ⚔ Sword                    │
│    ⚔ Sword        ● Lv 70 (+5) │  Level 70 (+5 bonus)         │
│    🛡 Shield       ● Lv 65      │  ████████████░░░ 45,230 XP   │
│    🗡 Knife          Lv 50      │                              │
│    ...                           │  ── Session ──               │
│                                  │  XP Gained: 12,450           │
│  ► Trade Skills (8)              │  XP/Hour: 8,200              │
│    ⚗ Alchemy      ● Lv 60      │  Levels Gained: 2            │
│    🔨 Blacksmithing  Lv 45      │  Est. Next Level: ~18 min    │
│    ...                           │                              │
│                                  │  ── Info ──                  │
│  ► Other Skills (5)              │  "The art of fighting..."    │
│    ...                           │  Advancement Hints: ...      │
│                                  │  Rewards: ...                │
│              [📋 Copy] [📷 Share]│                    [⭐ Track] │
└──────────────────────────────────┴──────────────────────────────┘
```

### Summary Bar

A header bar showing character identity and aggregate skill stats:

- **Character name and server** — from active character context
- **Total Combined Skill Levels** — sum of all base skill levels
- **Total Bonus Levels** — sum of all bonus levels, shown as `(+N)`

These are fun trivia numbers that players like to compare and show off.

### Tracked Skills

A promoted section at the top of the screen showing skills the player has chosen to watch closely. These render as richer cards (similar to the current `SkillCard` style) with:

- Skill icon and name
- Current level (with bonus)
- XP progress bar toward next level
- Session stats if active (XP gained, XP/hour)
- "idle" indicator if no session XP

**Tracking behavior:**
- Players add skills to the tracked list via a button on the detail panel or a picker
- Tracked skills persist across sessions (stored in the database, per-character)
- Skills currently gaining XP in the session could auto-suggest for tracking (but don't auto-add)
- Reasonable default: no skills tracked initially — let the player curate their own list
- Reorderable via drag-and-drop or manual sort

**Why tracked skills instead of showing all cards:**
The current card grid works well for 2-3 skills but gets unwieldy with many. Tracking lets players promote the skills they care about into the rich card view while keeping the full list compact in the table below.

### Left Panel — Skill List

A filterable, sortable, grouped list of **all known skills** for the character. Data comes from `game_state_skills` (persisted from login dumps) merged with `sessionSkills` (live XP tracking).

**Data source priority:**
1. Login dump skills (`ProcessLoadSkills`) — baseline levels for all skills
2. Character report skills — imported snapshot data (may be newer/older)
3. Session tracking — overlaid XP gains, levels gained, XP/hour

**Grouping options:**
- **By type** (default) — Combat / Trade / Other, using the CDN `combat` flag and `keywords`
- **By level range** — 0-20, 21-40, 41-60, 61-80, 81+
- **None** — flat list

**Sorting options:**
- Level (descending, default)
- Name (alphabetical)
- XP gained this session (descending) — highlights active training
- Recent activity — skills with session data float to top

**Filters:**
- Text filter on skill name
- Hide maxed skills (where `tnl == -1`)
- Hide zero-level skills (level 0, never trained)

**Each row shows:**
- Skill icon (from CDN `icon_id`)
- Skill name (using `SkillInline` for tooltip/navigation)
- Level (with bonus levels shown as `+N`)
- Compact XP progress bar
- Session activity indicator (dot or glow) if XP was gained this session
- Active skill badge if this is one of the two equipped combat skills

### Right Panel — Skill Detail

Clicking a skill in the list (or a tracked skill card) opens a detail panel. This merges persisted data, session data, and CDN data into a single view.

**Sections:**

1. **Header**
   - Skill icon (large)
   - Skill name
   - Level (with bonus levels)
   - Combat / Non-Combat badge
   - Active skill badge (if equipped)
   - Track/Untrack button

2. **XP Progress**
   - Visual progress bar toward next level
   - `X / Y XP` (current toward next level)
   - "MAX" if at max level

3. **Session Stats** (only shown if this skill has session data)
   - XP Gained
   - XP / Hour
   - Levels Gained
   - Estimated time to next level

4. **Description** — from CDN `description` field

5. **Advancement Hints** — from CDN `advancement_hints`, keyed by level. Show the next relevant hint based on current level. E.g., if you're level 48, show the hint for level 50.

6. **Upcoming Rewards** — from CDN `rewards`, showing abilities/bonuses that unlock at future levels. Use `AbilityInline` for ability rewards and `SkillInline` for bonus-to-skill rewards.

7. **Related Abilities** — abilities that belong to this skill (from `getAbilitiesForSkill`). Collapsible list using `AbilityInline`.

8. **Keywords** — tag badges from CDN `keywords` array

## Sharing & Export

Players love showing off their skills. The screen provides two sharing mechanisms:

### Copy to Clipboard

A "Copy" button on the skill list (or per-group) generates a formatted text block suitable for pasting into Discord, forums, etc:

```
── Combat Skills ──
Sword          70 (+5)    Shield         65
Knife          50         Unarmed        45
Psychology     42         Mentalism      38

── Trade Skills ──
Alchemy        60         Blacksmithing  45
Cooking        55         Carpentry      30

Total Levels: 1,247 (+83 bonus)
```

Monospaced, aligned, compact. Players can copy all skills, a single group, or just their tracked skills.

### Share as Image

A "Share" button renders a styled skill card image (PNG) that can be saved or shared directly:

- Character name and server as a header
- Skill list in a visually appealing layout matching the app's dark theme
- Total levels summary
- Glogger branding/watermark (subtle)
- Options for what to include: all skills, a single group, tracked skills only, or the currently selected skill's detail view

This could use `html2canvas` or a similar library to render the existing DOM into an image. The card format makes it naturally shareable — fits well in Discord embeds, forum posts, etc.

### Empty States

- **No character data at all** — "Log in to a character or import a character report to see skills."
- **Character data exists but no session** — Show full skill list with persisted data, session stats section hidden on detail panel.
- **Session active but no login dump** — Show only session-tracked skills (current behavior as fallback).
- **No tracked skills** — Brief prompt in the tracked section: "Track skills you want to watch closely. Click ⭐ on any skill to add it here."

## Data Merging Strategy

The screen needs to unify three data sources:

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

CDN skills don't have an explicit "category" field, but we can derive groups:

- **Combat** — `combat === true` in CDN data
- **Trade** — `combat === false` and has crafting-related keywords (e.g., contains recipe-related keywords or advancement_table is set)
- **Other** — everything else (non-combat, non-trade)

This is a rough heuristic. We may want to refine grouping later or make it data-driven from keywords. Start simple.

## Active Skills

The `game_state_active_skills` table tracks the two skills currently equipped on the combat bar (from `ProcessSetActiveSkills`). The skills screen should highlight these with a badge or visual indicator so players can see at a glance which skills are actively being used.

## Implementation Notes

### New Components Needed
- `SkillsScreen.vue` — top-level screen component (summary bar + tracked skills + two-panel below)
- `TrackedSkillsBar.vue` — horizontal row of tracked skill cards
- `TrackedSkillCard.vue` — individual tracked skill card (evolved from current `SkillCard`)
- `SkillListPanel.vue` — left panel with grouped/filtered/sorted skill list
- `SkillDetailPanel.vue` — right panel detail view
- `SkillListRow.vue` — individual row in the skill list
- `SkillShareMenu.vue` — copy/share UI with format options

### Components to Reuse
- `SkillInline` — for skill name display with tooltips
- `AbilityInline` — for ability rewards
- `EntityTooltipWrapper` — for hover behavior
- `EmptyState` — for empty states

### Components to Retire
- `SkillGrid.vue` — replaced by the new screen (keep on Dashboard for live session summary)
- `SkillCard.vue` — evolves into `TrackedSkillCard` (keep on Dashboard for live session summary)
- `SkillTable.vue` on the Stats tab — skill data moves to the Skills tab

### Store Changes
- Computed that merges `game_state_skills` + `sessionSkills` into a unified list
- Skill grouping/categorization logic in `gameDataStore`
- Tracked skills list (persisted per-character)

### Backend Changes
- New table for tracked skills (character_name, skill_name, sort_order)
- Tauri commands for tracked skill CRUD

### Dependencies for Share-as-Image
- `html2canvas` or similar library for DOM-to-PNG rendering
- Could be deferred to a later pass — copy-to-clipboard is the MVP
