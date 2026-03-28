# Documentation Index

## [TODO.md](TODO.md)
Small tasks and notes that don't belong in a dedicated plan.

---

## Architecture

Core structure, patterns, and standards used across the app.

- [architecture-summary.md](architecture/architecture-summary.md) — High-level overview of the Rust/Tauri + Vue architecture and data flow.
- [live-event-streams.md](architecture/live-event-streams.md) — Unified reference for all live game events: Player.log and Chat.log data flow, all event streams, how features subscribe, timestamp handling, and how to extend both parsers.
- [player-event-parser.md](architecture/player-event-parser.md) — PlayerEventParser: event types, internal state, pending delete buffer, encoded value decoding, frontend listening, and how to extend.
- [player-log-events.md](architecture/player-log-events.md) — Complete reference for all Player.log event types: item, skill, NPC, vendor, storage, and screen events with encoding formats and practical patterns.
- [game-state.md](architecture/game-state.md) — Centralized game state system: design principles, database schema, GameStateManager, frontend store API, and domain catalog.
- [cdn-data-parsing.md](architecture/cdn-data-parsing.md) — How CDN JSON files are deserialized, typed, and stored with raw JSON preservation.
- [cdn-field-schemas.json](architecture/cdn-field-schemas.json) — Machine-readable field inventory for all 27 CDN data types.
- [cdn-gap-analysis.json](architecture/cdn-gap-analysis.json) — Per-file comparison of CDN fields vs what our Rust parsers currently capture.
- [settings-file.md](architecture/settings-file.md) — How app configuration is stored and managed via the Rust settings system.
- [user-data-management.md](architecture/user-data-management.md) — Multi-character/multi-server data scoping, server auto-detection, character management, market values, and aggregate views.
- [shared-components.md](architecture/shared-components.md) — Reusable entity inline/tooltip components (ItemInline, NpcInline, etc.) and color tokens.
- [implementation-checklist.md](architecture/implementation-checklist.md) — Step-by-step checklists for common dev tasks (new parsers, DB tables, CDN fields, commands).
- [startup-sequence.md](architecture/startup-sequence.md) — Full startup sequence: backend init, frontend phases, game data loading, and readiness audit.
- [styling.md](architecture/styling.md) — CSS architecture using Tailwind v4, theme tokens, and component classes.
- [layout-patterns.md](architecture/layout-patterns.md) — Layout system: v-show navigation, TabBar, EmptyState, CollapsiblePane, pane layout patterns.
- [ux-standards.md](architecture/ux-standards.md) — UX/UI standards: keyboard navigation, layout rules, state persistence, empty states, toasts, visual consistency.
- [ux-composables.md](architecture/ux-composables.md) — UX composables: useKeyboard (nav/hotkeys), useToast (notifications), useViewPrefs (persistent screen preferences).
- [toast-system.md](architecture/toast-system.md) — Toast notification system: store, composable, container component, types, and usage guidelines.

## Features

Documentation for individual features, both built and in-progress.

- [chat-parser.md](features/chat-parser.md) — Chat log parser: file format, line parsing, channel exclusion, item link extraction, watch rules.
- [chat-item-linking.md](features/chat-item-linking.md) — Detecting and linking item references in chat messages to CDN data.
- [data-browser.md](features/data-browser.md) — Multi-tab Data Browser (Items, Skills, Abilities, Recipes, Quests, NPCs).
- [inventory-import.md](features/inventory-import.md) — Inventory snapshot import: data flow, auto-polling, backend commands.
- [character-import.md](features/character-import.md) — Character report JSON import from `/outputcharacter`.
- [gourmand-tracker.md](features/gourmand-tracker.md) — Gourmand food tracker: progress tracking, report import, food buff comparison.
- [farming-calculator.md](features/farming-calculator.md) — Farming Calculator: manual session tracking for XP, items, favor, and vendor gold.
- [crafting-helper.md](features/crafting-helper.md) — Crafting Helper: project planning, ingredient resolution, XP leveling optimizer, live craft detection, work orders.
- [storage-tracker.md](features/storage-tracker.md) — Storage Vault Database: CDN-driven vault reference, area-grouped capacity tracking, live deposit/withdrawal.
- [surveying-tracker.md](features/surveying-tracker.md) — Surveying Tracker: real-time session tracking, survey event parsing, loot/XP/cost analytics.
- [dashboard.md](features/dashboard.md) — Dashboard: context bar, skill cards, transaction log, player notes, and aggregate server view.
- [advanced-settings.md](features/advanced-settings.md) — Advanced Settings tab: log reparsing, database statistics, and diagnostics.

## Screens

Per-screen documentation for the Character view tabs.

- [character-skills.md](features/screens/character-skills.md) — Skills screen: two-panel layout, tracked skills, XP progression, CDN enrichment.
- [character-npcs.md](features/screens/character-npcs.md) — NPCs screen: favor progression, services (vendor/training/barter/storage), gift preferences.
- [character-stats.md](features/screens/character-stats.md) — Stats screen: character report import, snapshot management.
- [character-gourmand.md](features/screens/character-gourmand.md) — Gourmand screen.
- [character-quests.md](features/screens/character-quests.md) — Quests screen: personalized quest reference with requirement eligibility checking.
- [character-buildplanner.md](features/screens/character-buildplanner.md) — Build Planner screen (stub).
- [dashboard.md](features/screens/dashboard.md) — Dashboard screen.

## Plans

- [unified-event-stream.md](plans/unified-event-stream.md) — Unifying Player.log and Chat.log into a single event stream (Phase 0+1 complete, Phase 2+ remaining).

## Samples

Sample data files for development and testing (gitignored).

- `samples/CDN-full-examples/` — Complete CDN JSON snapshots.
- `samples/character-export-samples/` — Character export JSON examples.
- `samples/player-log-samples/` — Player.log excerpts for parser testing.
