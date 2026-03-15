# Chat Item Linking

## Overview

This feature automatically detects and links item references in chat messages to actual game items from the CDN data.

## Channel Filtering

The following channels are **excluded** from import to reduce database size and noise:
- Error
- Emotes
- Action Emotes
- NPC Chatter
- System

These channels typically don't contain relevant player communication or item references.

## Item Link Format

When players link items in chat, they appear in the following formats:

- `[Item: Leatherworking: Great Evasion Shirt]` - Item with skill prefix
- `[Item: Amazing Cloth Shirt]` - Item without skill prefix

## Implementation

### Parsing

The chat parser ([chat_parser.rs](../../src-tauri/src/chat_parser.rs)) extracts item links from messages using the `extract_item_links()` function, which:

1. Searches for `[Item: ...]` patterns in messages
2. Extracts the item name and optional skill
3. Returns a vector of `ItemLink` structs

### Database Storage

Item links are stored in the `chat_item_links` table with the following schema:

```sql
CREATE TABLE chat_item_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER NOT NULL,
    raw_text TEXT NOT NULL,           -- e.g., "[Item: Leatherworking: Great Evasion Shirt]"
    item_name TEXT NOT NULL,          -- e.g., "Great Evasion Shirt"
    skill TEXT,                       -- e.g., "Leatherworking"
    item_id INTEGER,                  -- Foreign key to items table (resolved by name)
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (message_id) REFERENCES chat_messages(id) ON DELETE CASCADE,
    FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE SET NULL
);
```

### Item Resolution

When inserting chat messages, the system attempts to resolve item names to item IDs by:

1. Querying the `items` table with case-insensitive name matching
2. Storing the `item_id` if found, or `NULL` if not found
3. This allows future UI features to display full item information (description, icon, value, etc.)

## Usage

### When Reading Chat Messages

The `get_chat_messages()` function automatically loads item links for each message:

```rust
pub struct ChatMessageRow {
    pub id: i64,
    pub timestamp: String,
    pub channel: Option<String>,
    pub sender: Option<String>,
    pub message: String,
    pub is_system: bool,
    pub from_player: Option<bool>,
    pub item_links: Vec<ChatItemLinkRow>,
}

pub struct ChatItemLinkRow {
    pub raw_text: String,
    pub item_name: String,
    pub skill: Option<String>,
    pub item_id: Option<i64>,
}
```

### Frontend Integration

The frontend automatically renders item links in chat messages:

#### Components

- **[MessageWithItemLinks.vue](../../src/components/Chat/MessageWithItemLinks.vue)** - Renders item links as interactive elements
- **[ChatMessage.vue](../../src/components/Chat/ChatMessage.vue)** - Integrates item link rendering into chat messages

#### Features

- **Visual Distinction**: Item links appear with purple background and border
- **Hover Tooltips**: Shows item name, skill, and database ID on hover
- **Click Handling**: Click events for future integration with item detail views
- **Graceful Degradation**: Shows original message text if no item links present

Frontend code can use the `item_id` to:

- Display item tooltips with full details
- Link to item detail pages
- Show item icons
- Filter chat by linked items
- Track which items are being discussed in chat

## Testing

Tests are included in [chat_parser.rs](../../src-tauri/src/chat_parser.rs#L417) covering:

- Item links with skill prefix
- Item links without skill prefix
- Multiple item links in a single message
- Messages with no item links
- Full chat message parsing with item links
