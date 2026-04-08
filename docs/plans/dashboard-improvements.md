# Dashboard Improvements Plan

## Overview

The dashboard is the first thing users see. It currently has skill tracking, activity feeds, a zone/NPC card, player notes, and a context bar with currencies. There's a solid card-based layout system in place already — most of these items are about adding new cards or improving existing ones.

Update documentation when done. This will track our task list for now.

## Tasks

### Polish & Layout

- [ ] Dashboard currency card layout improvements and a more useful header card
  - `ContextBar.vue` currently shows a status row (weather, combat, mount, effects count) and a flex-wrapped row of non-zero currency balances. The currencies are plain text with no visual hierarchy — just "amount name" repeated. Could benefit from: grouped/categorized currencies, icons or color coding, a more prominent display for councils/gold, and a richer header that surfaces the most useful at-a-glance info (character name, level range, current area, session duration, etc.).
  - **Effort: Small | Impact: Low-Medium (polish)**

### New Dashboard Cards

- [ ] Show critical resources on the dashboard
  - Diamonds, amethysts, aquamarines, eternal greens, salt, fire dust — inventory data is already tracked in `gameStateStore` with full item stacks. Needs a new dashboard card that pulls specific item counts from the inventory and displays them prominently. Users should probably be able to configure which items are "critical" rather than hardcoding the list. Could start with a hardcoded list and add configuration later.
  - The inventory store already has all the data; this is primarily a UI task. The main design question is whether to use a fixed list or let users pick their tracked items.
  - **Effort: Medium | Impact: High (at-a-glance value)**

- [ ] Show latest watchword detections on the dashboard
  - Watchword matches are already stored and viewable per-rule in `WatchwordsView.vue`. The watchword system stores matches with timestamps and rule info. Needs a new dashboard card that aggregates recent matches across all rules into a live feed, similar to the existing activity feed cards (items incoming/outgoing, council changes). The activity feed pattern is well-established and could be reused here.
  - **Effort: Medium | Impact: Medium (awareness without navigating to chat)**

- [x] Moon phase tracker dashboard card
  - Implemented using real-world synodic cycle math (29.53 day period, Jan 6 2000 epoch) snapped to midnight Eastern. The `useMoonPhase` composable calculates the current phase, and a Tauri command `get_quests_by_moon_phase` queries CDN quest data for quests with matching `RequirementsToSustain` conditions. The dashboard card shows the current phase with emoji, a cycle progress bar, next phase countdown, and any active moon-gated quests. All 8 game phases are supported: NewMoon, WaxingCrescentMoon, QuarterMoon, WaxingGibbousMoon, FullMoon, WaningGibbousMoon, LastQuarterMoon, WaningCrescentMoon.
  - **Effort: Medium | Impact: Medium-High (broadly useful game mechanic)**

- [ ] Daily quest tracker dashboard card
  - Track daily quest availability/completion. No daily quest tracking exists currently. Would need investigation into: what log events fire when daily quests are accepted/completed, whether the quest system in CDN data marks quests as daily, and what the reset schedule looks like. The existing quest CDN data and `QuestInline` component provide a foundation for display.
  - **Effort: Medium | Impact: Medium (daily engagement feature)**

- [ ] Statehelm tracker summary dashboard card
  - The full Statehelm tracker already exists as a composable (`useStatehelmTracker.ts`) with weekly gift tracking per NPC, favor tiers, and real-time updates. A dashboard card would be a lightweight summary: total gifts given this week vs. max, maybe a few favorite/priority NPCs, and a link to the full tracker. Most of the backend work is done — this is primarily a UI card pulling from existing data.
  - **Effort: Small-Medium | Impact: Medium (at-a-glance NPC favor info)**

- [ ] Show rez timer on the dashboard
  - No death/resurrection tracking exists. Would need to detect the death event in `PlayerEventParser` (look for a `ProcessXxx` line that fires on death or resurrection), track the cooldown start time in game state, and display a countdown on the dashboard. The parser handles ~24 of ~60 known event types — death events may already be in the log but unhandled. Research needed into what the death/rez log lines look like.
  - **Effort: Large | Impact: Medium**

- [ ] Detect long-cooldown activations and display timers on dashboard
  - Resuscitate, opening portals, Hoplology, etc. No cooldown tracking infrastructure exists. Would need: identifying the relevant log events and adding them to `PlayerEventParser`, a cooldown registry (ability name -> duration -> start time), and dashboard timer cards. Cooldown durations likely aren't in log data and may need manual configuration or CDN lookup. This is a generalized version of the rez timer — could share infrastructure.
  - **Effort: Large | Impact: Medium-High (very useful if feasible)**

- [ ] "What should I do next" dashboard card
  - Generate suggestions — some randomized, but could also be smart: "craft 5 [highest level uncrafted item in a craft skill they know]" etc. Needs access to skill levels, recipe data, and crafting history. The stores already have skill data and recipe completions. The interesting part is the suggestion engine logic — could start simple (random tips) and get smarter over time.
  - **Effort: Large | Impact: Medium (fun, engagement feature)**

- [ ] Gardening almanac dashboard card
  - Unclear if garden data appears in Player.log. Could be user-fed. Needs research into what data is available from logs or game files. If garden events exist in logs, the `PlayerEventParser` could track planting/harvesting. If not, this might need manual entry which limits its appeal.
  - **Effort: Large (research) | Impact: Medium (if feasible)**
