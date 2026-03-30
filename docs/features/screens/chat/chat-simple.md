# Chat — Party, Nearby, Guild, System

## Overview

Four simple single-channel views that each display messages from a fixed channel.

## Tabs

| Tab | Channel | Content |
|-----|---------|---------|
| **Party** | Party | Party chat messages |
| **Nearby** | Nearby | Local/area chat |
| **Guild** | Guild | Guild chat messages |
| **System** | Status | System messages (loot, XP gains, status updates) |

## How They Work

All four tabs follow the same pattern:
- Fixed to their respective channel (no channel selection needed)
- Simple message list via `ChatMessageList` in standard layout
- Paginated loading (100 messages per page)
- Item links rendered via `ItemInline`
- No sidebar or additional filtering
