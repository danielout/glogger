# Data Architecture Planning

**Status:** ✅ Phases 1-7 Complete | 🚧 Phases 8-9 Pending

**Implementation Date:** 2026-03-09

**Documentation:**
- [Architecture Reference](../reference/data-architecture.md) - Complete technical reference
- [Architecture Summary](../reference/architecture-summary.md) - Quick reference guide
- [Developer Guide](../guides/working-with-data-architecture.md) - How to work with the system
- [Implementation Checklist](../reference/implementation-checklist.md) - Task checklists

---

# Original Planning Document

Below is the original planning document used for the refactor. See documentation links above for current implementation details.

## Current State Analysis

### Data Sources

We have multiple concurrent data sources that need to be managed:

1. **Player.log** - Real-time game events
   - Character login detection: `Logged in as character [Name]`
   - Network state changes
   - Area transitions: `LOADING LEVEL AreaCasino`
   - Chat log path announcements: `Logging chat to [path]`
   - Survey completions, XP gains, item loots
   - Continuous append-only file
   - **Master log** - tells us which character is active and where their chat log is

2. **ChatLogs/Chat-YY-MM-DD.log** - Daily chat logs
   - One file per day, **spans multiple play sessions**
   - Player messages, system messages, tells
   - Item links embedded in messages
   - Contains login/logout markers: `******** Logged In As [Name]` and `******** Logged Out`
   - Path announced in Player.log: `Logging chat to C:/Users/.../ChatLogs/Chat-26-03-06.log`
   - Continuous append-only (current day)

3. **CDN Game Data** (static, periodic updates)
   - Items, skills, abilities, recipes, NPCs, quests
   - Updated when game patches
   - Relatively stable reference data

4. **Character Export JSONs** (user-triggered snapshots)
   - **Character sheets**: Skills, levels, XP, abilities, NPC favor
   - **Storage/Inventory**: Items across all vaults with TypeID, StackSize, Value
   - Generated manually by player using in-game `/outputcharacter` and `/outputitems` commands
   - Saved to game data folder: `%AppData%\LocalLow\Elder Game\Project Gorgon\`
   - Point-in-time snapshots for tracking progression over time
   - Samples in `docs/character-export-samples/`
   - Large files (1400+ lines for character, 8400+ lines for items)
   - Should support both auto-detection AND manual import for testing/saved files

5. **User Actions** (UI-triggered)
   - Manual data entry
   - Character export imports
   - Settings changes
   - Database management operations

6. **settings.json** (application configuration)
   - Game data path: `%AppData%\LocalLow\Elder Game\Project Gorgon\` by default. Changable in settings
   - From this root, can locate:
     - `Player.log` - in root
     - `ChatLogs/` - subdirectory with daily logs
     - Character export JSONs - in `Reports` subdirectory
   - UI preferences
   - Auto-start options
   - Should load early, independent of database

### Current Implementation Issues

#### 1. **Duplicate Message Imports**
- Location: [chat_commands.rs:6-60](src-tauri/src/db/chat_commands.rs#L6-L60)
- No uniqueness constraint or deduplication logic
- Re-scanning or overlapping operations cause duplicates

#### 2. **Database Lock Contention**
- Tailing polls every 2 seconds from frontend
- Manual scans can run concurrently
- Both compete for connections from shared pool (max 15)
- No coordination between operations

#### 3. **Frontend-Driven Architecture**
- Location: [chatStore.ts:36-45](src/stores/chatStore.ts#L36-L45)
- Frontend manages polling intervals
- Rust is passive responder
- No operation awareness or conflict prevention

#### 4. **No Progress Feedback**
- Initial tailing scan can process thousands of messages silently
- User has no visibility into what's happening

#### 5. **No Unified Log File Management**
- Chat logs and Player.log handled separately
- No shared abstraction for "file watching + incremental parsing"
- Duplicated position tracking logic

### Current Technology: SQLite

**Why SQLite works well:**
- Single file, zero-config
- ACID transactions
- Good for read-heavy workloads
- FTS5 for full-text search
- Handles concurrent readers well
- Perfect for desktop apps

**Current limitations we're hitting:**
- Write serialization (only one writer at a time)
- Connection pool contention when multiple operations overlap
- No built-in pub/sub for real-time updates

**SQLite is still the right choice** - we just need better coordination.

### Character Export Data

Character exports provide valuable point-in-time snapshots:

**Character Sheet Export** (`Character_[Name]_[Server].json`):
- All skills with levels, bonus levels, and XP progress
- Complete ability lists per skill
- All NPC favor levels (300+ NPCs)
- Character metadata (race, server, timestamp)

**Storage/Inventory Export** (`[Name]_[Server]_items_[timestamp]Z.json`):
- Every item across all storage vaults
- Item metadata: TypeID, StackSize, Value, Name
- Vault location (e.g., "CouncilVault", "Inventory", etc.)
- Can contain thousands of items

**Use Cases:**
- Track skill progression over time
- Analyze wealth/item accumulation
- See NPC favor changes
- Compare snapshots to measure progress
- Historical analysis of character development

**Challenge:** These are large, user-initiated imports that could take significant time to process. Need clear progress feedback and coordination with other operations.

## Proposed Architecture

### Principle: Rust-Managed, Event-Driven

**Core concept:** Rust should own the lifecycle of all file watching and database operations. The frontend should be a **display layer** that reacts to state changes.

### Component Design

#### 1. **LogFileWatcher** (New Abstraction)

A generic file watcher that:
- Tracks file position in database
- Polls file for new content
- Parses lines incrementally
- Emits parsed events
- Handles rotation (e.g., daily chat logs)
- **PlayerLogWatcher is master** - detects active character and chat log path

```rust
trait LogFileWatcher {
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self);
    fn poll(&mut self) -> Result<Vec<LogEvent>>;
    fn get_position(&self) -> u64;
}

struct PlayerLogWatcher {
    // Monitors Player.log for:
    // - Character login/logout
    // - Chat log path changes
    // - Area transitions
    // - Survey events, XP, etc.
    active_character: Option<String>,
    current_chat_log: Option<PathBuf>,
}

struct ChatLogWatcher {
    // Monitors specific chat log file
    // Multiple sessions per file (daily file)
    // Parses login/logout markers to track sessions
    current_session_start: Option<NaiveDateTime>,
}
```

#### 2. **DataIngestCoordinator** (New)

Central coordinator that:
- Manages all file watchers
- Prevents overlapping operations
- Coordinates database writes
- Emits progress/status events to frontend
- Handles operation priorities

```rust
struct DataIngestCoordinator {
    chat_watcher: Option<ChatLogWatcher>,
    player_watcher: Option<PlayerLogWatcher>,
    operation_lock: Arc<RwLock<OperationType>>,
    db_pool: DbPool,
}

enum OperationType {
    Idle,
    FullChatScan { progress: ScanProgress },
    ChatTailing,
    PlayerLogTailing,
    CharacterImport { progress: ImportProgress },
    UserAction,
}
```

#### 3. **Settings Manager** (Early Init)

Loads settings.json before database initialization:
- Game data paths
- Auto-start preferences
- Database location
- Must be available for all other components

```rust
// In main.rs or lib.rs
fn setup() -> Result<AppState> {
    let settings = SettingsManager::load()?;
    let db_pool = init_pool(&settings.db_path)?;
    let coordinator = DataIngestCoordinator::new(db_pool, &settings)?;

    // Auto-start based on settings
    if settings.auto_tail_chat {
        coordinator.start_chat_tailing()?;
    }

    Ok(AppState { settings, coordinator, db_pool })
}
```

#### 4. **Frontend Communication Strategy**

**Hybrid approach:** Components choose based on their needs.

##### Events (for important state changes)
Rust emits key events that components can subscribe to:

```rust
// High-level state changes
app.emit("watcher-status-changed", WatcherStatus {
    watcher_type: "chat",
    state: "tailing",
    active_character: "Zenith",
})?;

app.emit("character-login", CharacterLogin {
    character_name: "Zenith",
    timestamp: "2026-03-09 18:15:08",
    area: "Statehelm",
})?;

app.emit("import-progress", ImportProgress {
    operation: "character_snapshot",
    current: 50,
    total: 100,
})?;
```

##### Polling (for data queries)
Most data access happens via frontend polling:

```typescript
// Component decides when to refresh
onMounted(async () => {
  messages.value = await invoke('get_chat_messages', { limit: 50 })
})

// Or periodic refresh
setInterval(async () => {
  stats.value = await invoke('get_chat_stats')
}, 5000)
```

**Rationale:**
- Events for **state changes** (started tailing, character logged in, import finished)
- Polling for **data retrieval** (get messages, get stats, get skills)
- Prevents event spam (don't emit event for every chat message)
- Components control their own refresh rates
- Simpler mental model: "ask when you need it"

#### 5. **Database Write Strategy**

**Deduplication:**
Add unique constraints and use `INSERT OR IGNORE`:

```sql
-- For chat messages
CREATE UNIQUE INDEX idx_chat_messages_unique
ON chat_messages(timestamp, channel, sender, message);

-- Insert becomes:
INSERT OR IGNORE INTO chat_messages (...) VALUES (...);
```

**Batching:**
Accumulate writes and commit in batches to reduce lock contention:

```rust
impl DataIngestCoordinator {
    fn process_batch(&mut self, events: Vec<LogEvent>) -> Result<()> {
        let conn = self.db_pool.get()?;
        let tx = conn.transaction()?;

        for event in events {
            // All inserts in one transaction
            insert_event(&tx, event)?;
        }

        tx.commit()?;
        Ok(())
    }
}
```

**Priority Queue:**
User actions get priority over background tasks:

```rust
enum WriteRequest {
    HighPriority(UserAction),
    LowPriority(BackgroundEvent),
}
```

#### 5. **Frontend Communication**

Use Tauri events for state updates:

```rust
// Rust emits events
app.emit("chat-status", ChatStatus {
    state: "tailing",
    messages_processed: 1234,
    last_message_time: "2026-03-09 10:30:00",
})?;

app.emit("player-log-event", SurveyCompleted { ... })?;
```

```typescript
// Frontend listens
import { listen } from '@tauri-apps/api/event'

listen('chat-status', (event) => {
  chatStore.updateStatus(event.payload)
})

listen('player-log-event', (event) => {
  // Update UI with new survey data
})
```

### Operation Coordination

#### Starting Chat Tailing

**Current flow:**
1. User clicks "Start Tailing"
2. Frontend starts setInterval
3. Frontend calls `tail_chat_log` every 2s
4. Rust processes request each time

**New flow:**
1. User clicks "Start Tailing"
2. Frontend calls `start_chat_tailing` ONCE
3. Rust spawns background task
4. Rust emits events as messages arrive
5. Frontend displays events

```rust
#[tauri::command]
async fn start_chat_tailing(
    coordinator: State<'_, Arc<Mutex<DataIngestCoordinator>>>,
) -> Result<(), String> {
    let mut coord = coordinator.lock().unwrap();
    coord.start_chat_tailing()?;
    Ok(())
}

impl DataIngestCoordinator {
    fn start_chat_tailing(&mut self) -> Result<()> {
        // Check for conflicts
        if matches!(self.operation_lock.read(), OperationType::FullScan { .. }) {
            return Err("Cannot start tailing during full scan");
        }

        // Create watcher
        let watcher = ChatLogWatcher::new(&self.settings)?;

        // Initial scan with progress
        self.initial_scan(&watcher)?;

        // Start background polling
        self.spawn_tailing_task(watcher)?;

        Ok(())
    }
}
```

#### Full Scan vs Tailing

**Mutex-based coordination:**

```rust
impl DataIngestCoordinator {
    async fn start_full_scan(&mut self) -> Result<()> {
        // Stop tailing if active
        if self.chat_watcher.is_some() {
            self.stop_chat_tailing()?;
        }

        *self.operation_lock.write() = OperationType::FullScan {
            progress: ScanProgress::default()
        };

        // Scan all files...

        *self.operation_lock.write() = OperationType::Idle;
        Ok(())
    }
}
```

### File Position Tracking

Unified approach for all log files:

```sql
-- Already exists: chat_log_files
-- Add similar for player log
CREATE TABLE log_file_positions (
    file_path TEXT PRIMARY KEY,
    file_type TEXT NOT NULL, -- 'chat' or 'player'
    last_position INTEGER NOT NULL,
    last_modified TIMESTAMP,
    last_processed TIMESTAMP
);
```

### Character Snapshot Storage

Store character exports as historical snapshots:

```sql
-- Character snapshots (from /outputcharacter)
CREATE TABLE character_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    character_name TEXT NOT NULL,
    server_name TEXT NOT NULL,
    snapshot_timestamp TIMESTAMP NOT NULL,
    race TEXT,
    import_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    raw_json TEXT NOT NULL,  -- Store full JSON for reference
    UNIQUE(character_name, server_name, snapshot_timestamp)
);
CREATE INDEX idx_snapshots_char ON character_snapshots(character_name, snapshot_timestamp DESC);

-- Skill levels per snapshot
CREATE TABLE character_skill_levels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    snapshot_id INTEGER NOT NULL,
    skill_name TEXT NOT NULL,
    level INTEGER NOT NULL,
    bonus_levels INTEGER NOT NULL DEFAULT 0,
    xp_toward_next INTEGER NOT NULL DEFAULT 0,
    xp_needed_for_next INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (snapshot_id) REFERENCES character_snapshots(id) ON DELETE CASCADE
);
CREATE INDEX idx_skill_levels_snapshot ON character_skill_levels(snapshot_id);
CREATE INDEX idx_skill_levels_skill ON character_skill_levels(skill_name, snapshot_id);

-- NPC favor levels per snapshot
CREATE TABLE character_npc_favor (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    snapshot_id INTEGER NOT NULL,
    npc_key TEXT NOT NULL,
    favor_level TEXT NOT NULL,
    FOREIGN KEY (snapshot_id) REFERENCES character_snapshots(id) ON DELETE CASCADE
);
CREATE INDEX idx_npc_favor_snapshot ON character_npc_favor(snapshot_id);
CREATE INDEX idx_npc_favor_npc ON character_npc_favor(npc_key, snapshot_id);

-- Item snapshots (from /outputitems)
CREATE TABLE character_item_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    character_name TEXT NOT NULL,
    server_name TEXT NOT NULL,
    snapshot_timestamp TIMESTAMP NOT NULL,
    import_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    raw_json TEXT NOT NULL,
    UNIQUE(character_name, server_name, snapshot_timestamp)
);

-- Individual items in snapshots
CREATE TABLE character_snapshot_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_snapshot_id INTEGER NOT NULL,
    type_id INTEGER NOT NULL,
    storage_vault TEXT NOT NULL,
    stack_size INTEGER NOT NULL,
    value INTEGER,
    item_name TEXT NOT NULL,
    FOREIGN KEY (item_snapshot_id) REFERENCES character_item_snapshots(id) ON DELETE CASCADE,
    FOREIGN KEY (type_id) REFERENCES items(id) ON DELETE SET NULL
);
CREATE INDEX idx_snapshot_items_snapshot ON character_snapshot_items(item_snapshot_id);
CREATE INDEX idx_snapshot_items_vault ON character_snapshot_items(storage_vault);
CREATE INDEX idx_snapshot_items_type ON character_snapshot_items(type_id);
```

### Settings Management

**settings.json structure:**
```json
{
  "gameDataPath": "C:/Users/.../ProjectGorgon",
  "autoTailChat": true,
  "autoTailPlayerLog": true,
  "autoPurgeEnabled": false,
  "autoPurgeDays": 30,
  "dbPath": null  // null = default app data dir
}
```

**Loading sequence:**
1. App starts
2. Load settings.json (or create defaults)
3. Initialize database at specified path
4. Run migrations
5. Initialize DataIngestCoordinator with settings
6. Auto-start watchers based on settings
7. Show UI

## Implementation Plan

### Phase 1: Deduplication & Conflict Prevention
- Add unique indexes to prevent duplicates
- Add operation locking to coordinator
- Change inserts to `INSERT OR IGNORE`

### Phase 2: Settings Early Init
- Create SettingsManager
- Load settings before database init
- Pass settings to all components

### Phase 3: Refactor Chat Tailing
- Create ChatLogWatcher abstraction
- Move tailing to Rust background task
- Add progress events
- Update frontend to listen for events

### Phase 4: Generalize for Player.log
- Create PlayerLogWatcher
- Use same coordination patterns
- Share file position tracking

### Phase 5: Character Export Imports
- Create character snapshot tables
- Build JSON parser for both export types
- Add import commands with progress events
- Support both auto-detection (scan game data folder) AND manual import
- Handle deduplication (same timestamp = same snapshot)
- UI for viewing progression over time

### Phase 6: Unified Coordinator
- Centralize all file watching and imports
- Single point of coordination
- Priority queue for writes
- Prevent conflicts between operations

## Open Questions

1. **Event replay:** Should we support replaying events from database (e.g., "reprocess all chat from last week")?

2. **Real-time updates:** How should UI update when new messages arrive?
   - Option A: Frontend polls `get_chat_messages` periodically
   - Option B: Rust emits events with actual message data
   - Option C: Hybrid - emit event signal, frontend queries for details

3. **Error handling:** How should we handle file read errors, parse errors, database errors?
   - Log and continue?
   - Stop watching?
   - Emit error events to UI?

4. **Multiple characters:** Users likely have multiple characters on different servers
   - ✅ Player.log tells us active character: `Logged in as character [Name]`
   - ✅ Player.log tells us chat log path: `Logging chat to [path]`
   - Only one character can be active at a time (game limitation)
   - Should support tracking multiple characters over time (character snapshots)
   - Switch chat log watcher when character changes?

5. **Performance tuning:** What batch sizes work best? How often to poll files?

6. **Testing:** How do we test the coordination logic? Mock file system? Test database?

7. **Character export UX:** How should users import exports?
   - ✅ Auto-detect in game data folder (primary method)
   - ✅ Manual file picker for testing/saved files (secondary)
   - Drag-and-drop? (nice-to-have)
   - Should we auto-delete old exports after import?
   - Show "new exports available" notification?

8. **Snapshot comparison:** Should we build diff/comparison tools in the UI?
   - Show skill level changes between snapshots
   - Show item accumulation/loss
   - Display NPC favor progression
   - Calculate net worth changes

## Key Design Decisions

Based on analysis:

1. **Player.log is the master log**
   - Tells us which character is active
   - Points to the correct chat log path
   - Primary watcher, chat watcher is secondary

2. **Chat logs span multiple sessions**
   - Daily files contain multiple login/logout cycles
   - Must track sessions within daily files
   - Login/logout markers: `******** Logged In As [Name]` / `******** Logged Out`

3. **Game data folder is the root**
   - Setting one path (`%AppData%\LocalLow\Elder Game\Project Gorgon\`) gives access to everything
   - Player.log, ChatLogs/, exports all relative to this root

4. **Hybrid event/polling model**
   - Events for important state changes only
   - Polling for data retrieval
   - Components choose what works best for them
   - Avoids event spam

5. **Support both auto and manual imports**
   - Auto-detect exports in game folder
   - Manual import for testing and saved files
   - Critical for development workflow

## Success Criteria

After implementation:
- ✅ No duplicate messages in database
- ✅ No database lock errors
- ✅ Clear progress feedback during operations
- ✅ Tailing works independently of frontend
- ✅ Settings load before any other operations
- ✅ Scan and tail operations don't conflict
- ✅ User actions take priority over background tasks
- ✅ Player.log detects character changes and updates chat watcher
- ✅ Chat logs properly track multiple sessions per day
- ✅ Character exports support both auto-detection and manual import
