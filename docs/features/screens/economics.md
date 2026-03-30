# Economics Screen

## Overview

The economics screen consolidates tools for tracking wealth, market prices, farming profitability, and surveying analytics. Four tabs cover different aspects of the game's economy.

## Architecture

### Files

**Backend (Rust):**
- `src-tauri/src/db/market_commands.rs` — market value CRUD, import/export
- `src-tauri/src/db/farming_commands.rs` — farming session persistence
- `src-tauri/src/db/survey_commands.rs` — survey type data
- `src-tauri/src/db/player_commands_survey_events.rs` — survey event logging

**Frontend (Vue/TS):**
- `src/components/Economics/EconomicsView.vue` — 4-tab container
- `src/components/Market/MarketView.vue` — market price management
- `src/components/Economics/EconomicsFarmingView.vue` — farming session wrapper
- `src/components/Farming/` — farming session components
- `src/components/Economics/EconomicsSurveyView.vue` — surveying session wrapper
- `src/components/Surveying/` — surveying session components

**Stores:**
- `marketStore` — market value CRUD, import/export, valuation modes
- `farmingStore` — farming session lifecycle, live event tracking
- `surveyStore` — survey session lifecycle, loot/profit tracking

### Component Hierarchy

```
EconomicsView.vue                   — 4-tab container
├── MarketView.vue                  — market price management
├── EconomicsFarmingView.vue        — farming wrapper
│   ├── FarmingSessionCard.vue      — active session with live tracking
│   └── HistoricalTab.vue           — past session browser
├── EconomicsSurveyView.vue         — surveying wrapper
│   ├── SessionTab.vue              — active survey session
│   │   ├── SessionSidebar.vue      — stats, XP, economics
│   │   ├── SurveyTypeAccordion.vue — per-survey-type breakdown
│   │   └── SurveyLog.vue           — activity log
│   ├── HistoricalTab.vue           — past sessions
│   └── AnalyticsTab.vue            — aggregated analytics
└── EmptyState                      — Stall Tracker (stub)
```

## Per-Tab Documentation

- [economics-market.md](economics/economics-market.md) — Market Prices
- [economics-farming.md](economics/economics-farming.md) — Farming Sessions
- [economics-surveying.md](economics/economics-surveying.md) — Surveying Sessions
- Stall Tracker — not yet implemented

## Database Tables

- **`market_values`** — item prices keyed by (server_name, item_type_id), with timestamps
- **`farming_sessions`** — session headers (name, notes, start/end, elapsed, vendor gold)
- **`farming_session_skills`** — XP/level data per skill per session
- **`farming_session_items`** — net item quantities per session
- **`farming_session_favors`** — NPC favor deltas per session
- **`survey_types`** — CDN reference table for survey costs, XP, and recipes

## Key Design Decisions

- **Universal vs per-server prices** — market values can be scoped to a specific server or marked universal (`server="*"`), controlled by a settings toggle.
- **Valuation modes** — six options for how items are valued (highest of market/vendor, market-only, vendor-only, etc.), affecting profit calculations across farming and surveying.
- **Session-based tracking** — both farming and surveying use explicit start/pause/end session lifecycles rather than always-on tracking, to keep data meaningful.
- **Market store as shared dependency** — survey profit calculations reactively update when market prices change.
