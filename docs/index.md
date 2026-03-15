# Documentation Index

## [TODO.md](TODO.md)
Small tasks and notes that don't belong in a dedicated plan.

---

## Architecture

Core structure, patterns, and standards used across the app.

- [architecture-summary.md](architecture/architecture-summary.md) — High-level overview of the Rust/Tauri + Vue architecture and data flow.
- [data-architecture.md](architecture/data-architecture.md) — Technical reference for the DataIngestCoordinator, log watchers, database schema, and event flow.
- [database-schema.md](architecture/database-schema.md) — Full SQLite database schema documentation.
- [using-the-database.md](architecture/using-the-database.md) — Overview of the local SQLite database: purpose, location, and usage patterns.
- [cdn-data-parsing.md](architecture/cdn-data-parsing.md) — How CDN JSON files are deserialized, typed, and stored with raw JSON preservation.
- [cdn-field-schemas.json](architecture/cdn-field-schemas.json) — Machine-readable field inventory for all 27 CDN data types.
- [cdn-gap-analysis.json](architecture/cdn-gap-analysis.json) — Per-file comparison of CDN fields vs what our Rust parsers currently capture.
- [settings-file.md](architecture/settings-file.md) — How app configuration is stored and managed via the Rust settings system.
- [styling.md](architecture/styling.md) — CSS architecture using Tailwind v4, theme tokens, and component classes.
- [shared-components.md](architecture/shared-components.md) — Reusable entity inline/tooltip components (ItemInline, NpcInline, etc.) and color tokens.
- [implementation-checklist.md](architecture/implementation-checklist.md) — Step-by-step checklists for common dev tasks (new parsers, DB tables, CDN fields, commands).
- [working-with-data-architecture.md](architecture/working-with-data-architecture.md) — Quick guide for adding new event types and extending the data layer.
- [player-log-pattern-registration.md](architecture/player-log-pattern-registration.md) — How to register custom pattern matchers for detecting game events in Player.log.

## Features

Documentation for individual features, both built and in-progress.

- [data-browser.md](features/data-browser.md) — Component architecture for the multi-tab Data Browser (Items, Skills, Abilities, Recipes, Quests, NPCs).
- [data-browser-guide.md](features/data-browser-guide.md) — How the Data Browser works from a user perspective: search, filtering, cross-navigation.
- [inventory-import.md](features/inventory-import.md) — Inventory snapshot import: data flow, auto-polling, backend commands, and TypeScript types.
- [character-import.md](features/character-import.md) — Spec for importing character report JSON from the `/outputcharacter` command.
- [chat-parser.md](features/chat-parser.md) — Spec for the chat log parser: file format, line parsing, incremental reading.
- [chat-item-linking.md](features/chat-item-linking.md) — Spec for detecting and linking item references in chat messages to CDN data.
- [advanced-settings.md](features/advanced-settings.md) — Advanced Settings tab: log reparsing, database statistics, and diagnostics.

## Plans

In-progress or future work. Create a `plans/` folder when needed. Delete plans when no longer relevant.

_(No active plans)_

## Samples

Sample data files for development and testing (gitignored).

- `samples/CDN-full-examples/` — Complete CDN JSON snapshots.
- `samples/character-export-samples/` — Character export JSON examples.
- `samples/player-log-samples/` — Player.log excerpts for parser testing.
