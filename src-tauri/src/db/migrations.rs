use rusqlite::{Connection, Result};

/// Run all database migrations
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Create migrations table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    let current_version = super::get_schema_version(conn)?;

    if current_version < 1 {
        migration_v1_unified_schema(conn)?;
        super::record_migration(conn, 1)?;
    }

    Ok(())
}

/// Migration V1: Complete unified schema with all tables
fn migration_v1_unified_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- ============================================================
        -- CDN DATA TABLES (game reference data)
        -- ============================================================

        -- CDN Version tracking
        CREATE TABLE cdn_version (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            version INTEGER NOT NULL,
            loaded_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        -- Items
        CREATE TABLE items (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            icon_id INTEGER,
            value REAL,
            max_stack_size REAL,
            keywords TEXT,
            effect_descs TEXT,
            internal_name TEXT,
            food_desc TEXT,
            equip_slot TEXT,
            num_uses INTEGER,
            skill_reqs TEXT,
            behaviors TEXT,
            bestow_recipes TEXT,
            bestow_ability TEXT,
            bestow_quest TEXT,
            bestow_title INTEGER,
            craft_points INTEGER,
            crafting_target_level INTEGER,
            tsys_profile TEXT,
            raw_json TEXT NOT NULL
        );
        CREATE INDEX idx_items_name ON items(name COLLATE NOCASE);
        CREATE INDEX idx_items_icon ON items(icon_id);
        CREATE INDEX idx_items_equip_slot ON items(equip_slot);
        CREATE INDEX idx_items_food_desc ON items(food_desc);
        CREATE INDEX idx_items_tsys_profile ON items(tsys_profile);

        -- Skills
        CREATE TABLE skills (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            icon_id INTEGER,
            xp_table TEXT,
            keywords TEXT,
            combat BOOLEAN,
            max_bonus_levels INTEGER,
            parents TEXT,
            advancement_table TEXT,
            guest_level_cap INTEGER,
            hide_when_zero BOOLEAN,
            advancement_hints TEXT,
            rewards TEXT,
            reports TEXT,
            raw_json TEXT NOT NULL
        );
        CREATE INDEX idx_skills_name ON skills(name COLLATE NOCASE);

        -- Abilities
        CREATE TABLE abilities (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            icon_id INTEGER,
            skill TEXT,
            level_req REAL,
            keywords TEXT,
            damage_type TEXT,
            reset_time REAL,
            target TEXT,
            prerequisite TEXT,
            is_harmless BOOLEAN,
            animation TEXT,
            special_info TEXT,
            works_underwater BOOLEAN,
            works_while_falling BOOLEAN,
            pve TEXT,
            pvp TEXT,
            mana_cost INTEGER,
            power_cost INTEGER,
            armor_cost INTEGER,
            health_cost INTEGER,
            range REAL,
            raw_json TEXT NOT NULL
        );
        CREATE INDEX idx_abilities_name ON abilities(name COLLATE NOCASE);
        CREATE INDEX idx_abilities_skill ON abilities(skill);
        CREATE INDEX idx_abilities_damage_type ON abilities(damage_type);

        -- Recipes
        CREATE TABLE recipes (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            skill TEXT,
            skill_level_req REAL,
            icon_id INTEGER,
            num_result_items INTEGER,
            action_label TEXT,
            keywords TEXT,
            shares_name_with_item_id INTEGER,
            result_item_ids TEXT,
            ingredient_item_ids TEXT,
            result_effects TEXT,
            usage_delay REAL,
            reward_skill_xp_drop_off_level INTEGER,
            sort_skill TEXT,
            raw_json TEXT NOT NULL
        );
        CREATE INDEX idx_recipes_name ON recipes(name COLLATE NOCASE);
        CREATE INDEX idx_recipes_skill ON recipes(skill);
        CREATE INDEX idx_recipes_sort_skill ON recipes(sort_skill);

        -- Recipe Ingredients (normalized)
        CREATE TABLE recipe_ingredients (
            recipe_id INTEGER NOT NULL,
            item_id INTEGER,
            item_keys TEXT,
            description TEXT,
            stack_size INTEGER NOT NULL,
            chance_to_consume REAL,
            FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE,
            FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE
        );
        CREATE INDEX idx_recipe_ingredients_recipe ON recipe_ingredients(recipe_id);
        CREATE INDEX idx_recipe_ingredients_item ON recipe_ingredients(item_id);

        -- NPCs
        CREATE TABLE npcs (
            key TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            area_name TEXT,
            area_description TEXT,
            preferences TEXT,
            pos TEXT,
            services TEXT,
            raw_json TEXT NOT NULL
        );
        CREATE INDEX idx_npcs_name ON npcs(name COLLATE NOCASE);
        CREATE INDEX idx_npcs_area ON npcs(area_name);

        -- NPC Skills (many-to-many)
        CREATE TABLE npc_skills (
            npc_key TEXT NOT NULL,
            skill TEXT NOT NULL,
            PRIMARY KEY (npc_key, skill),
            FOREIGN KEY (npc_key) REFERENCES npcs(key) ON DELETE CASCADE
        );
        CREATE INDEX idx_npc_skills_skill ON npc_skills(skill);

        -- Quests
        CREATE TABLE quests (
            internal_name TEXT PRIMARY KEY,
            raw_data TEXT NOT NULL
        );

        -- XP Tables
        CREATE TABLE xp_tables (
            id INTEGER PRIMARY KEY,
            internal_name TEXT,
            xp_amounts TEXT,
            raw_json TEXT NOT NULL
        );
        CREATE INDEX idx_xp_tables_name ON xp_tables(internal_name);

        -- TSys Client Info (crafting system definitions)
        CREATE TABLE tsys_client_info (
            key TEXT PRIMARY KEY,
            internal_name TEXT,
            skill TEXT,
            slots TEXT,
            prefix TEXT,
            suffix TEXT,
            tiers TEXT,
            is_unavailable BOOLEAN,
            is_hidden_from_transmutation BOOLEAN,
            raw_json TEXT NOT NULL
        );
        CREATE INDEX idx_tsys_client_info_skill ON tsys_client_info(skill);

        -- Item Uses (recipe cross-reference)
        CREATE TABLE item_uses (
            key TEXT PRIMARY KEY,
            recipes_that_use_item TEXT,
            raw_json TEXT NOT NULL
        );

        -- Areas
        CREATE TABLE areas (
            key TEXT PRIMARY KEY,
            friendly_name TEXT,
            short_friendly_name TEXT,
            raw_json TEXT NOT NULL
        );
        CREATE INDEX idx_areas_friendly_name ON areas(friendly_name);

        -- ============================================================
        -- PLAYER DATA TABLES (user-generated data)
        -- ============================================================

        -- User Characters
        CREATE TABLE user_characters (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            source TEXT NOT NULL CHECK (source IN ('report', 'manual', 'login')),
            is_active BOOLEAN NOT NULL DEFAULT 0,
            latest_report_time TIMESTAMP,
            last_login_time TIMESTAMP,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(character_name, server_name)
        );
        CREATE INDEX idx_user_characters_active ON user_characters(is_active);

        -- Vendor Prices
        CREATE TABLE vendor_prices (
            npc_key TEXT NOT NULL,
            item_id INTEGER NOT NULL,
            sell_price REAL NOT NULL,
            currency TEXT DEFAULT 'Councils',
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (npc_key, item_id, currency),
            FOREIGN KEY (npc_key) REFERENCES npcs(key) ON DELETE CASCADE,
            FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE
        );
        CREATE INDEX idx_vendor_prices_item ON vendor_prices(item_id);
        CREATE INDEX idx_vendor_prices_npc ON vendor_prices(npc_key);

        -- Market Prices
        CREATE TABLE market_prices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            item_id INTEGER NOT NULL,
            price REAL NOT NULL,
            quantity INTEGER NOT NULL DEFAULT 1,
            vendor_type TEXT CHECK (vendor_type IN ('bazaar', 'player_vendor', 'work_order')),
            vendor_name TEXT,
            observed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            notes TEXT,
            FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE
        );
        CREATE INDEX idx_market_prices_item ON market_prices(item_id);
        CREATE INDEX idx_market_prices_observed ON market_prices(observed_at DESC);
        CREATE INDEX idx_market_prices_vendor_type ON market_prices(vendor_type);

        -- Sales History
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

        -- Event Log
        CREATE TABLE event_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            event_type TEXT NOT NULL,
            event_data TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX idx_event_log_type ON event_log(event_type);
        CREATE INDEX idx_event_log_created ON event_log(created_at DESC);

        -- ============================================================
        -- SURVEY DATA TABLES
        -- ============================================================

        -- Pre-parsed survey types (derived from items + recipes during CDN ingestion)
        CREATE TABLE survey_types (
            item_id          INTEGER PRIMARY KEY,
            internal_name    TEXT NOT NULL,
            name             TEXT NOT NULL,
            zone             TEXT,
            icon_id          INTEGER,
            survey_category  TEXT NOT NULL,
            is_motherlode    BOOLEAN NOT NULL DEFAULT 0,
            skill_req_name   TEXT,
            skill_req_level  INTEGER,
            survey_skill_req INTEGER,
            recipe_id        INTEGER,
            survey_xp        REAL,
            survey_xp_first_time REAL,
            crafting_cost    REAL
        );
        CREATE INDEX idx_survey_types_zone ON survey_types(zone);
        CREATE INDEX idx_survey_types_category ON survey_types(survey_category);
        CREATE INDEX idx_survey_types_name ON survey_types(name COLLATE NOCASE);

        -- Survey Session Stats - pre-computed aggregate summary for historical browsing
        CREATE TABLE survey_session_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL DEFAULT 'Survey Session',
            notes TEXT NOT NULL DEFAULT '',
            start_time TIMESTAMP NOT NULL,
            end_time TIMESTAMP,
            maps_started INTEGER NOT NULL DEFAULT 0,
            surveys_located INTEGER NOT NULL DEFAULT 0,
            surveys_completed INTEGER NOT NULL DEFAULT 0,
            surveying_xp_gained INTEGER NOT NULL DEFAULT 0,
            mining_xp_gained INTEGER NOT NULL DEFAULT 0,
            geology_xp_gained INTEGER NOT NULL DEFAULT 0,
            total_revenue INTEGER NOT NULL DEFAULT 0,
            total_cost INTEGER NOT NULL DEFAULT 0,
            total_profit INTEGER NOT NULL DEFAULT 0,
            profit_per_hour INTEGER NOT NULL DEFAULT 0,
            elapsed_seconds INTEGER NOT NULL DEFAULT 0,
            is_manual BOOLEAN DEFAULT 0,
            speed_bonus_count INTEGER NOT NULL DEFAULT 0,
            survey_types_used TEXT,
            maps_used_summary TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX idx_survey_session_stats_start ON survey_session_stats(start_time DESC);
        CREATE INDEX idx_survey_session_stats_created ON survey_session_stats(created_at DESC);

        -- Survey Events - detailed logging of individual survey events
        CREATE TABLE survey_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TIMESTAMP NOT NULL,
            session_id INTEGER,
            event_type TEXT NOT NULL CHECK (event_type IN ('session_start', 'completed', 'map_crafted', 'survey_used')),
            map_type TEXT,
            survey_type TEXT,
            speed_bonus_earned BOOLEAN DEFAULT 0,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (session_id) REFERENCES survey_session_stats(id) ON DELETE CASCADE
        );
        CREATE INDEX idx_survey_events_timestamp ON survey_events(timestamp DESC);
        CREATE INDEX idx_survey_events_session ON survey_events(session_id);
        CREATE INDEX idx_survey_events_type ON survey_events(event_type);

        -- Survey Loot Items - individual items obtained from surveys
        CREATE TABLE survey_loot_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            event_id INTEGER NOT NULL,
            item_id INTEGER,
            item_name TEXT NOT NULL,
            quantity INTEGER NOT NULL DEFAULT 1,
            is_speed_bonus BOOLEAN NOT NULL DEFAULT 0,
            is_primary BOOLEAN NOT NULL DEFAULT 0,
            obtained_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (event_id) REFERENCES survey_events(id) ON DELETE CASCADE,
            FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE SET NULL
        );
        CREATE INDEX idx_survey_loot_items_event ON survey_loot_items(event_id);
        CREATE INDEX idx_survey_loot_items_item ON survey_loot_items(item_id);
        CREATE INDEX idx_survey_loot_items_name ON survey_loot_items(item_name);
        CREATE INDEX idx_survey_loot_items_speed_bonus ON survey_loot_items(is_speed_bonus);
        CREATE INDEX idx_survey_loot_items_obtained ON survey_loot_items(obtained_at DESC);

        -- ============================================================
        -- CHAT DATA TABLES
        -- ============================================================

        -- Chat Messages with deduplication constraint
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
        CREATE INDEX idx_chat_messages_timestamp ON chat_messages(timestamp DESC);
        CREATE INDEX idx_chat_messages_channel ON chat_messages(channel);
        CREATE INDEX idx_chat_messages_sender ON chat_messages(sender);
        CREATE INDEX idx_chat_messages_log_file ON chat_messages(log_file);
        CREATE INDEX idx_chat_messages_created ON chat_messages(created_at DESC);

        -- Unique constraint to prevent duplicate messages
        CREATE UNIQUE INDEX idx_chat_messages_unique
        ON chat_messages(timestamp, channel, sender, message);

        -- Full-text search index for message content
        CREATE VIRTUAL TABLE chat_messages_fts USING fts5(
            message,
            sender,
            content=chat_messages,
            content_rowid=id
        );

        -- Triggers to keep FTS index in sync
        CREATE TRIGGER chat_messages_fts_insert AFTER INSERT ON chat_messages BEGIN
            INSERT INTO chat_messages_fts(rowid, message, sender)
            VALUES (new.id, new.message, new.sender);
        END;

        CREATE TRIGGER chat_messages_fts_delete AFTER DELETE ON chat_messages BEGIN
            DELETE FROM chat_messages_fts WHERE rowid = old.id;
        END;

        CREATE TRIGGER chat_messages_fts_update AFTER UPDATE ON chat_messages BEGIN
            DELETE FROM chat_messages_fts WHERE rowid = old.id;
            INSERT INTO chat_messages_fts(rowid, message, sender)
            VALUES (new.id, new.message, new.sender);
        END;

        -- Chat Item Links - items referenced in chat messages
        CREATE TABLE chat_item_links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            message_id INTEGER NOT NULL,
            raw_text TEXT NOT NULL,
            item_name TEXT NOT NULL,
            skill TEXT,
            item_id INTEGER,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (message_id) REFERENCES chat_messages(id) ON DELETE CASCADE,
            FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE SET NULL
        );
        CREATE INDEX idx_chat_item_links_message ON chat_item_links(message_id);
        CREATE INDEX idx_chat_item_links_item ON chat_item_links(item_id);
        CREATE INDEX idx_chat_item_links_item_name ON chat_item_links(item_name);
        CREATE INDEX idx_chat_item_links_skill ON chat_item_links(skill);

        -- Chat log file tracking (legacy - will be replaced by log_file_positions)
        CREATE TABLE chat_log_files (
            file_path TEXT PRIMARY KEY,
            file_name TEXT NOT NULL,
            file_date TEXT NOT NULL,
            last_position INTEGER NOT NULL DEFAULT 0,
            last_processed TIMESTAMP,
            player_name TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX idx_chat_log_files_date ON chat_log_files(file_date DESC);

        -- ============================================================
        -- LOG FILE POSITION TRACKING (unified for all log types)
        -- ============================================================

        CREATE TABLE log_file_positions (
            file_path TEXT PRIMARY KEY,
            file_type TEXT NOT NULL CHECK (file_type IN ('chat', 'player')),
            last_position INTEGER NOT NULL DEFAULT 0,
            last_modified TIMESTAMP,
            last_processed TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            player_name TEXT,
            metadata TEXT
        );
        CREATE INDEX idx_log_positions_type ON log_file_positions(file_type);
        CREATE INDEX idx_log_positions_processed ON log_file_positions(last_processed DESC);

        -- ============================================================
        -- CHARACTER SNAPSHOT TABLES (from /outputcharacter and /outputitems)
        -- ============================================================

        -- Character snapshots (from /outputcharacter)
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
        CREATE INDEX idx_snapshots_char ON character_snapshots(character_name, snapshot_timestamp DESC);
        CREATE INDEX idx_snapshots_import ON character_snapshots(import_date DESC);

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
        CREATE INDEX idx_item_snapshots_char ON character_item_snapshots(character_name, snapshot_timestamp DESC);
        CREATE INDEX idx_item_snapshots_import ON character_item_snapshots(import_date DESC);

        -- Individual items in snapshots
        CREATE TABLE character_snapshot_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            item_snapshot_id INTEGER NOT NULL,
            type_id INTEGER NOT NULL,
            storage_vault TEXT NOT NULL DEFAULT '',
            is_in_inventory BOOLEAN NOT NULL DEFAULT 0,
            stack_size INTEGER NOT NULL,
            value INTEGER,
            item_name TEXT NOT NULL,
            rarity TEXT,
            slot TEXT,
            level INTEGER,
            is_crafted BOOLEAN NOT NULL DEFAULT 0,
            crafter TEXT,
            durability REAL,
            craft_points INTEGER,
            uses_remaining INTEGER,
            transmute_count INTEGER,
            attuned_to TEXT,
            tsys_powers TEXT,
            tsys_imbue_power TEXT,
            tsys_imbue_power_tier INTEGER,
            pet_husbandry_state TEXT,
            FOREIGN KEY (item_snapshot_id) REFERENCES character_item_snapshots(id) ON DELETE CASCADE,
            FOREIGN KEY (type_id) REFERENCES items(id) ON DELETE SET NULL
        );
        CREATE INDEX idx_snapshot_items_snapshot ON character_snapshot_items(item_snapshot_id);
        CREATE INDEX idx_snapshot_items_vault ON character_snapshot_items(storage_vault);
        CREATE INDEX idx_snapshot_items_type ON character_snapshot_items(type_id);

        -- Recipe completions per snapshot
        CREATE TABLE character_recipe_completions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            snapshot_id INTEGER NOT NULL,
            recipe_key TEXT NOT NULL,
            completions INTEGER NOT NULL,
            FOREIGN KEY (snapshot_id) REFERENCES character_snapshots(id) ON DELETE CASCADE
        );
        CREATE INDEX idx_recipe_completions_snapshot ON character_recipe_completions(snapshot_id);
        CREATE INDEX idx_recipe_completions_key ON character_recipe_completions(recipe_key, snapshot_id);

        -- Character stats per snapshot
        CREATE TABLE character_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            snapshot_id INTEGER NOT NULL,
            stat_key TEXT NOT NULL,
            value REAL NOT NULL,
            FOREIGN KEY (snapshot_id) REFERENCES character_snapshots(id) ON DELETE CASCADE
        );
        CREATE INDEX idx_character_stats_snapshot ON character_stats(snapshot_id);
        CREATE INDEX idx_character_stats_key ON character_stats(stat_key, snapshot_id);

        -- Character currencies per snapshot
        CREATE TABLE character_currencies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            snapshot_id INTEGER NOT NULL,
            currency_key TEXT NOT NULL,
            amount INTEGER NOT NULL,
            FOREIGN KEY (snapshot_id) REFERENCES character_snapshots(id) ON DELETE CASCADE
        );
        CREATE INDEX idx_character_currencies_snapshot ON character_currencies(snapshot_id);
        CREATE INDEX idx_character_currencies_key ON character_currencies(currency_key, snapshot_id);

        -- ============================================================
        -- GOURMAND TRACKER TABLES
        -- ============================================================

        -- Pre-parsed food items (derived from items table during CDN ingestion)
        CREATE TABLE foods (
            item_id     INTEGER PRIMARY KEY,
            name        TEXT NOT NULL,
            icon_id     INTEGER,
            food_category TEXT NOT NULL,
            food_level  INTEGER NOT NULL,
            gourmand_req INTEGER,
            effect_descs TEXT NOT NULL,
            keywords    TEXT NOT NULL,
            value       REAL
        );
        CREATE INDEX idx_foods_category ON foods(food_category);
        CREATE INDEX idx_foods_level ON foods(food_level);
        CREATE INDEX idx_foods_name ON foods(name COLLATE NOCASE);

        -- Last-known gourmand report (single snapshot, overwritten on each import)
        CREATE TABLE gourmand_eaten_foods (
            food_name   TEXT PRIMARY KEY,
            times_eaten INTEGER NOT NULL,
            imported_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- ============================================================
        -- FARMING CALCULATOR TABLES
        -- ============================================================

        -- Farming session summary
        CREATE TABLE farming_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL DEFAULT 'Farming Session',
            notes TEXT NOT NULL DEFAULT '',
            start_time TEXT NOT NULL,
            end_time TEXT,
            elapsed_seconds INTEGER NOT NULL DEFAULT 0,
            total_paused_seconds INTEGER NOT NULL DEFAULT 0,
            vendor_gold INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX idx_farming_sessions_created ON farming_sessions(created_at DESC);

        -- XP gains per skill per session
        CREATE TABLE farming_session_skills (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id INTEGER NOT NULL,
            skill_name TEXT NOT NULL,
            xp_gained INTEGER NOT NULL DEFAULT 0,
            levels_gained INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (session_id) REFERENCES farming_sessions(id) ON DELETE CASCADE
        );
        CREATE INDEX idx_farming_skills_session ON farming_session_skills(session_id);

        -- Net item changes per session
        CREATE TABLE farming_session_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id INTEGER NOT NULL,
            item_name TEXT NOT NULL,
            net_quantity INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (session_id) REFERENCES farming_sessions(id) ON DELETE CASCADE
        );
        CREATE INDEX idx_farming_items_session ON farming_session_items(session_id);

        -- Favor changes per session
        CREATE TABLE farming_session_favors (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id INTEGER NOT NULL,
            npc_name TEXT NOT NULL,
            npc_id INTEGER,
            delta REAL NOT NULL DEFAULT 0,
            FOREIGN KEY (session_id) REFERENCES farming_sessions(id) ON DELETE CASCADE
        );
        CREATE INDEX idx_farming_favors_session ON farming_session_favors(session_id);

        -- ============================================================
        -- CRAFTING HELPER TABLES
        -- ============================================================

        -- Saved crafting projects
        CREATE TABLE crafting_projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            notes TEXT NOT NULL DEFAULT '',
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX idx_crafting_projects_updated ON crafting_projects(updated_at DESC);

        -- Recipes within a project
        CREATE TABLE crafting_project_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            recipe_id INTEGER NOT NULL,
            recipe_name TEXT NOT NULL,
            quantity INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0,
            expanded_ingredient_ids TEXT NOT NULL DEFAULT '[]',
            FOREIGN KEY (project_id) REFERENCES crafting_projects(id) ON DELETE CASCADE,
            FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE
        );
        CREATE INDEX idx_cpe_project ON crafting_project_entries(project_id);
        "
    )?;

    Ok(())
}
