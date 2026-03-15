# Using the Local Database

## Overview

The application now includes a local SQLite database (`glogger.db`) that provides:

1. **Persistent CDN Data Cache** - All items, recipes, skills, NPCs, etc. are saved to disk
2. **Player Data Storage** - Market prices, sales history, survey logs, and event tracking
3. **Relationship Queries** - Fast lookups for recipes using/producing items, NPCs by skill, etc.

The database is automatically created and migrated on first run.

## Database Location

- **Windows:** `%APPDATA%\com.glogger.dev\glogger.db`
- **macOS:** `~/Library/Application Support/com.glogger.dev/glogger.db`
- **Linux:** `~/.local/share/glogger/glogger.db`

## What Gets Saved Automatically

### CDN Data (Auto-cached)
When the app downloads CDN data, it automatically persists:
- Items (all 28 data files from Project Gorgon CDN)
- Skills & Abilities
- Recipes with ingredients
- NPCs with trained skills
- Quests

The CDN version is tracked, so re-downloads only happen when the version changes.

### Player Data (Manual tracking)
You can log:
- Market price observations
- Sales transactions
- Survey sessions & results
- Event logs

## Available Tauri Commands

### Market Prices

**Add a market price observation:**
```typescript
import { invoke } from '@tauri-apps/api/core'

await invoke('add_market_price', {
  input: {
    item_id: 12345,
    price: 150.0,
    quantity: 1,
    vendor_type: 'bazaar',  // 'bazaar' | 'player_vendor' | 'work_order'
    vendor_name: 'PlayerName',
    notes: 'Good price!'
  }
})
```

**Get recent market prices for an item:**
```typescript
const prices = await invoke('get_market_prices_for_item', {
  item_id: 12345,
  limit: 20
})
// Returns: MarketPriceRecord[]
```

### Sales History

**Record a sale:**
```typescript
await invoke('add_sale', {
  input: {
    item_id: 12345,
    quantity: 5,
    sale_price: 200.0,
    sale_method: 'vendor',  // 'vendor' | 'bazaar' | 'trade' | 'consignment'
    buyer_name: 'BuyerName',
    notes: 'Quick sale'
  }
})
```

**Get recent sales:**
```typescript
const sales = await invoke('get_recent_sales', {
  days: 30,
  limit: 50
})
// Returns: SaleRecord[]
```

### Survey Sessions

**Start a survey session:**
```typescript
const sessionId = await invoke('start_survey_session', {
  input: {
    map_item_id: 5001,
    map_name: 'Serbule Crypt Map',
    zone: 'Serbule Crypt'
  }
})
```

**Add a survey result:**
```typescript
await invoke('add_survey_result', {
  input: {
    session_id: sessionId,
    survey_number: 1,
    quality: 75
  }
})
```

**Add loot from survey:**
```typescript
await invoke('add_survey_loot', {
  input: {
    session_id: sessionId,
    item_id: 12345,
    quantity: 3
  }
})
```

**Complete session:**
```typescript
await invoke('complete_survey_session', {
  session_id: sessionId
})
```

**Get survey history:**
```typescript
const sessions = await invoke('get_survey_sessions', {
  limit: 20
})
// Returns: SurveySessionSummary[]
```

### Event Logging

**Log a generic event:**
```typescript
await invoke('log_event', {
  input: {
    event_type: 'craft',
    event_data: {
      recipe_id: 123,
      result_quality: 'Excellent',
      xp_gained: 500
    }
  }
})
```

**Get recent events:**
```typescript
// All events
const allEvents = await invoke('get_recent_events', {
  limit: 50
})

// Filter by type
const craftEvents = await invoke('get_recent_events', {
  event_type: 'craft',
  limit: 20
})
```

## Database Schema

See [database-schema.md](database-schema.md) for full schema documentation.

## Example: Integrating Survey Tracking

The existing survey feature can now persist data. Here's how to update [SurveySessionCard.vue](../../src/components/Surveying/SurveySessionCard.vue):

```typescript
import { invoke } from '@tauri-apps/api/core'

// When starting a new session
const sessionId = await invoke('start_survey_session', {
  input: {
    map_item_id: mapItem.id,
    map_name: mapItem.name,
    zone: 'Current Zone'  // Get from game state
  }
})

// Store sessionId in component state
currentSessionId.value = sessionId

// Each time a survey completes
await invoke('add_survey_result', {
  input: {
    session_id: currentSessionId.value,
    survey_number: surveyCount.value,
    quality: surveyQuality
  }
})

// When loot drops
await invoke('add_survey_loot', {
  input: {
    session_id: currentSessionId.value,
    item_id: lootItemId,
    quantity: lootQuantity
  }
})

// When session ends
await invoke('complete_survey_session', {
  session_id: currentSessionId.value
})
```

## Performance Notes

- **CDN queries** still use in-memory `Arc<RwLock<GameData>>` for speed
- **Player data queries** go directly to SQLite (always fresh)
- **Connection pooling** supports up to 15 concurrent database connections
- **Indexes** are created for common query patterns (item lookups, date ranges, etc.)

## Future Enhancements

Potential features to add:
- Analytics dashboard showing sales trends
- Item profitability calculator using market data
- Survey session comparison/statistics
- Export player data to CSV/JSON
- Cloud sync for multi-device usage
- Full-text search on items/recipes using SQLite FTS5

## Database Maintenance

The database is currently write-only (no automatic cleanup). If you want to reset:

1. Close the application
2. Delete `glogger.db` from the app data directory
3. Restart - it will be recreated automatically

To keep only recent data, you could add periodic cleanup commands:
```sql
-- Delete market prices older than 90 days
DELETE FROM market_prices WHERE observed_at < datetime('now', '-90 days');

-- Delete completed survey sessions older than 6 months
DELETE FROM survey_sessions WHERE completed = 1 AND end_time < datetime('now', '-6 months');
```

## Troubleshooting

**Database locked errors:**
- The app uses WAL mode and connection pooling to minimize this
- If it persists, check for other processes accessing the DB

**Migration errors:**
- Check console logs for specific migration failures
- The `schema_migrations` table tracks which migrations ran
- You can manually inspect the DB with tools like DB Browser for SQLite

**CDN data not persisting:**
- Check console logs for persistence errors
- Ensure app has write permissions to the data directory
- CDN persistence happens after successful download (check for download errors first)
