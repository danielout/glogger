# Working with the Data Architecture

Quick guide for developers working with glogger's data architecture.

## Quick Start

### Using the Player Event Parser (Preferred for Game Events)

Most game events (`ProcessAddItem`, `ProcessUpdateItemCode`, `ProcessDeleteItem`, `ProcessLoadSkills`, `ProcessStartInteraction`, `ProcessDeltaFavor`, vendor events, storage events, `ProcessScreenText`, `ProcessBook`) are already parsed by the **PlayerEventParser**. Features should subscribe to these events rather than writing their own parsers.

See [`player-event-parser.md`](player-event-parser.md) for the full API, event types, and how to:
- Listen to events on the frontend via `"player-event"` Tauri events
- Add persistence for specific event types in the coordinator
- Extend the parser with new `ProcessXxx` event types

### Adding a New Event Type (Simple Pattern Matcher)

For non-`Process*` lines (e.g., free-text log messages), use the pattern matcher system:

1. **Add LogEvent variant** in [`log_watchers.rs`](../../src-tauri/src/log_watchers.rs):
```rust
pub enum LogEvent {
    // ... existing variants

    /// New event type
    ItemCrafted {
        item_name: String,
        skill: String,
        timestamp: NaiveDateTime,
    },
}
```

2. **Parse the event** in appropriate watcher:
```rust
// In PlayerLogWatcher::parse_line or ChatLogWatcher::parse_line
if line.contains("You crafted") {
    // Parse crafting event
    return Some(LogEvent::ItemCrafted {
        item_name: extracted_name,
        skill: extracted_skill,
        timestamp: chrono::Local::now().naive_local(),
    });
}
```

3. **Handle the event** in [`coordinator.rs`](../../src-tauri/src/coordinator.rs):
```rust
fn process_player_events(&mut self, events: Vec<LogEvent>) -> Result<(), String> {
    for event in events {
        match event {
            LogEvent::ItemCrafted { item_name, skill, .. } => {
                // Store to database
                // Emit to frontend
                self.app_handle.emit("item-crafted", &item_name).ok();
            }
            // ... other events
        }
    }
    Ok(())
}
```

4. **Listen in frontend** (optional):
```typescript
import { listen } from '@tauri-apps/api/event'

listen<string>('item-crafted', (event) => {
  console.log('Item crafted:', event.payload)
})
```

### Adding a New Database Table

1. **Add to schema** in [`migrations.rs`](../../src-tauri/src/db/migrations.rs):
```rust
fn migration_v1_unified_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- ... existing tables

        -- New table
        CREATE TABLE crafting_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            item_name TEXT NOT NULL,
            skill TEXT NOT NULL,
            crafted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX idx_crafting_log_skill ON crafting_log(skill);
        "
    )?;
    Ok(())
}
```

2. **Add query functions** in [`db/queries.rs`](../../src-tauri/src/db/queries.rs) or new module:
```rust
pub mod crafting {
    use super::*;

    pub fn insert_craft(
        conn: &DbConnection,
        item_name: &str,
        skill: &str,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO crafting_log (item_name, skill) VALUES (?1, ?2)",
            params![item_name, skill],
        )?;
        Ok(())
    }

    pub fn get_recent_crafts(
        conn: &DbConnection,
        limit: usize,
    ) -> Result<Vec<CraftEntry>> {
        // Implementation
    }
}
```

3. **Add Tauri command** (if needed for frontend access):
```rust
#[tauri::command]
pub fn get_recent_crafts(
    db: State<DbPool>,
    limit: usize,
) -> Result<Vec<CraftEntry>, String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    crafting::get_recent_crafts(&conn, limit)
        .map_err(|e| e.to_string())
}
```

4. **Register command** in [`lib.rs`](../../src-tauri/src/lib.rs):
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    get_recent_crafts,
])
```

### Adding a New Coordinator Operation

1. **Add OperationType variant** in [`coordinator.rs`](../../src-tauri/src/coordinator.rs):
```rust
pub enum OperationType {
    // ... existing variants

    /// Crafting session tracking
    CraftingSession {
        items_crafted: usize,
    },
}
```

2. **Add start/stop methods**:
```rust
impl DataIngestCoordinator {
    pub fn start_crafting_session(&mut self) -> Result<(), String> {
        // Check for conflicts
        let operation = self.operation_lock.read().unwrap();
        if *operation != OperationType::Idle {
            return Err(format!("Cannot start crafting: {:?} in progress", *operation));
        }
        drop(operation);

        // Set operation type
        *self.operation_lock.write().unwrap() = OperationType::CraftingSession {
            items_crafted: 0,
        };

        // Emit status change
        self.emit_status_change()?;

        Ok(())
    }

    pub fn stop_crafting_session(&mut self) -> Result<(), String> {
        *self.operation_lock.write().unwrap() = OperationType::Idle;
        self.emit_status_change()?;
        Ok(())
    }
}
```

3. **Add Tauri commands**:
```rust
#[tauri::command]
pub fn start_crafting_session(
    coordinator: State<'_, Arc<Mutex<DataIngestCoordinator>>>,
) -> Result<(), String> {
    let mut coord = coordinator.lock().unwrap();
    coord.start_crafting_session()
}
```

### Adding Frontend Store Integration

1. **Extend coordinatorStore** in [`coordinatorStore.ts`](../../src/stores/coordinatorStore.ts):
```typescript
// Add event listener
listen<CraftEvent>('item-crafted', (event) => {
  craftCount.value++
  console.log('Item crafted:', event.payload)
})

// Add action
async function startCraftingSession(): Promise<void> {
  await invoke('start_crafting_session')
  await refreshStatus()
}
```

2. **Or create specialized store**:
```typescript
// src/stores/craftingStore.ts
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

export const useCraftingStore = defineStore('crafting', () => {
  const craftCount = ref(0)
  const sessionActive = ref(false)

  listen('item-crafted', (event) => {
    craftCount.value++
  })

  async function startSession() {
    await invoke('start_crafting_session')
    sessionActive.value = true
  }

  return { craftCount, sessionActive, startSession }
})
```

## Common Patterns

### Pattern: Incremental File Reading

```rust
// Save position when stopping
fn stop(&mut self) {
    let position = self.watcher.get_position();
    log_positions::update_position(
        &conn,
        file_path,
        "custom",
        position,
        None,
        None,
    )?;
}

// Resume from position when starting
fn start(&mut self) {
    let position = log_positions::get_position(&conn, file_path)?;
    self.watcher = CustomWatcher::from_position(path, position);
}
```

### Pattern: Batch Database Inserts

```rust
pub fn insert_batch(conn: &DbConnection, items: &[Item]) -> Result<usize> {
    let mut inserted = 0;

    for item in items {
        let rows = conn.execute(
            "INSERT OR IGNORE INTO items (...) VALUES (...)",
            params![...],
        )?;

        if rows > 0 {
            inserted += 1;
        }
    }

    Ok(inserted)
}
```

### Pattern: Event Emission with Data

```rust
use serde::Serialize;

#[derive(Serialize)]
struct ProgressPayload {
    current: usize,
    total: usize,
    message: String,
}

fn emit_progress(&self, current: usize, total: usize) -> Result<(), String> {
    self.app_handle.emit("operation-progress", ProgressPayload {
        current,
        total,
        message: format!("Processing {} of {}", current, total),
    }).map_err(|e| format!("Failed to emit: {}", e))
}
```

### Pattern: Operation Locking

```rust
pub fn start_operation(&mut self) -> Result<(), String> {
    // Acquire read lock first to check
    let operation = self.operation_lock.read().unwrap();
    if *operation != OperationType::Idle {
        return Err(format!("Cannot start: {:?} in progress", *operation));
    }
    drop(operation); // Release read lock

    // Acquire write lock to modify
    *self.operation_lock.write().unwrap() = OperationType::MyOperation;

    // Do work...

    Ok(())
}
```

## Testing Your Changes

### Unit Tests

Add tests to the relevant module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_crafting_event() {
        let mut watcher = PlayerLogWatcher::new(PathBuf::from("test.log"));

        let event = watcher.parse_line("You crafted Iron Sword!");
        assert!(event.is_some());

        if let Some(LogEvent::ItemCrafted { item_name, .. }) = event {
            assert_eq!(item_name, "Iron Sword");
        } else {
            panic!("Expected ItemCrafted event");
        }
    }
}
```

Run tests:
```bash
cd src-tauri
cargo test
```

### Integration Testing

1. Build the app:
```bash
cd src-tauri
cargo build
```

2. Run the app:
```bash
npm run tauri dev
```

3. Test in browser console:
```javascript
// Check coordinator status
await window.__TAURI__.core.invoke('get_coordinator_status')

// Start operation
await window.__TAURI__.core.invoke('start_my_operation')

// Listen for events
window.__TAURI__.event.listen('my-event', (event) => {
  console.log('Event received:', event.payload)
})
```

## Performance Considerations

### Polling Intervals

Current intervals:
- **Frontend → Rust**: 1.5 seconds (App.vue polling poll_watchers)
- **Watcher file reads**: On-demand when poll() is called

Adjust based on:
- **Faster (< 1s)**: Real-time requirements, instant feedback
- **Slower (> 2s)**: Reduce CPU usage, less critical updates

### Database Connection Pool

Default: 5 connections (see [`db/mod.rs`](../../src-tauri/src/db/mod.rs))

Increase if experiencing timeouts:
```rust
pub fn init_pool(db_path: PathBuf) -> Result<DbPool, String> {
    let manager = SqliteConnectionManager::file(db_path);
    Pool::builder()
        .max_size(10) // Increased from 5
        .build(manager)
        .map_err(|e| format!("Failed to create pool: {e}"))
}
```

### Batch Sizes

For message insertion, process in batches:
```rust
const BATCH_SIZE: usize = 100;

for chunk in messages.chunks(BATCH_SIZE) {
    insert_chat_messages(&conn, chunk, log_file)?;
}
```

## Debugging Tips

### Enable Rust Debug Output

Add to watcher or coordinator:
```rust
eprintln!("[DEBUG] Processing {} events", events.len());
eprintln!("[DEBUG] Current position: {}", self.get_position());
```

### Monitor Database

```bash
sqlite3 path/to/glogger.db
```

```sql
-- Check recent activity
SELECT * FROM log_file_positions ORDER BY last_processed DESC LIMIT 5;

-- Count messages per channel
SELECT channel, COUNT(*) FROM chat_messages GROUP BY channel;

-- Find slowest queries (enable query logging first)
EXPLAIN QUERY PLAN SELECT * FROM chat_messages WHERE channel = 'Global';
```

### Frontend Event Monitoring

```typescript
// Log all events
const unlisten = await listen('*', (event) => {
  console.log('Event:', event.event, event.payload)
})

// Stop logging
unlisten()
```

## Common Pitfalls

### 1. Forgetting to Drop Read Locks

❌ **Wrong:**
```rust
let operation = self.operation_lock.read().unwrap();
if *operation != OperationType::Idle {
    return Err("Busy".to_string());
}
// Lock still held!
*self.operation_lock.write().unwrap() = OperationType::NewOp; // Deadlock!
```

✅ **Correct:**
```rust
let operation = self.operation_lock.read().unwrap();
if *operation != OperationType::Idle {
    return Err("Busy".to_string());
}
drop(operation); // Explicitly release
*self.operation_lock.write().unwrap() = OperationType::NewOp;
```

### 2. Not Handling Database Errors

❌ **Wrong:**
```rust
conn.execute("INSERT INTO ...", params![]).unwrap(); // Will panic!
```

✅ **Correct:**
```rust
conn.execute("INSERT INTO ...", params![])
    .map_err(|e| format!("Failed to insert: {}", e))?;
```

### 3. Forgetting INSERT OR IGNORE

❌ **Wrong:**
```rust
conn.execute("INSERT INTO chat_messages (...) VALUES (...)", params![...])?;
// Crashes on duplicate messages!
```

✅ **Correct:**
```rust
let rows = conn.execute(
    "INSERT OR IGNORE INTO chat_messages (...) VALUES (...)",
    params![...],
)?;
if rows == 0 {
    // Was a duplicate, skip
    continue;
}
```

### 4. Not Emitting Status Changes

❌ **Wrong:**
```rust
pub fn start_operation(&mut self) -> Result<(), String> {
    *self.operation_lock.write().unwrap() = OperationType::Running;
    Ok(())
    // Frontend doesn't know state changed!
}
```

✅ **Correct:**
```rust
pub fn start_operation(&mut self) -> Result<(), String> {
    *self.operation_lock.write().unwrap() = OperationType::Running;
    self.emit_status_change()?; // Notify frontend
    Ok(())
}
```

## Getting Help

- Check the [Data Architecture Reference](../reference/data-architecture.md)
- Review similar implementations in existing code
- Ask in team chat with specific error messages
- Include relevant code snippets and logs

## Additional Resources

- [Tauri Event System](https://tauri.app/v1/guides/features/events/)
- [Rusqlite Documentation](https://docs.rs/rusqlite/)
- [Pinia Documentation](https://pinia.vuejs.org/)
- [Vue Composition API](https://vuejs.org/api/composition-api-setup.html)
