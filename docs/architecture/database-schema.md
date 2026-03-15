# SQLite Database Schema

## Overview

This database serves dual purposes:
1. **CDN Data Cache** - Persistent storage for parsed Project Gorgon CDN data
2. **Player Data** - Market prices, sales history, survey logs, events

## Design Principles

- **Version tracking** - Track CDN version for cache invalidation
- **Normalized relationships** - Foreign keys for data integrity
- **Indexed lookups** - Fast queries on common patterns (name searches, item relationships)
- **JSON columns** - Store complex/flexible data (preferences, raw quest data)
- **Timestamps** - Track when player data was recorded

---

## CDN Data Tables

### `cdn_version`
Tracks the current CDN data version loaded in the database.

```sql
CREATE TABLE cdn_version (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- Singleton table
    version INTEGER NOT NULL,
    loaded_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### `items`
Core item definitions from CDN.

```sql
CREATE TABLE items (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    icon_id INTEGER,
    value REAL,  -- Base value from CDN
    max_stack_size REAL,
    keywords TEXT,  -- JSON array: ["Equipment", "Armor"]
    effect_descs TEXT  -- JSON array of effect descriptions
);

CREATE INDEX idx_items_name ON items(name COLLATE NOCASE);
CREATE INDEX idx_items_icon ON items(icon_id);
```

### `skills`
Skill definitions.

```sql
CREATE TABLE skills (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    icon_id INTEGER,
    xp_table TEXT,
    keywords TEXT  -- JSON array
);

CREATE INDEX idx_skills_name ON skills(name COLLATE NOCASE);
```

### `abilities`
Ability definitions.

```sql
CREATE TABLE abilities (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    icon_id INTEGER,
    skill TEXT,
    level_req INTEGER,
    keywords TEXT  -- JSON array
);

CREATE INDEX idx_abilities_name ON abilities(name COLLATE NOCASE);
CREATE INDEX idx_abilities_skill ON abilities(skill);
```

### `recipes`
Recipe definitions.

```sql
CREATE TABLE recipes (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    skill TEXT,
    skill_level_req INTEGER,
    icon_id INTEGER,
    num_result_items INTEGER,
    action_label TEXT,
    keywords TEXT,  -- JSON array
    shares_name_with_item_id INTEGER,
    result_item_ids TEXT,  -- JSON array of result item IDs
    ingredient_item_ids TEXT  -- JSON array of ingredient item IDs
);

CREATE INDEX idx_recipes_name ON recipes(name COLLATE NOCASE);
CREATE INDEX idx_recipes_skill ON recipes(skill);
```

### `recipe_ingredients`
Recipe ingredient relationships (normalized).

```sql
CREATE TABLE recipe_ingredients (
    recipe_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    stack_size INTEGER NOT NULL,
    chance_to_consume REAL,
    PRIMARY KEY (recipe_id, item_id),
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE,
    FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE
);

CREATE INDEX idx_recipe_ingredients_item ON recipe_ingredients(item_id);
```

### `npcs`
NPC definitions.

```sql
CREATE TABLE npcs (
    key TEXT PRIMARY KEY,  -- Internal key like "Nishika"
    name TEXT NOT NULL,
    area_name TEXT,
    area_description TEXT,
    preferences TEXT  -- JSON array of NpcPreference objects
);

CREATE INDEX idx_npcs_name ON npcs(name COLLATE NOCASE);
CREATE INDEX idx_npcs_area ON npcs(area_name);
```

### `npc_skills`
Skills that NPCs train (many-to-many).

```sql
CREATE TABLE npc_skills (
    npc_key TEXT NOT NULL,
    skill TEXT NOT NULL,
    PRIMARY KEY (npc_key, skill),
    FOREIGN KEY (npc_key) REFERENCES npcs(key) ON DELETE CASCADE
);

CREATE INDEX idx_npc_skills_skill ON npc_skills(skill);
```

### `quests`
Quest definitions (stores raw JSON for flexibility).

```sql
CREATE TABLE quests (
    internal_name TEXT PRIMARY KEY,
    raw_data TEXT NOT NULL  -- Full JSON of QuestData
);
```

---

## Player Data Tables

### `vendor_prices`
Vendor sell prices (overrides item.value when vendor is specified).

```sql
CREATE TABLE vendor_prices (
    npc_key TEXT NOT NULL,
    item_id INTEGER NOT NULL,
    sell_price REAL NOT NULL,
    currency TEXT DEFAULT 'Councils',  -- Future: favor, etc.
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (npc_key, item_id, currency),
    FOREIGN KEY (npc_key) REFERENCES npcs(key) ON DELETE CASCADE,
    FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE
);

CREATE INDEX idx_vendor_prices_item ON vendor_prices(item_id);
CREATE INDEX idx_vendor_prices_npc ON vendor_prices(npc_key);
```

### `market_prices`
Player market price observations (player bazaar, player vendors).

```sql
CREATE TABLE market_prices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id INTEGER NOT NULL,
    price REAL NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    vendor_type TEXT CHECK (vendor_type IN ('bazaar', 'player_vendor', 'work_order')),
    vendor_name TEXT,  -- Player name or location
    observed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    notes TEXT,  -- User notes about the listing
    FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE
);

CREATE INDEX idx_market_prices_item ON market_prices(item_id);
CREATE INDEX idx_market_prices_observed ON market_prices(observed_at DESC);
CREATE INDEX idx_market_prices_vendor_type ON market_prices(vendor_type);
```

### `sales_history`
Player's own sales transactions.

```sql
CREATE TABLE sales_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    sale_price REAL NOT NULL,
    sale_method TEXT CHECK (sale_method IN ('vendor', 'bazaar', 'trade', 'consignment')),
    buyer_name TEXT,
    sold_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    notes TEXT,
    FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE
);

CREATE INDEX idx_sales_history_item ON sales_history(item_id);
CREATE INDEX idx_sales_history_sold_at ON sales_history(sold_at DESC);
```

### `survey_sessions`
Survey session tracking (from surveying feature).

```sql
CREATE TABLE survey_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    map_item_id INTEGER NOT NULL,
    map_name TEXT NOT NULL,
    zone TEXT,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    total_surveys INTEGER DEFAULT 0,
    quality_sum INTEGER DEFAULT 0,  -- For calculating average
    best_quality INTEGER,
    completed BOOLEAN DEFAULT 0,
    notes TEXT,
    FOREIGN KEY (map_item_id) REFERENCES items(id) ON DELETE CASCADE
);

CREATE INDEX idx_survey_sessions_map ON survey_sessions(map_item_id);
CREATE INDEX idx_survey_sessions_start ON survey_sessions(start_time DESC);
CREATE INDEX idx_survey_sessions_zone ON survey_sessions(zone);
```

### `survey_results`
Individual survey results within a session.

```sql
CREATE TABLE survey_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    survey_number INTEGER NOT NULL,
    quality INTEGER NOT NULL,
    surveyed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES survey_sessions(id) ON DELETE CASCADE
);

CREATE INDEX idx_survey_results_session ON survey_results(session_id);
CREATE INDEX idx_survey_results_quality ON survey_results(quality);
```

### `survey_loot`
Loot dropped during survey sessions.

```sql
CREATE TABLE survey_loot (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    obtained_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES survey_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE
);

CREATE INDEX idx_survey_loot_session ON survey_loot(session_id);
CREATE INDEX idx_survey_loot_item ON survey_loot(item_id);
```

### `event_log`
General purpose event log for tracking player activities.

```sql
CREATE TABLE event_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,  -- 'craft', 'loot', 'death', 'quest_complete', etc.
    event_data TEXT NOT NULL,  -- JSON payload with event details
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_event_log_type ON event_log(event_type);
CREATE INDEX idx_event_log_created ON event_log(created_at DESC);
```

---

## Supporting Tables

### `item_keywords`
Normalized item keywords for efficient filtering (optional - depends on query patterns).

```sql
CREATE TABLE item_keywords (
    item_id INTEGER NOT NULL,
    keyword TEXT NOT NULL,
    PRIMARY KEY (item_id, keyword),
    FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE
);

CREATE INDEX idx_item_keywords_keyword ON item_keywords(keyword);
```

---

## Migration Strategy

1. **Version 1:** Create all CDN tables
2. **Version 2:** Create player data tables
3. **Future versions:** Add new features as needed

Each migration tracked in:

```sql
CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

---

## Query Patterns

### Common CDN Queries

```sql
-- Find item by name
SELECT * FROM items WHERE name LIKE '%sword%' COLLATE NOCASE LIMIT 20;

-- Get recipes using an item
SELECT r.* FROM recipes r
JOIN recipe_ingredients ri ON r.id = ri.recipe_id
WHERE ri.item_id = 1234;

-- Get recipes producing an item
SELECT * FROM recipes
WHERE result_item_ids LIKE '%[1234]%' OR result_item_ids LIKE '%[1234,%'
   OR result_item_ids LIKE '%,1234]%' OR result_item_ids LIKE '%,1234,%';

-- NPCs that train a skill
SELECT n.* FROM npcs n
JOIN npc_skills ns ON n.key = ns.npc_key
WHERE ns.skill = 'Leatherworking';
```

### Common Player Data Queries

```sql
-- Recent market prices for an item
SELECT * FROM market_prices
WHERE item_id = 1234
ORDER BY observed_at DESC
LIMIT 10;

-- Average market price last 30 days
SELECT AVG(price) as avg_price, COUNT(*) as observations
FROM market_prices
WHERE item_id = 1234
  AND observed_at > datetime('now', '-30 days');

-- Sales history for last week
SELECT s.*, i.name as item_name
FROM sales_history s
JOIN items i ON s.item_id = i.id
WHERE sold_at > datetime('now', '-7 days')
ORDER BY sold_at DESC;

-- Survey session summary
SELECT
    s.map_name,
    s.total_surveys,
    s.quality_sum * 1.0 / s.total_surveys as avg_quality,
    s.best_quality,
    COUNT(DISTINCT sl.item_id) as unique_loot_types
FROM survey_sessions s
LEFT JOIN survey_loot sl ON s.id = sl.session_id
WHERE s.id = 123
GROUP BY s.id;

-- Most profitable items sold
SELECT
    i.name,
    COUNT(*) as times_sold,
    SUM(s.quantity * s.sale_price) as total_revenue,
    AVG(s.sale_price) as avg_price
FROM sales_history s
JOIN items i ON s.item_id = i.id
WHERE s.sold_at > datetime('now', '-30 days')
GROUP BY s.item_id
ORDER BY total_revenue DESC
LIMIT 20;
```

---

## Implementation Notes

### Rust Libraries

- **rusqlite** - Primary SQLite interface
- **refinery** or **sqlx** - Migration management
- **serde_json** - JSON column serialization

### JSON Column Strategy

For columns storing JSON arrays/objects:
- Store as TEXT with `json_valid()` constraint
- Use SQLite JSON functions for queries: `json_extract()`, `json_each()`
- Alternative: Store as serialized strings, deserialize in Rust

### CDN Data Loading Flow

```
1. Check CDN version
2. If version changed OR database empty:
   a. Download CDN files
   b. Begin transaction
   c. Clear old CDN data (DELETE FROM items, etc.)
   d. Parse and INSERT new data
   e. Update cdn_version table
   f. Commit transaction
3. Load data into Arc<RwLock<GameData>> for in-memory queries
4. Player data queries go directly to SQLite (fresher data)
```

### Performance Considerations

- **In-memory cache:** Keep hot data (items, recipes) in `GameData` struct
- **Hybrid approach:** CDN lookups from memory, player data from SQLite
- **Connection pooling:** Use `r2d2` for connection management
- **Prepared statements:** Cache common queries
- **Write-ahead logging (WAL):** Enable for better concurrency

---

## Future Enhancements

- **Full-text search:** SQLite FTS5 for item/recipe descriptions
- **Aggregated statistics:** Materialized views for market trends
- **Character profiles:** Multiple character support
- **Backup/export:** Export player data to JSON
- **Sync:** Cloud sync for player data across devices (future consideration)
