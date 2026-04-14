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
- [layout-patterns.md](architecture/layout-patterns.md) — Layout system: v-show navigation, TabBar, EmptyState, PaneLayout/SidePane, pane layout patterns.
- [ux-standards.md](architecture/ux-standards.md) — UX/UI standards: desktop-first design principles, keyboard navigation, layout rules, state persistence, empty states, toasts, visual consistency.
- [ux-composables.md](architecture/ux-composables.md) — UX composables: useKeyboard (nav/hotkeys), useToast (notifications), useViewPrefs (persistent screen preferences).
- [toast-system.md](architecture/toast-system.md) — Toast notification system: store, composable, container component, types, and usage guidelines.
- [time.md](architecture/time.md) — Time & timestamp standards: UTC storage, timezone offset detection, display modes, formatting API, and rules.

## Features

Cross-cutting feature documentation not tied to a single screen.

- [chat-parser.md](features/chat-parser.md) — Chat log parser: file format, line parsing, channel exclusion, item link extraction, watch rules.
- [chat-item-linking.md](features/chat-item-linking.md) — Detecting and linking item references in chat messages to CDN data.
- [advanced-settings.md](features/advanced-settings.md) — Advanced Settings tab: log reparsing, database statistics, and diagnostics.
- [update-notifications.md](features/update-notifications.md) — Update notifications: GitHub release check, bottom bar indicator, toast alert.

*Feature docs that were specific to a single screen have been merged into the corresponding screen docs below (character import → [character-stats](features/screens/character/character-stats.md), inventory import → [inventory-snapshots](features/screens/inventory/inventory-snapshots.md), farming → [economics-farming](features/screens/economics/economics-farming.md), surveying → [economics-surveying](features/screens/economics/economics-surveying.md), storage vaults → [inventory-vaults](features/screens/inventory/inventory-vaults.md), data browser → [data-browser](features/screens/data-browser.md), dashboard → [dashboard](features/screens/dashboard.md), gourmand → [character-gourmand](features/screens/character/character-gourmand.md)).*

## Screens

Per-screen documentation organized by view.

### Dashboard
- [dashboard.md](features/screens/dashboard.md) — Dashboard screen: widget registry, 6-column grid, drag-to-reorder, settings pane.
  - [widget-status.md](features/screens/dashboard/widget-status.md) — Status widget: weather, combat/mount, effects, currencies.
  - [widget-skill-tracking.md](features/screens/dashboard/widget-skill-tracking.md) — Live Skill Tracking widget: session XP gains.
  - [widget-items-incoming.md](features/screens/dashboard/widget-items-incoming.md) — Items Incoming widget: loot, crafts, summoned.
  - [widget-items-outgoing.md](features/screens/dashboard/widget-items-outgoing.md) — Items Outgoing widget: sold, stored, consumed.
  - [widget-councils.md](features/screens/dashboard/widget-councils.md) — Councils widget: gold currency changes.
  - [widget-current-zone.md](features/screens/dashboard/widget-current-zone.md) — Current Zone widget: area + NPCs with favor.
  - [widget-favor-changes.md](features/screens/dashboard/widget-favor-changes.md) — Favor Changes widget: NPC favor deltas.
  - [widget-notes.md](features/screens/dashboard/widget-notes.md) — Notes widget: personal checklist.

  - [widget-critical-resources.md](features/screens/dashboard/widget-critical-resources.md) — Critical Resources widget: tracked item quantities.
  - [widget-statehelm-summary.md](features/screens/dashboard/widget-statehelm-summary.md) — Statehelm Gifting widget: weekly gift progress.
  - [widget-watchword-alerts.md](features/screens/dashboard/widget-watchword-alerts.md) — Watchword Alerts widget: recent match feed.
  - [widget-death-tracker.md](features/screens/dashboard/widget-death-tracker.md) — Death Tracker widget: recent deaths, rez tracking, top rezzers.

### Character
- [character.md](features/screens/character.md) — Character screen: architecture, component hierarchy, data sources.
  - [character-skills.md](features/screens/character/character-skills.md) — Skills tab: two-panel layout, tracked skills, XP progression, CDN enrichment.
  - [character-stats.md](features/screens/character/character-stats.md) — Stats tab: character report import, snapshot management.
  - [character-npcs.md](features/screens/character/character-npcs.md) — NPCs tab: favor progression, services, gift preferences.
  - [character-quests.md](features/screens/character/character-quests.md) — Quests tab: personalized quest reference with requirement eligibility.
  - [character-gourmand.md](features/screens/character/character-gourmand.md) — Gourmand tab.
  - [character-statehelm.md](features/screens/character/character-statehelm.md) — Statehelm tab: weekly gift tracker with NPC services and filters.
  - [character-buildplanner.md](features/screens/character/character-buildplanner.md) — Build Planner tab: combat build planning with mod/ability/CP management.
  - Account tab: server-wide aggregate view (wealth, inventory, skills across all characters). Uses `AggregateView.vue` from Dashboard components.

### Inventory
- [inventory.md](features/screens/inventory.md) — Inventory screen: architecture, component hierarchy, vault capacity models.
  - [inventory-live.md](features/screens/inventory/inventory-live.md) — Live Inventory tab: real-time tracking from player log.
  - [inventory-snapshots.md](features/screens/inventory/inventory-snapshots.md) — Snapshots tab: point-in-time browsing from /outputitems.
  - [inventory-vaults.md](features/screens/inventory/inventory-vaults.md) — Storage Vault Database tab: area-grouped vault browser with capacity tracking.

### Crafting
- [crafting.md](features/screens/crafting.md) — Crafting screen: architecture, component hierarchy, shared commands, design decisions.
  - [crafting-quickcalc.md](features/screens/crafting/crafting-quickcalc.md) — Quick Calculator tab.
  - [crafting-projects.md](features/screens/crafting/crafting-projects.md) — Projects tab: material breakdown, pickup list, shopping list, live crafting detection.
  - [crafting-price-helper.md](features/screens/crafting/crafting-price-helper.md) — Price Helper: integrated into Projects tab as a pricing mode with customer-provides and fee configuration.
  - [crafting-leveling.md](features/screens/crafting/crafting-leveling.md) — XP Leveling Optimizer tab.
  - [crafting-history.md](features/screens/crafting/crafting-history.md) — Crafting History tab.
  - [crafting-workorders.md](features/screens/crafting/crafting-workorders.md) — Work Orders tab.
  - [crafting-cookshelper.md](features/screens/crafting/crafting-cookshelper.md) — Cook's Helper tab.
  - [crafting-skills.md](features/screens/crafting/crafting-skills.md) — Skills tab: per-skill summaries with charts and recipe lists.
  - [crafting-dynamic-items.md](features/screens/crafting/crafting-dynamic-items.md) — Dynamic Items tab: configure which items are allowed for wildcard ingredient slots.

### Economics
- [economics.md](features/screens/economics.md) — Economics screen: architecture, component hierarchy, market/farming/surveying.
  - [economics-market.md](features/screens/economics/economics-market.md) — Market Prices tab: player-maintained price database.
  - [economics-farming.md](features/screens/economics/economics-farming.md) — Farming tab: session-based profitability tracking.
  - [economics-surveying.md](features/screens/economics/economics-surveying.md) — Surveying tab: architecture, event pipeline, database schema, shared infrastructure.
    - [economics-surveying-session.md](features/screens/economics/economics-surveying-session.md) — Session sub-tab: active session tracking with live loot/XP/profit.
    - [economics-surveying-historical.md](features/screens/economics/economics-surveying-historical.md) — Historical sub-tab: past session browser with loot breakdowns.
    - [economics-surveying-analytics.md](features/screens/economics/economics-surveying-analytics.md) — Analytics sub-tab: zone-grouped all-time speed bonus and loot stats.
  - [economics-stall-tracker.md](features/screens/economics/economics-stall-tracker.md) — Stall Tracker: shop log parsing, sales/revenue/inventory analytics from PlayerShopLog books.

### Chat Logs
- [chat.md](features/screens/chat.md) — Chat Logs screen: architecture, FTS search, item linking, shared components.
  - [chat-channels.md](features/screens/chat/chat-channels.md) — Channels tab: public/custom channel browser.
  - [chat-tells.md](features/screens/chat/chat-tells.md) — Tells tab: direct message conversations.
  - [chat-simple.md](features/screens/chat/chat-simple.md) — Party, Nearby, Guild, System tabs.
  - [chat-all.md](features/screens/chat/chat-all.md) — All Messages tab: global search with advanced filtering.
  - [chat-watchwords.md](features/screens/chat/chat-watchwords.md) — Watchwords tab: rule-based alerts and notifications.

### Data Browser
- [data-browser.md](features/screens/data-browser.md) — Data Browser overlay: architecture, layout, sidebar tabs (History/Favorites/Pinned), shared patterns, search/filter summary.
  - [data-browser-items.md](features/screens/data-browser/data-browser-items.md) — Items tab.
  - [data-browser-skills.md](features/screens/data-browser/data-browser-skills.md) — Skills tab.
  - [data-browser-abilities.md](features/screens/data-browser/data-browser-abilities.md) — Abilities tab.
  - [data-browser-recipes.md](features/screens/data-browser/data-browser-recipes.md) — Recipes tab.
  - [data-browser-quests.md](features/screens/data-browser/data-browser-quests.md) — Quests tab.
  - [data-browser-npcs.md](features/screens/data-browser/data-browser-npcs.md) — NPCs tab.
  - [data-browser-effects.md](features/screens/data-browser/data-browser-effects.md) — Effects tab.
  - [data-browser-titles.md](features/screens/data-browser/data-browser-titles.md) — Titles tab.

### Search
- [search.md](features/screens/search.md) — Search screen: quick search overlay (Ctrl+F), dedicated search page, cross-category search composable.

## Plans

- [dashboard-improvements.md](plans/dashboard-improvements.md) — Dashboard cards, layout polish, and new tracker features.
- [quick-reference-system.md](plans/quick-reference-system.md) — Reference Shelf: pinnable entity tooltips in a bottom tray (Phase 1 complete).
- [unified-event-stream.md](plans/unified-event-stream.md) — Unifying Player.log and Chat.log into a single event stream (Phase 0+1 complete, Phase 2+ remaining).
- [stall-tracker-implementation.md](plans/stall-tracker-implementation.md) — Stall Tracker build plan: phased Rust + Vue implementation guide with ASCII previews for every screen and modal.

## Samples

Sample data files for development and testing (gitignored).

- `samples/CDN-full-examples/` — Complete CDN JSON snapshots.
- `samples/character-export-samples/` — Character export JSON examples.
- `samples/player-log-samples/` — Player.log excerpts for parser testing.
