# Advanced Settings Guide

## Overview

The Advanced tab in Settings provides power-user features for managing the database and handling old log files.

## Features

### 1. Parse Old Logs

Parse historical log files without watching them in real-time. Useful for:
- Importing data from backup logs
- Processing logs from different characters
- Analyzing past game sessions

**How to use:**
1. Select the log file in the General tab
2. Switch to Advanced tab
3. Click "Parse Selected Log File"

**Date handling:** Player.log lines carry only `HH:MM:SS` (no date). The solo reparse path uses the file's modification time (converted to UTC) as the date for all events. Dual-log replay derives the date from the chat log filename / first chat timestamp instead. If you copy a Player.log between machines in a way that clobbers the mtime, the resulting rows will be dated accordingly — use dual-log replay (paired with the matching chat log) whenever possible for an authoritative date. See [time.md](../architecture/time.md) for the full rules.

Survey-session start/end times are always recomputed from the first and last attributed event timestamps on session end, so reparsed sessions land with accurate bounds regardless of when the end actually fires.

### 2. Database Statistics

View real-time statistics about your local database:

**Size Information:**
- **Total Size**: Complete database file size on disk
- **CDN Data**: Space used by game data cache (items, recipes, etc.)
- **Player Data**: Space used by your tracked data (prices, sales, surveys)

**Record Counts:**
- Market price observations
- Sales transactions
- Survey sessions
- Event log entries

Click "Refresh Statistics" to update the numbers.

### 3. Force CDN Table Rebuild

Rebuilds all CDN-derived database tables from the currently loaded JSON data.

**When to use:**
- Database corruption suspected in CDN tables
- After manual database modifications
- To fix data inconsistencies

**What it does:**
- Clears all CDN tables (items, recipes, skills, NPCs, quests)
- Repopulates from in-memory game data
- Does NOT affect player data

**Safety:** Your player data (market prices, sales, surveys) is never touched.

### 4. Player Data Cleanup

Two options for managing player data storage:

#### Purge Old Data

Delete records older than a specified number of days.

**Affects:**
- Market prices
- Sales history
- Survey sessions
- Event log

**Steps:**
1. Enter number of days (e.g., 90)
2. Click "Purge Old Data"
3. Review deletion summary
4. Database is automatically compacted (VACUUM)

**Example:** Setting 90 days will delete all records with timestamps before 90 days ago.

#### Purge ALL Player Data

Permanently delete all player data from the database.

**Safety mechanism:** Must check confirmation checkbox to enable button.

**Affects:**
- All market prices
- All sales history
- All survey sessions
- All event logs

**Does NOT affect:**
- CDN data (items, recipes, skills, NPCs)
- Application settings
- Log file paths

### 5. Auto-Purge Settings

Automatically delete old data when the application starts.

**Configuration:**
- Enable/disable auto-purge
- Set age threshold (default: 90 days)

**Behavior:**
- Runs on every app startup
- Silently deletes old records
- Logs deletions to console
- Skips if no old data found

**Use cases:**
- Keep database size manageable
- Only track recent market trends
- Automatic maintenance without manual intervention

**Default:** Disabled (never purges automatically)

## Best Practices

### Regular Maintenance

**Every 1-3 months:**
1. Check database statistics
2. Purge data older than your analysis window (e.g., 90 days for market trends)
3. Review disk space usage

### Before Major Updates

1. Export important data (if you've added export functionality)
2. Note your current CDN version
3. After update, verify data integrity with statistics

### Troubleshooting

**Database growing too large:**
- Enable auto-purge with appropriate days setting
- Manually purge old data
- Check if event logging is too verbose

**CDN data seems incorrect:**
1. Check CDN version in General tab
2. Force refresh CDN data (General tab)
3. Force rebuild CDN tables (Advanced tab)
4. Verify statistics show correct counts

**Player data disappeared:**
- Check if auto-purge is enabled with short timeframe
- Verify purge wasn't accidentally triggered
- Check database statistics for record counts

## Technical Details

### Database Operations

**Purge Performance:**
- Uses indexed queries for fast deletion
- VACUUM reclaims disk space
- May take longer with large datasets (1M+ records)

**CDN Rebuild:**
- Transactional (all-or-nothing)
- Takes 1-5 seconds depending on data size
- Safe to run multiple times

**Statistics Query:**
- Uses SQLite's `dbstat` table for size calculations
- Real-time counts via `SELECT COUNT(*)`
- Minimal performance impact

### Data Retention

**Market Prices:**
- Typically 100-1000 records per frequently tracked item
- 90 days ≈ reasonable for trend analysis
- 365 days ≈ full year of historical data

**Sales History:**
- Depends on your trading volume
- Most players: 50-500 sales/month
- High-volume traders: 1000+ sales/month

**Survey Sessions:**
- Each session: 1 row + N survey results + M loot items
- Active surveyors: 10-50 sessions/week
- Storage impact usually minimal

**Event Log:**
- Most flexible/variable
- Depends on what events you log
- Can grow quickly if logging verbose events

## Settings Storage

Auto-purge settings are stored in browser localStorage:
- `autoPurgeEnabled`: boolean
- `autoPurgeDays`: number

These persist across app restarts and are independent of the database.

## Safety Features

1. **Confirmation required** for purge-all operation
2. **Player data isolated** from CDN data
3. **Transactional operations** prevent partial updates
4. **VACUUM after purge** to reclaim space safely
5. **Error messages** show specific failure reasons

## Future Enhancements

Potential features:
- Export player data to CSV/JSON before purge
- Selective purge by data type (e.g., only market prices)
- Scheduled purges (not just on startup)
- Data archiving (move to separate file instead of delete)
- Backup/restore functionality
