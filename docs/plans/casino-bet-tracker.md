# Casino Arena Bet Tracker

Parse arena fight announcements, bet confirmations, and outcomes. Track bet history with win/loss stats and P&L.

**Blocked on Phase 0: Log sample collection is required before implementation can begin.**

## Log Events (Unconfirmed — Needs Captures)

No arena log samples exist. Likely events based on game mechanics:
- `ProcessScreenText` — fight announcements, bet confirmations, outcomes
- `ProcessTalkScreen` — arena NPC bet placement dialogue
- `[Status]` chat — "You used N councils" (bet) / "You received N Councils" (payout) — already parsed as `CouncilsChanged`
- Area detection — `current_area == "Casino"` provides context guard

## State Machine Design

Follows [survey aggregator](../../src-tauri/src/survey/aggregator.rs) pattern:

```
Idle → FightAnnounced → BetPlaced → FightInProgress → FightResolved → PayoutSettled
```

Cross-source correlation via timestamp proximity (Player.log events + chat CouncilsChanged). Area guard limits processing to Casino zone. Pending state in-memory with timeouts.

## Data Model

- **casino_fights**: fight_id, fighters, odds, winner, timestamps
- **casino_bets**: fight_id FK, bet_on, wager, odds, outcome, payout, net_profit
- **casino_sessions**: grouping with denormalized totals (like survey_sessions)

## UI

New "Casino" tab under Economics view with sub-tabs:
- **Live**: active fight card, recent bets, running P&L
- **History**: bet history table with date/fighter/outcome filters
- **Analytics**: total P&L, win rate, P&L over time chart, streaks, per-fighter breakdown

## Phases

### Phase 0: Log Sample Collection (PREREQUISITE)
- Capture full betting cycle via debug capture in-game
- Document ProcessScreenText, ProcessTalkScreen, and chat messages
- Store in `docs/samples/player-log-samples/casino-arena/`
- **This gates everything else**

### Phase 1: Parser
- `src-tauri/src/casino/parser.rs` — parse arena-specific events
- Extend ChatStatusEvent if needed

### Phase 2: Aggregator
- `src-tauri/src/casino/aggregator.rs` — cross-source state machine
- Wire into coordinator

### Phase 3: Persistence
- DB migration, read/write functions, Tauri commands

### Phase 4: Frontend — basic UI
- CasinoView with History tab under Economics

### Phase 5: Analytics + Polish
- P&L charts, win rate, streaks, live session tracking

## Key Files

- [survey/aggregator.rs](../../src-tauri/src/survey/aggregator.rs) — primary pattern to follow
- [coordinator.rs](../../src-tauri/src/coordinator.rs) — event pipeline wiring
- [chat_status_parser.rs](../../src-tauri/src/chat_status_parser.rs) — CouncilsChanged already parsed
- [EconomicsView.vue](../../src/components/Economics/EconomicsView.vue) — where Casino tab goes
