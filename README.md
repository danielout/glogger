# Glogger

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![Latest Release](https://img.shields.io/github/v/release/danielout/glogger)](https://github.com/danielout/glogger/releases/latest)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)]()

A desktop companion app for **[Project: Gorgon](https://projectgorgon.com)**. glogger reads your game's log files in real time and gives you dashboards, analytics, and tools for tracking your gameplay.

## About

- Thanks Reyetta, Wogan, Kaeus, Fidge, TwinkleofToes, DisasterGaymer, Cakedydidooda, Lucyfeir and Deradon! Feedback and early testing was super helpful in getting Glogger to where it is now. :D
- Some things only will work with VIP! This is just a limitation of how and where I can access data. The DataBrowser works no matter what though, and it is a very handy. Nothing in here is intentionally broken because of non-vip. I just can only work with what I have.
- It operates on the Player.log, ChatLogs, and the CDN data provided by Citan for app developers. (Thanks Citan!) No shady poking in to running processes or anything like that.
- There is no server for this app to talk to; everything runs locally to try and keep it as speedy and responsive as possible. Has to download some Citan-provided data on boot and with new patches, and pokes github to check for app updates.

I apologize for bad UI/UX/programming - I'm a game designer by trade, not a software engineer. I'm more at home with excel and scripts than app development, so I'm learning as I go.

## Why beta?

- We still don't have 100% capture rate on items going in/out of inventory sometimes. It is a stubborn bug, and I'm trying to track it down. It is pretty dang good, but not 100%
- There is a lot of UI/UX inconsistencies that I just haven't rooted out yet.
- Lots of little leftover minor bugs to fix here and there.
- A few key features I want to implement before it feels whole.
- but the biggest issue is _no documentation_ so far! It very much is in a "play with it and figure it out" state, and a lot of the screens _really_ need some guidance to tell you how to use them.

## Initial Setup

Instructions available in short (<2 minutes) video form here. Featuring Cheb! https://www.youtube.com/watch?v=9BhzbM92rv4

1. Go to your VIP settings, export your character and storage JSONs. (If you've got alts, it is helpful to do them as well!)
2. Go to GUI, scroll to the bottom, and turn on the chat logging! This will give you a chat history browser and allow you to set watchwords! Also turning on combat logging doesn't do a ton, but does allow you to see death reports on what killed you. (I have no plans to try to make DPS meters or anything.)
3. Start Glogger! The setup wizard should walk you through the very fast setup with hopefully minimal issues. 

## Features

### Dashboard
Customizable widget grid with drag-to-reorder. Widgets include live skill tracking, item gain/loss feeds, currency changes, zone info with NPC favor, death tracker, watchword alerts, Words of Power capture, trip planner, Statehelm gifting progress, critical resource tracking, and personal notes.

### Character
- **Skills** — Live XP tracking with session gains, progression charts, and CDN-enriched skill details
- **Stats** — Character report import and snapshot management
- **NPCs** — Favor progression, services, and gift preferences
- **Quests** — Personalized quest reference with requirement eligibility checking
- **Gourmand** — Food tracking for the Gourmand skill
- **Statehelm** — Weekly gift tracker with NPC services and filters
- **Build Planner** — Combat build planning with mod, ability, and combat point management
- **Account** — Aggregate view across all characters on a server

### Inventory
- **Live Inventory** — Real-time tracking of items as they enter and leave your inventory
- **Snapshots** — Point-in-time inventory browsing from `/outputitems` exports
- **Storage Vaults** — Area-grouped vault browser with capacity tracking

### Crafting
- **Projects** — Multi-recipe material breakdowns with pickup lists, shopping lists, and live crafting detection
- **Price Helper** — Pricing mode with customer-provides and fee configuration
- **XP Leveling Optimizer** — Find the best recipes to level a crafting skill
- **Crafting History** — Log of everything you've crafted
- **Work Orders** — Work order tracking
- **Cook's Helper** — Cooking-specific crafting assistant
- **Brewery** — Brewing discovery journal with per-player effect mapping and ingredient matrix
- **Dynamic Items** — Configure wildcard ingredient slot preferences

### Economics
- **Market Prices** — Player-maintained price database
- **Farming** — Session-based profitability tracking
- **Surveying** — Full survey session tracker with analytics, session history, and loot attribution
- **Stall Tracker** — Shop log parsing with sales, revenue, and inventory analytics

### Chat Logs
Full-text search across all chat channels. Browse by channel, tells, party, nearby, guild, or system. Watchword alerts let you set up keyword-based notifications for trade chat and more. Item references in chat are linked to the data browser.

### Data Browser
Browse the full Project: Gorgon CDN dataset — items, skills, abilities, recipes, quests, NPCs, enemies, areas, effects, lorebooks, and titles. Hover tooltips, click-to-navigate, search, and favorites.

### Unified Search
Scryfall-inspired structured query syntax across 14 searchable categories. Quick search overlay with `Ctrl+F`.

### Trip Planner
Multi-zone route planner with teleport-aware pathfinding. Respects bind locations and known teleport abilities.

## Screenshots

*Screenshots coming soon — in the meantime, install the app and explore!*

## Download

Grab the latest release for your platform:

**[Download Latest Release](https://github.com/danielout/glogger/releases/latest)**

| Platform | Installer |
|----------|-----------|
| Windows  | `.exe` (NSIS installer) |
| macOS    | `.dmg` |
| Linux    | `.deb`, `.AppImage` |

Release builds include automatic update checking — glogger will notify you when a new version is available.

## Building from Source

### Prerequisites

- [Node.js LTS](https://nodejs.org) (22+)
- [Rust + rustup](https://rustup.rs)
- Platform-specific build tools:
  - **Windows:** Visual Studio C++ build tools
  - **macOS:** Xcode command line tools
  - **Linux:** `webkit2gtk`, `libappindicator3`, `librsvg2`, `patchelf`

### Setup

```bash
git clone https://github.com/danielout/glogger.git
cd glogger
npm install
npm run tauri dev
```

### Build a release binary

```bash
npm run tauri:build
```

### Run tests

```bash
cd src-tauri && cargo test
```

## Tech Stack

- **[Tauri 2](https://tauri.app)** — Rust backend with OS webview, ~10MB distributable
- **[Vue 3](https://vuejs.org)** + **[Pinia](https://pinia.vuejs.org)** — Frontend UI and state management
- **[Tailwind CSS 4](https://tailwindcss.com)** — Styling
- **SQLite** (via rusqlite) — Local data persistence
- **tokio** — Async log file watching and network requests

## How It Works

glogger tails Project: Gorgon's `Player.log` and `Chat.log` files in real time. A Rust event parser processes raw log lines into structured events, which flow through feature-specific coordinators into a SQLite database. The Vue frontend renders live dashboards from that data.

```
Player.log → PlayerEventParser → Feature Coordinators → SQLite → Vue Frontend
```

Game reference data (items, skills, recipes, etc.) is downloaded from the public Project: Gorgon CDN and cached locally.

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions, conventions, and how to submit a PR.

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).
