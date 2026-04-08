# Stall Tracker - Ruby Proof of Concept

## Overview

This PoC parses Project Gorgon's `Player.log` files to extract player shop (stall) activity data. It identifies shop log entries embedded in `ProcessBook` calls, parses them into structured events, and exports them to CSV.

## Architecture

```
Player.log / Player-prev.log
    ↓
PlayerLog (identifies shop log lines via regex)
    ↓
Shop::Log (extracts individual entries from ProcessBook text)
    ↓
Shop::LogMessage factory (routes to correct subclass)
    ↓
LogMessage subclasses (parse event-specific fields)
    ↓
CollectShopLogItems (deduplicates, groups by player)
    ↓
CreateCsv (exports to CSV)
```

## Event Types

The parser recognizes 6 event types (plus an Unknown fallback):

| Action | Performer | Fields | Example Log Line |
|--------|-----------|--------|-----------------|
| `bought` | Customer | player, item, quantity, price_unit, price_total | `Kork bought Orcish Spell Pouch at a cost of 450 per 1 = 450` |
| `added` | Shop Owner | player, item | `Deradon added Quality Mystic Saddlebag to shop` |
| `removed` | Shop Owner | player, item, quantity | `Deradon removed Decent Horseshoes from shop` |
| `configured` | Shop Owner | player, item, quantity, price_unit, quantity_unit | `Deradon configured Horseshoes to cost 3500 per 1` |
| `visible` | Shop Owner | player, item, quantity, price_unit, quantity_unit | `Deradon made Amazing Reins visible in shop at a cost of 6000 per 1` |
| `collected` | Shop Owner | player, price_total | `Deradon collected 30500 Councils from customer purchases` |

### Sales Events vs Shop Log Events

- **Sales Events**: Only `bought` actions — represent customer purchases (revenue).
- **Shop Log Events**: All event types — the full operational log including inventory management, pricing, and sales.

## Log Line Structure

Shop logs appear in `Player.log` as `ProcessBook` calls:

```
[19:40:40] LocalPlayer: ProcessBook("Today's Shop Logs", "Sat Mar 28 15:39 - Deradon removed Decent Horseshoes from shop\n\nSat Mar 28 15:09 - MrBonq bought Quality Reins at a cost of 4500 per 1 = 4500\n\n...", "PlayerShopLog", "", "", False, False, False, False, False, "")
```

The title can be:
- `"Today's Shop Logs"`
- `"Yesterday's Shop Logs"`
- `"Shop Logs From N days ago"` (where N varies)

Each entry within the content follows the format:
```
Day Month Date HH:MM - [action description]
```

## Data Model

### LogMessage::Base (common fields)

| Field | Type | Description |
|-------|------|-------------|
| `body` | String | Raw log message text |
| `index` | Integer | Position in log (for dedup) |
| `date` | DateTime | Parsed from entry timestamp |
| `message` | String | Text after the timestamp |
| `action` | Symbol | `:bought`, `:added`, `:removed`, `:configured`, `:visible`, `:collected` |
| `player` | String | Character name |
| `item` | String | Item name (nil for `collected`) |
| `quantity` | Integer | Item count (default: 1) |
| `price_unit` | Float | Price per unit (price_total / quantity_unit) |
| `price_total` | Integer | Total gold amount |
| `quantity_unit` | Integer | Bulk size (e.g., "per 2" = 2) |
| `rest` | String | Extra text (e.g., purchase restrictions) |

### CSV Output

Headers: `date, player, action, item, price_unit, quantity, price_total`

Two export modes:
- **Sales CSV** (`{player}-sales.csv`): Only `bought` events
- **Debug CSV** (`{player}-debug.csv`): All events

## Key Business Logic

1. **Price calculation**: `price_unit = price_total / quantity_unit` (supports fractional prices for bulk items)
2. **Owner detection**: First event with `owner_action?` identifies the shop owner
3. **Deduplication**: Uses `body + index` as unique key across multiple log files
4. **Ordering**: Entries are reversed (game logs are newest-first, output is chronological)

## Regex Patterns

### Identifying shop log lines in Player.log

```ruby
/^\[\d{2}:\d{2}:\d{2}\] LocalPlayer: ProcessBook\("Today's Shop Logs",/
/^\[\d{2}:\d{2}:\d{2}\] LocalPlayer: ProcessBook\("Yesterday's Shop Logs",/
/^\[\d{2}:\d{2}:\d{2}\] LocalPlayer: ProcessBook\("Shop Logs From/
```

### Parsing individual events

**Bought:**
```ruby
/\A(?<player>\S+)\s+(?<action>bought)\s+(?<item>.+?)\s?x?(?<quantity>\d+)?\s+at\s+a\s+cost\s+of\s+(?<price_unit>\d+)\s+per\s+(?<quantity_unit>\d+)\s+=\s+(?<price_total>\d+)\z/x
```

**Added:**
```ruby
/\A(?<player>\S+)\s+(?<action>added)\s+(?<item>.+)\s+to\s+shop\z/x
```

**Removed:**
```ruby
/\A(?<player>\S+)\s+(?<action>removed)\s+(?<item>.+?)\s?x?(?<quantity>\d+)?\s+from\s+shop\z/x
```

**Configured:**
```ruby
/\A(?<player>\S+)\s+(?<action>configured)\s+(?<item>.+?)x?(?<quantity>\d+)?\s+to\s+cost\s+(?<price_unit>\d+)\s+per\s+(?<quantity_unit>\d+)\.?\s*(?<rest>.*)\z/x
```

**Made Visible:**
```ruby
/\A(?<player>\S+)\s+made\s+(?<item>.+?)x?(?<quantity>\d+)?\s+(?<action>visible)\s+in\s+shop\s+at\s+a\s+cost\s+of\s+(?<price_unit>\d+)\s+per\s+(?<quantity_unit>\d+)\.?\s*(?<rest>.*)\z/x
```

**Collected:**
```ruby
/\A(?<player>\S+)\s+(?<action>collected)\s+(?<price_total>\d+)\s+Councils\s+from\s+customer\s+purchases\z/x
```

## Running

```bash
cd playground/ruby/stall-tracker
bundle install
bundle exec rspec              # Run all specs
bundle exec ruby bin/create_csvs  # Generate CSV output
```
