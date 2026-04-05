# Chat Logs Screen

## Overview

A full chat log browser that imports, indexes, and displays all in-game chat with full-text search, per-channel filtering, item link detection, and configurable watchword alerts. Chat data is parsed from the game's chat log files and stored in SQLite with FTS indexing.

## Architecture

### Files

**Backend (Rust):**
- `src-tauri/src/chat_parser.rs` — chat log file parsing
- `src-tauri/src/chat_status_parser.rs` — status channel event parsing
- `src-tauri/src/chat_commands.rs` — Tauri command handlers
- `src-tauri/src/db/chat_commands.rs` — database query layer

**Frontend (Vue/TS):**
- `src/components/Chat/ChatView.vue` — 8-tab container
- `src/components/Chat/ChatMessageList.vue` — shared paginated message renderer
- `src/components/Chat/ChatMessage.vue` — individual message display
- `src/components/Chat/MessageWithItemLinks.vue` — item link detection and rendering
- Channel views: `ChannelView`, `TellsView`, `PartyView`, `NearbyView`, `GuildView`, `SystemView`, `AllMessagesView`, `WatchwordsView`

**Stores:**
- `chatStore` — tailing state management
- `settingsStore` — watchword rule persistence

### Component Hierarchy

```
ChatView.vue                        — 8-tab container
├── ChannelView.vue                 — public/custom channels with sidebar
├── TellsView.vue                   — direct messages with conversation list
├── PartyView.vue                   — party channel
├── NearbyView.vue                  — nearby/local chat
├── GuildView.vue                   — guild chat
├── SystemView.vue                  — system/status messages
├── AllMessagesView.vue             — global search across all channels
└── WatchwordsView.vue              — rule-based filtering and alerts

Shared:
├── ChatMessageList.vue             — paginated message list (standard + bubble layouts)
└── MessageWithItemLinks.vue        — parses and links item references
```

## Per-Tab Documentation

- [chat-channels.md](chat/chat-channels.md) — Channels
- [chat-tells.md](chat/chat-tells.md) — Tells (Direct Messages)
- [chat-simple.md](chat/chat-simple.md) — Party, Nearby, Guild, System
- [chat-all.md](chat/chat-all.md) — All Messages (Global Search)
- [chat-watchwords.md](chat/chat-watchwords.md) — Watchwords (Alert Rules)

## Shared Components

### ChatMessageList

Generic message list renderer used by all tabs:
- **Standard layout** — timestamp, channel badge (optional), sender name, message body
- **Tell/bubble layout** — chat bubbles with player messages on right, others on left
- **Pagination** — infinite scroll (auto-loads at 200px from bottom), with fallback "Load More" button. 100 messages per page. Loading indicator is inline, preserving scroll position.
- **Timestamps** — short format for today, full format for older messages

### MessageWithItemLinks

Parses message text to detect `[Item: ItemName]` patterns and renders them as `ItemInline` components with hover tooltips and click-to-navigate behavior.

## Database Schema

| Table | Purpose |
|-------|---------|
| `chat_messages` | Core message storage (timestamp, channel, sender, message, flags) |
| `chat_item_links` | Item references found in messages (raw_text, item_name, item_id) |
| `chat_messages_fts` | Full-text search index on message content |

## Tauri Commands

### Import & Tailing
- `scan_chat_logs(path) → ScanResult` — bulk import all chat logs from directory
- `scan_chat_log_file(path) → ScanResult` — import single file
- `tail_chat_log(chat_log_file) → Vec<ChatMessage>` — continuous import of active log

### Query
- `get_chat_messages(ChatFilter) → Vec<ChatMessage>` — filtered message query
- `get_chat_channels() → Vec<String>` — list all channels
- `get_chat_channel_stats() → Vec<ChannelStat>` — per-channel message counts
- `get_tell_conversations() → Vec<ChannelStat>` — list conversation partners
- `get_watch_rule_messages(rule_id, limit, offset) → Vec<ChatMessage>` — messages matching a watchword rule
- `get_chat_stats() → ChatStats` — overall statistics

### Maintenance
- `purge_chat_messages(days) → usize` — delete messages older than N days
- `delete_all_chat_messages() → usize` — wipe all chat data

### ChatFilter

```typescript
interface ChatFilter {
  channel?: string;
  sender?: string;
  searchText?: string;
  startTime?: string;
  endTime?: string;
  hasItemLinks?: boolean;
  itemName?: string;
  tellPartner?: string;
  limit?: number;
  offset?: number;
}
```

## Key Design Decisions

- **FTS indexing** — full-text search via SQLite FTS5 for fast text queries across potentially millions of messages.
- **Item link extraction at parse time** — item references are detected during import and stored in a separate table, enabling "item links only" filtering without re-scanning messages.
- **Watchword rules in settings** — rules persist in `settingsStore` (app settings file) rather than the database, keeping them lightweight and portable.
- **Deduplication** — `INSERT OR IGNORE` prevents duplicate messages when re-importing or tailing overlapping ranges.
- **Offset pagination** — simple offset/limit pagination rather than cursor-based, sufficient for chat browsing patterns.
