# Dashboard

## Overview

The dashboard provides an at-a-glance view of the player's current session. It supports two modes: **Active Character** (live session data) and **All Characters on Server** (aggregate view across all characters).

## Layout

### Active Character View

- **ContextBar** — Single horizontal row showing current zone weather, combat/mount status, active effects count, and non-zero currency balances
- **Skill Cards** — Grid of `SkillCard` components showing live XP gains for the session
- **Bottom row** (two-column):
  - **TransactionLog** (left) — Scrollable list of recent inventory events from `gameStateStore.liveEventLog`, color-coded by transaction type (green=added, red=removed, yellow=stack change), uses `ItemInline` for item names
  - **PlayerNotes** (right) — Simple checklist persisted to localStorage. Add/remove/check-off items, "clear completed" button

### Aggregate View (All Characters on Server)

**AggregateView** provides a server-wide overview:
- Wealth summary (total currencies, inventory market value, grand total)
- Per-character wealth breakdown table
- Combined inventory table with per-character breakdown
- Aggregate skills table (collapsible) showing skill levels across all characters
- Search functionality for items and skills

## Components

All in `src/components/Dashboard/`:
- **DashboardView** — Main container with toggle between Active Character / Aggregate views
- **ContextBar** — Weather, combat, mount, effects, currencies
- **TransactionLog** — Recent inventory event list
- **PlayerNotes** — localStorage-backed checklist
- **AggregateView** — Server-wide aggregate stats

## Data Sources

| Data | Source |
|------|--------|
| Weather, combat, mount | `gameStateStore.world` |
| Currencies | `gameStateStore.currencies` |
| Inventory events | `gameStateStore.liveEventLog` |
| Skills | `characterStore.sessionSkillList` |
| Notes | `localStorage` (frontend-only) |
| Aggregate data | `aggregateStore` |
