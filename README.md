# Glogger

A desktop app for tracking various game data in **Project: Gorgon**. Built with [Tauri](https://tauri.app) (Rust backend) and Vue 3 frontend.

## What it does

Glogger tails your game's log file in real time and parses events as they happen — items gathered, speed bonuses, XP/skill updates, acquisition rates, and (eventually) session history, running averages, etc.

## Tech stack

- **Tauri 2** — Rust backend, tiny distributable (~5–10MB), uses OS webview
- **Vue 3** — frontend UI
- **Pinia** — state management
- **tokio** — async file watching in Rust

## Prerequisites

- [Git](https://git-scm.com/downloads)
- [Node.js LTS](https://nodejs.org)
- [Rust + rustup](https://rustup.rs) + Visual Studio C++ build tools (Windows)
- VS Code with extensions: Tauri, Rust Analyzer, Vue - Official

## Getting started

```bash
git clone https://github.com/YOUR_USERNAME/glogger.git
cd glogger
npm install
npm run tauri dev
```

## Project structure

```
glogger/
├── src/              # Vue frontend
├── src-tauri/
│   ├── src/
│   │   ├── main.rs   # Entry point 
│   │   └── lib.rs    # Tauri commands, file watcher, log parsers
│   ├── capabilities/
│   │   └── default.json  # Permission config
│   └── Cargo.toml
├── package.json
└── docs/
    └── sample.log    # Test data
```

## Log events parsed

| Log pattern | Data extracted |
|---|---|
| `ProcessDoDelayLoop` | Survey start (map name) |
| `ProcessScreenText` | Item collected, speed bonus, quantity |
| `ProcessUpdateSkill` | Skill type, level, XP, TNL |

## Status

Early development — currently proving the Rust → Vue event pipeline and working on basic log parsing.