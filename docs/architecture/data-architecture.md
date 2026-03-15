# Data Architecture Reference

This document describes the current data architecture for glogger, implemented during the pre-alpha refactor. This architecture provides a stable foundation for the application with proper separation of concerns between Rust backend and Vue frontend.

## Architecture Overview

The architecture follows an **event-driven, Rust-managed** design pattern:

- **Rust owns the lifecycle** - File watching, polling intervals, and data processing are managed by Rust
- **Frontend is the display layer** - Vue components listen to events and display state
- **Database is the source of truth** - All persistent state lives in SQLite with proper deduplication
- **Events flow from Rust to Frontend** - State changes are pushed via Tauri events, not polled

## Core Components

### 1. DataIngestCoordinator (Rust)

**Location:** [`src-tauri/src/coordinator.rs`](../../src-tauri/src/coordinator.rs)

The central coordinator for all file watching and database operations.

**Responsibilities:**
- Manages PlayerLogWatcher and ChatLogWatcher lifecycle
- Prevents operation conflicts via operation locking
- Coordinates database writes
- Emits progress events to frontend
- Saves/restores file positions from database

**Key Methods:**
```rust
// Start/stop watchers
pub fn start_player_log_tailing(&mut self) -> Result<(), String>
pub fn stop_player_log_tailing(&mut self) -> Result<(), String>
pub fn start_chat_log_tailing(&mut self, chat_log_path: PathBuf) -> Result<(), String>
pub fn stop_chat_log_tailing(&mut self) -> Result<(), String>

// Poll all active watchers for new events
pub fn poll(&mut self) -> Result<(), String>

// Get current status
pub fn get_status(&self) -> CoordinatorStatus
```

**Operation Types:**
- `Idle` - No operation in progress
- `PlayerLogTailing` - Monitoring Player.log
- `ChatTailing` - Monitoring daily chat logs
- `FullChatScan` - Scanning all historical chat logs (future)
- `CharacterImport` - Importing character snapshots (future)
- `UserAction` - User-triggered priority action (future)

**Events Emitted:**
- `coordinator-status` - Status changes (watchers started/stopped)
- `character-login` - Character logged in
- `area-transition` - Area changed
- `chat-messages-inserted` - New chat messages added to database

### 2. Log File Watchers (Rust)

**Location:** [`src-tauri/src/log_watchers.rs`](../../src-tauri/src/log_watchers.rs)

Unified trait-based system for watching and parsing log files.

#### LogFileWatcher Trait

```rust
pub trait LogFileWatcher {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self);
    fn poll(&mut self) -> Result<Vec<LogEvent>, String>;
    fn get_position(&self) -> u64;
    fn is_active(&self) -> bool;
}
```

#### PlayerLogWatcher

Monitors `Player.log` - the master log file.

**Tracks:**
- Character login/logout events
- Chat log path announcements
- Area transitions
- Survey events, XP gains, item loots (extensible via pattern registration)

**Pattern Registration System:**
The PlayerLogWatcher uses an extensible pattern registration system that allows modules to add custom log line matchers without modifying core code. See [Player Log Pattern Registration Guide](../guides/player-log-pattern-registration.md) for details.

**Core Patterns:**
```
Logged in as character [CharacterName]
Logging chat to C:/Users/.../ChatLogs/Chat-26-03-06.log
LOADING LEVEL AreaCasino
```

**Adding Custom Patterns:**
```rust
fn match_custom_event(line: &str, _watcher: &mut PlayerLogWatcher) -> Option<LogEvent> {
    if line.contains("MY PATTERN") {
        Some(LogEvent::CustomEvent { /* ... */ })
    } else {
        None
    }
}

watcher.register_pattern(match_custom_event);
```

#### ChatLogWatcher

Monitors daily chat log files (`Chat-YY-MM-DD.log`).

**Features:**
- Session tracking with login/logout markers
- Message parsing and deduplication
- Position tracking for incremental reads

**Key Patterns Parsed:**
```
******** Logged In As [Name]
******** Logged Out
26-03-09 14:23:45\t[Global] Sender: Message
```

### 3. Settings Manager (Rust)

**Location:** [`src-tauri/src/settings.rs`](../../src-tauri/src/settings.rs)

Early-initialization settings management system.

**Key Features:**
- Loads before database initialization
- Determines database path (custom or default)
- Stores auto-start preferences
- File-based storage (`settings.json` in app data dir)

**Settings Schema:**
```rust
pub struct AppSettings {
    pub log_file_path: String,              // Legacy, may deprecate
    pub auto_watch_on_startup: bool,        // Legacy player log watching
    pub game_data_path: String,             // Root game data directory
    pub auto_purge_enabled: bool,           // Auto-delete old data
    pub auto_purge_days: u32,               // Days to keep
    pub auto_tail_chat: bool,               // Auto-start chat tailing
    pub auto_tail_player_log: bool,         // Auto-start player log tailing
    pub db_path: Option<String>,            // Custom database path
}
```

**Initialization Order:**
1. Get app data directory
2. Initialize SettingsManager (reads `settings.json`)
3. Get database path from settings
4. Initialize database with that path
5. Initialize DataIngestCoordinator

See: [`src-tauri/src/lib.rs:108-130`](../../src-tauri/src/lib.rs#L108-L130)

### 4. Database Schema

**Location:** [`src-tauri/src/db/migrations.rs`](../../src-tauri/src/db/migrations.rs)

Single unified schema (V1) with no versioning needed yet.

#### Core Tables

**log_file_positions** - Unified position tracking for all log types
```sql
CREATE TABLE log_file_positions (
    file_path TEXT PRIMARY KEY,
    file_type TEXT NOT NULL CHECK (file_type IN ('chat', 'player')),
    last_position INTEGER NOT NULL DEFAULT 0,
    last_modified TIMESTAMP,
    last_processed TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    player_name TEXT,
    metadata TEXT
);
```

**chat_messages** - Chat messages with deduplication
```sql
CREATE TABLE chat_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TIMESTAMP NOT NULL,
    channel TEXT,
    sender TEXT,
    message TEXT NOT NULL,
    is_system BOOLEAN NOT NULL DEFAULT 0,
    log_file TEXT NOT NULL,
    from_player BOOLEAN DEFAULT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Unique constraint prevents duplicates
CREATE UNIQUE INDEX idx_chat_messages_unique
ON chat_messages(timestamp, channel, sender, message);
```

**character_snapshots** - Character progression tracking (from /outputcharacter)
```sql
CREATE TABLE character_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    character_name TEXT NOT NULL,
    server_name TEXT NOT NULL,
    snapshot_timestamp TIMESTAMP NOT NULL,
    race TEXT,
    import_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    raw_json TEXT NOT NULL,
    UNIQUE(character_name, server_name, snapshot_timestamp)
);
```

See full schema: [`migration_v1_unified_schema`](../../src-tauri/src/db/migrations.rs#L26-L468)

### 5. Frontend Stores

#### coordinatorStore (Vue/Pinia)

**Location:** [`src/stores/coordinatorStore.ts`](../../src/stores/coordinatorStore.ts)

Event-driven store that bridges Rust coordinator to Vue frontend.

**Listens for Events:**
- `coordinator-status` - Updates coordinator status
- `character-login` - Logs character login
- `area-transition` - Logs area transitions
- `chat-messages-inserted` - Updates new message count

**Provides Actions:**
```typescript
startPlayerTailing(): Promise<void>
stopPlayerTailing(): Promise<void>
startChatTailing(): Promise<void>
stopChatTailing(): Promise<void>
refreshStatus(): Promise<void>
startPolling(intervalMs): void  // Start frontend polling loop
stopPolling(): void
resetMessageCount(): void
```

**Computed State:**
```typescript
activeCharacter: string | null
isPlayerLogTailing: boolean
isChatLogTailing: boolean
currentOperation: string
newChatMessageCount: number
```

#### chatStore (Vue/Pinia)

**Location:** [`src/stores/chatStore.ts`](../../src/stores/chatStore.ts)

Thin wrapper around coordinatorStore for backward compatibility.

**Purpose:**
- Maintains existing component interfaces
- Delegates to coordinatorStore
- Simplified from 70 lines to 40 lines

## Data Flow

### Chat Message Ingestion Flow

```
1. User logs into game
   └─> Player.log: "Logging chat to .../Chat-26-03-09.log"

2. PlayerLogWatcher.poll() detects new line
   └─> Emits LogEvent::ChatLogPath

3. DataIngestCoordinator.process_player_events()
   └─> Stops old ChatLogWatcher (if any)
   └─> Starts new ChatLogWatcher for today's log

4. Frontend polls via poll_watchers() every 1.5s
   └─> Coordinator calls poll() on all active watchers

5. ChatLogWatcher.poll() reads new lines
   └─> Parses chat messages
   └─> Returns Vec<LogEvent::ChatMessage>

6. DataIngestCoordinator.process_chat_events()
   └─> Batch inserts to database (INSERT OR IGNORE)
   └─> Emits "chat-messages-inserted" event

7. Frontend coordinatorStore receives event
   └─> Updates newChatMessageCount
   └─> Components reactively update
```

### Position Persistence Flow

```
1. Watcher stops (manually or on app shutdown)
   └─> DataIngestCoordinator.stop_chat_log_tailing()

2. Get current file position from watcher
   └─> watcher.get_position() -> u64

3. Save to database
   └─> log_positions::update_position()
   └─> Upserts to log_file_positions table

4. Next startup
   └─> DataIngestCoordinator.start_chat_log_tailing()
   └─> log_positions::get_position() from database
   └─> ChatLogWatcher::from_position(path, position)
   └─> Resumes from saved position
```

## Tauri Commands

### Coordinator Commands

All commands use `State<Arc<Mutex<DataIngestCoordinator>>>` pattern.

```rust
#[tauri::command]
pub fn start_player_tailing(
    coordinator: State<Arc<Mutex<DataIngestCoordinator>>>,
) -> Result<(), String>

#[tauri::command]
pub fn stop_player_tailing(
    coordinator: State<Arc<Mutex<DataIngestCoordinator>>>,
) -> Result<(), String>

#[tauri::command]
pub fn start_chat_tailing(
    coordinator: State<Arc<Mutex<DataIngestCoordinator>>>,
    settings: State<Arc<SettingsManager>>,
) -> Result<(), String>

#[tauri::command]
pub fn stop_chat_tailing(
    coordinator: State<Arc<Mutex<DataIngestCoordinator>>>,
) -> Result<(), String>

#[tauri::command]
pub fn get_coordinator_status(
    coordinator: State<Arc<Mutex<DataIngestCoordinator>>>,
) -> Result<CoordinatorStatus, String>

#[tauri::command]
pub fn poll_watchers(
    coordinator: State<Arc<Mutex<DataIngestCoordinator>>>,
) -> Result<(), String>
```

### Chat Query Commands

```rust
#[tauri::command]
pub fn get_chat_messages(
    db: State<DbPool>,
    channel: Option<String>,
    search: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<ChatMessageRow>, String>

#[tauri::command]
pub fn get_chat_channels(
    db: State<DbPool>,
) -> Result<Vec<String>, String>

#[tauri::command]
pub fn scan_chat_logs(
    db: State<DbPool>,
    chat_logs_dir: String,
    app_handle: AppHandle,
) -> Result<ScanResult, String>
```

## Frontend Usage

### Initializing Coordinator

In [`App.vue`](../../src/App.vue):

```typescript
import { useCoordinatorStore } from "./stores/coordinatorStore"

const coordinator = useCoordinatorStore()

onMounted(async () => {
  // Start coordinator polling
  coordinator.startPolling(1500) // Poll every 1.5 seconds

  // Auto-start if enabled in settings
  if (settingsStore.settings.autoTailPlayerLog) {
    await coordinator.startPlayerTailing()
  }

  if (settingsStore.settings.autoTailChat) {
    await coordinator.startChatTailing()
  }
})
```

### Using Chat Store

Components continue using the chatStore interface:

```vue
<script setup lang="ts">
import { useChatStore } from '../../stores/chatStore'

const chatStore = useChatStore()

async function startTailing() {
  await chatStore.startTailing()
}
</script>

<template>
  <div v-if="chatStore.tailing">
    <p>Monitoring: {{ chatStore.currentLogFile }}</p>
    <p v-if="chatStore.newMessageCount > 0">
      {{ chatStore.newMessageCount }} new messages
    </p>
  </div>
</template>
```

## Testing

### Running Tests

```bash
cd src-tauri
cargo test
```

### Test Coverage

**log_watchers module** - 6 tests, all passing:
- `test_parse_character_login` - PlayerLogWatcher login detection
- `test_parse_chat_log_path` - PlayerLogWatcher chat path detection
- `test_parse_area_transition` - PlayerLogWatcher area detection
- `test_chat_login_marker` - ChatLogWatcher login marker
- `test_chat_logout_marker` - ChatLogWatcher logout marker
- `test_chat_message_parsing` - ChatLogWatcher message parsing

## Future Enhancements

### Planned Features (Phase 8-9)

1. **Character Export Import** - Schema exists, commands needed
   - Import `/outputcharacter` JSON files
   - Import `/outputitems` JSON files
   - Track progression over time

2. **Full Chat Scan** - Background scanning of all historical logs
   - Progress reporting
   - Cancellation support
   - Operation locking integration

3. **Survey Event Parsing** - Extract from Player.log
   - Session detection
   - Loot tracking
   - XP tracking

4. **Background Polling** - Move from frontend to Rust
   - Tokio background task
   - Automatic polling without frontend
   - Wake lock / system sleep handling

### Architecture Improvements

1. **Error Handling**
   - Better error propagation
   - User-friendly error messages
   - Retry logic for transient failures

2. **Performance**
   - Batch size tuning for message inserts
   - Database connection pooling optimization
   - Memory usage monitoring

3. **Observability**
   - Structured logging
   - Performance metrics
   - Debug mode for development

## Migration Notes

### From Old Architecture

**Before (Frontend-driven):**
- Frontend polled Rust every 2 seconds
- `tail_chat_log` command read file on each call
- No position persistence
- Database lock contention
- Duplicate messages

**After (Rust-managed):**
- Rust polls watchers every 1.5 seconds
- Frontend polls Rust to trigger watcher poll
- Position saved to database
- Coordinated writes via mutex
- Deduplication via unique index

### Breaking Changes

None - chatStore interface maintained for backward compatibility.

### Deprecated Commands

The following may be deprecated in the future:
- `tail_chat_log` - Use coordinator commands instead
- Legacy survey tracking tables - Use new survey_events schema

## Debugging

### Enable Debug Logging

In Rust code:
```rust
eprintln!("Debug: {:?}", some_value);
```

### View Events in Frontend

```javascript
import { listen } from '@tauri-apps/api/event'

// Log all coordinator status events
listen('coordinator-status', (event) => {
  console.log('Coordinator status:', event.payload)
})
```

### Check Database State

```sql
-- View current log positions
SELECT * FROM log_file_positions ORDER BY last_processed DESC;

-- Check for duplicate messages (should be none)
SELECT timestamp, channel, sender, message, COUNT(*) as count
FROM chat_messages
GROUP BY timestamp, channel, sender, message
HAVING count > 1;

-- View recent chat messages
SELECT * FROM chat_messages ORDER BY created_at DESC LIMIT 10;
```

## References

- [Original Architecture Plan](../plans/data-architecture.md)
- [Database Schema](../../src-tauri/src/db/migrations.rs)
- [Coordinator Implementation](../../src-tauri/src/coordinator.rs)
- [Log Watchers](../../src-tauri/src/log_watchers.rs)
- [Frontend Store](../../src/stores/coordinatorStore.ts)
