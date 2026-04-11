use rusqlite::{Connection, Result};

/// Run all database migrations in order.
///
/// Each migration is applied exactly once and tracked in `schema_migrations`.
/// The v1 migration is the baseline schema — all tables as of the initial release.
/// New schema changes MUST be added as new numbered migrations (v2, v3, …) below.
/// Never modify migration_v1 after it has shipped — existing user databases already
/// have that schema applied and won't re-run it.
///
/// Example of adding a new migration:
/// ```text
/// if current_version < 2 {
///     migration_v2_add_foo(conn)?;
///     super::record_migration(conn, 2)?;
/// }
/// ```
pub fn run_migrations(conn: &Connection, tz_offset_seconds: Option<i32>) -> Result<()> {
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

    if current_version < 2 {
        migration_v2_skill_base_level(conn)?;
        super::record_migration(conn, 2)?;
    }

    if current_version < 3 {
        migration_v3_fix_skill_levels(conn)?;
        super::record_migration(conn, 3)?;
    }

    if current_version < 4 {
        migration_v4_fix_crafting_project_cascade(conn)?;
        super::record_migration(conn, 4)?;
    }

    if current_version < 5 {
        migration_v5_build_planner(conn)?;
        super::record_migration(conn, 5)?;
    }

    if current_version < 6 {
        migration_v6_build_preset_slot_items(conn)?;
        super::record_migration(conn, 6)?;
    }

    if current_version < 7 {
        migration_v7_build_preset_abilities(conn)?;
        super::record_migration(conn, 7)?;
    }

    if current_version < 8 {
        migration_v8_slot_item_level_rarity(conn)?;
        super::record_migration(conn, 8)?;
    }

    if current_version < 9 {
        migration_v9_slot_per_slot_skills(conn)?;
        super::record_migration(conn, 9)?;
    }

    if current_version < 10 {
        migration_v10_crafting_groups_and_stock_targets(conn)?;
        super::record_migration(conn, 10)?;
    }

    if current_version < 11 {
        migration_v11_character_deaths(conn)?;
        super::record_migration(conn, 11)?;
    }

    if current_version < 12 {
        migration_v12_game_state_area(conn)?;
        super::record_migration(conn, 12)?;
    }

    if current_version < 13 {
        migration_v13_death_damage_type(conn)?;
        super::record_migration(conn, 13)?;
    }

    if current_version < 14 {
        migration_v14_death_damage_sources(conn)?;
        super::record_migration(conn, 14)?;
    }

    if current_version < 15 {
        migration_v15_item_transactions(conn)?;
        super::record_migration(conn, 15)?;
    }

    if current_version < 16 {
        migration_v16_survey_data_imports(conn)?;
        super::record_migration(conn, 16)?;
    }

    if current_version < 17 {
        migration_v17_gift_log(conn)?;
        super::record_migration(conn, 17)?;
    }

    if current_version < 18 {
        migration_v18_fix_timestamps(conn, tz_offset_seconds)?;
        super::record_migration(conn, 18)?;
    }

    if current_version < 19 {
        migration_v19_price_helper(conn)?;
        super::record_migration(conn, 19)?;
    }

    if current_version < 20 {
        migration_v20_project_pricing(conn)?;
        super::record_migration(conn, 20)?;
    }

    if current_version < 21 {
        migration_v21_build_preset_cp_recipes(conn)?;
        super::record_migration(conn, 21)?;
    }

    Ok(())
}

/// Migration V21: CP-consuming recipes (shamanic infusion, crafting enhancements) for build planner.
fn migration_v21_build_preset_cp_recipes(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE build_preset_cp_recipes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            preset_id INTEGER NOT NULL REFERENCES build_presets(id) ON DELETE CASCADE,
            equip_slot TEXT NOT NULL,
            recipe_id INTEGER NOT NULL,
            recipe_name TEXT,
            cp_cost INTEGER NOT NULL,
            effect_type TEXT NOT NULL,
            effect_key TEXT NOT NULL,
            sort_order INTEGER DEFAULT 0
        );
        CREATE INDEX idx_build_preset_cp_recipes_preset ON build_preset_cp_recipes(preset_id);"
    )?;
    Ok(())
}

/// Migration V5: Build planner tables for saving gear/mod builds per character.
fn migration_v5_build_planner(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE build_presets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            character_id TEXT NOT NULL,
            name TEXT NOT NULL,
            skill_primary TEXT,
            skill_secondary TEXT,
            target_level INTEGER DEFAULT 90,
            target_rarity TEXT DEFAULT 'Epic',
            notes TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE build_preset_mods (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            preset_id INTEGER NOT NULL REFERENCES build_presets(id) ON DELETE CASCADE,
            equip_slot TEXT NOT NULL,
            power_name TEXT NOT NULL,
            tier INTEGER,
            is_augment INTEGER DEFAULT 0,
            sort_order INTEGER DEFAULT 0
        );

        CREATE INDEX idx_build_presets_character ON build_presets(character_id);
        CREATE INDEX idx_build_preset_mods_preset ON build_preset_mods(preset_id);",
    )?;
    Ok(())
}

/// Migration V7: Ability bar planning for build presets.
/// Stores which abilities the user wants in each bar (primary, secondary, sidebar).
fn migration_v7_build_preset_abilities(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE build_preset_abilities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            preset_id INTEGER NOT NULL REFERENCES build_presets(id) ON DELETE CASCADE,
            bar TEXT NOT NULL,
            slot_position INTEGER NOT NULL,
            ability_id INTEGER NOT NULL,
            ability_name TEXT
        );

        CREATE INDEX idx_build_preset_abilities_preset ON build_preset_abilities(preset_id);",
    )?;
    Ok(())
}

/// Migration V8: Add per-slot level, rarity, and crafting flags to slot items.
/// Items can now have individual level/rarity instead of using a single global value.
fn migration_v8_slot_item_level_rarity(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "ALTER TABLE build_preset_slot_items ADD COLUMN slot_level INTEGER NOT NULL DEFAULT 90;
         ALTER TABLE build_preset_slot_items ADD COLUMN slot_rarity TEXT NOT NULL DEFAULT 'Epic';
         ALTER TABLE build_preset_slot_items ADD COLUMN is_crafted INTEGER NOT NULL DEFAULT 0;
         ALTER TABLE build_preset_slot_items ADD COLUMN is_masterwork INTEGER NOT NULL DEFAULT 0;",
    )?;
    Ok(())
}

/// Migration V9: Add per-slot skill overrides to build preset slot items.
/// Each slot can now independently choose which two skills its mods come from.
fn migration_v9_slot_per_slot_skills(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "ALTER TABLE build_preset_slot_items ADD COLUMN slot_skill_primary TEXT;
         ALTER TABLE build_preset_slot_items ADD COLUMN slot_skill_secondary TEXT;",
    )?;
    Ok(())
}

/// Migration V10: Add project grouping and per-entry stock targets for crafting projects.
/// group_name allows organizing projects under collapsible group headers.
/// target_stock enables "restock to X" mode where craft quantity auto-calculates from inventory.
fn migration_v10_crafting_groups_and_stock_targets(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "ALTER TABLE crafting_projects ADD COLUMN group_name TEXT DEFAULT NULL;
         ALTER TABLE crafting_project_entries ADD COLUMN target_stock INTEGER DEFAULT NULL;",
    )?;
    Ok(())
}

/// Migration V11: Character deaths tracking table.
/// Records each player death with the killer, ability, damage, and area context.
fn migration_v11_character_deaths(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE character_deaths (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            died_at TEXT NOT NULL,
            killer_name TEXT NOT NULL,
            killer_entity_id TEXT,
            killing_ability TEXT NOT NULL,
            health_damage INTEGER NOT NULL,
            armor_damage INTEGER NOT NULL DEFAULT 0,
            area TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX idx_deaths_char ON character_deaths(character_name, server_name);",
    )?;
    Ok(())
}

/// Migration V12: Track current area per character in game state.
fn migration_v12_game_state_area(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE game_state_area (
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            area_name TEXT NOT NULL,
            last_confirmed_at TEXT NOT NULL,
            PRIMARY KEY (character_name, server_name)
        );",
    )?;
    Ok(())
}

/// Migration V13: Add damage_type column to character_deaths for CDN ability enrichment.
fn migration_v13_death_damage_type(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "ALTER TABLE character_deaths ADD COLUMN damage_type TEXT DEFAULT NULL;",
    )?;
    Ok(())
}

/// Migration V14: Damage sources leading up to each death.
fn migration_v14_death_damage_sources(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE death_damage_sources (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            death_id INTEGER NOT NULL REFERENCES character_deaths(id) ON DELETE CASCADE,
            event_order INTEGER NOT NULL,
            timestamp TEXT NOT NULL,
            attacker_name TEXT NOT NULL,
            attacker_entity_id TEXT,
            ability_name TEXT NOT NULL,
            health_damage INTEGER NOT NULL DEFAULT 0,
            armor_damage INTEGER NOT NULL DEFAULT 0,
            is_crit INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX idx_damage_sources_death ON death_damage_sources(death_id);",
    )?;
    Ok(())
}

/// Migration V15: Item transaction ledger — records every item gain/loss from both
/// Player.log and chat status for historical analysis and cross-source correlation.
fn migration_v15_item_transactions(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE item_transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            item_name TEXT NOT NULL,
            internal_name TEXT,
            item_type_id INTEGER,
            quantity INTEGER NOT NULL,
            context TEXT NOT NULL,
            source TEXT NOT NULL,
            instance_id INTEGER,
            vault_key TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX idx_item_tx_char ON item_transactions(character_name, server_name);
        CREATE INDEX idx_item_tx_item ON item_transactions(item_name);
        CREATE INDEX idx_item_tx_time ON item_transactions(timestamp);",
    )?;
    Ok(())
}

/// Migration V6: Base item selection per equipment slot in build planner.
/// Stores which item the user wants in each slot of a build preset.
fn migration_v6_build_preset_slot_items(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE build_preset_slot_items (
            preset_id INTEGER NOT NULL REFERENCES build_presets(id) ON DELETE CASCADE,
            equip_slot TEXT NOT NULL,
            item_id INTEGER NOT NULL,
            item_name TEXT,
            PRIMARY KEY (preset_id, equip_slot)
        );

        CREATE INDEX idx_build_preset_slot_items_preset ON build_preset_slot_items(preset_id);",
    )?;
    Ok(())
}

/// Migration V4: Remove the ON DELETE CASCADE foreign key from crafting_project_entries.recipe_id.
/// The recipes table is CDN data that gets wiped and reloaded on each CDN update.
/// The CASCADE FK caused all project entries to be deleted whenever CDN data refreshed.
/// We keep the project_id CASCADE (user-owned data) but drop the recipe_id FK entirely,
/// since recipe_name is already denormalized for display purposes.
fn migration_v4_fix_crafting_project_cascade(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE crafting_project_entries_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            recipe_id INTEGER NOT NULL,
            recipe_name TEXT NOT NULL,
            quantity INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0,
            expanded_ingredient_ids TEXT NOT NULL DEFAULT '[]',
            FOREIGN KEY (project_id) REFERENCES crafting_projects(id) ON DELETE CASCADE
        );
        INSERT INTO crafting_project_entries_new
            SELECT * FROM crafting_project_entries;
        DROP TABLE crafting_project_entries;
        ALTER TABLE crafting_project_entries_new RENAME TO crafting_project_entries;
        CREATE INDEX idx_cpe_project ON crafting_project_entries(project_id);",
    )?;
    Ok(())
}

/// Migration V3: Fix skill level data after the incorrect v2 migration.
/// v2 set base_level = level - bonus_levels, but level was still the raw (base) value,
/// so base_level ended up as raw - bonus (wrong). Level was never updated to be the total.
/// Fix: set base_level = level (which is still raw/base), then level = base_level + bonus_levels.
fn migration_v3_fix_skill_levels(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "UPDATE game_state_skills SET base_level = level;
         UPDATE game_state_skills SET level = base_level + bonus_levels;",
    )?;
    Ok(())
}

/// Migration V2: Add base_level column to game_state_skills.
/// `level` stores the total (base + bonus) — what the game displays.
/// `base_level` stores level without bonuses — used for XP table indexing.
fn migration_v2_skill_base_level(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "ALTER TABLE game_state_skills ADD COLUMN base_level INTEGER NOT NULL DEFAULT 0;
         -- Existing `level` column stores the base (raw=). Copy it to base_level,
         -- then update level to be the total (base + bonus).
         UPDATE game_state_skills SET base_level = level;
         UPDATE game_state_skills SET level = level + bonus_levels;",
    )?;
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
            internal_name TEXT,
            description TEXT,
            icon_id INTEGER,
            skill TEXT,
            level_req REAL,
            keywords TEXT,
            damage_type TEXT,
            reset_time REAL,
            target TEXT,
            prerequisite TEXT,
            upgrade_of TEXT,
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

        -- Market Values (user-specified player-to-player prices)
        CREATE TABLE market_values (
            server_name TEXT NOT NULL,
            item_type_id INTEGER NOT NULL,
            item_name TEXT NOT NULL,
            market_value INTEGER NOT NULL,
            notes TEXT,
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (server_name, item_type_id)
        );
        CREATE INDEX idx_market_values_item ON market_values(item_type_id);
        CREATE INDEX idx_market_values_name ON market_values(item_name);

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
            event_type TEXT NOT NULL CHECK (event_type IN ('session_start', 'completed', 'map_crafted', 'survey_used', 'motherlode_completed')),
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
            FOREIGN KEY (item_snapshot_id) REFERENCES character_item_snapshots(id) ON DELETE CASCADE
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

        -- Active quests per snapshot (from /outputcharacter ActiveQuests + ActiveWorkOrders + CompletedWorkOrders)
        CREATE TABLE character_active_quests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            snapshot_id INTEGER NOT NULL,
            quest_key TEXT NOT NULL,
            category TEXT NOT NULL DEFAULT 'active',
            FOREIGN KEY (snapshot_id) REFERENCES character_snapshots(id) ON DELETE CASCADE
        );
        CREATE INDEX idx_active_quests_snapshot ON character_active_quests(snapshot_id);
        CREATE INDEX idx_active_quests_key ON character_active_quests(quest_key, snapshot_id);

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
            skill_id INTEGER NOT NULL,
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
            npc_key TEXT NOT NULL,
            npc_name TEXT NOT NULL,
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

        -- ============================================================
        -- GAME STATE TABLES (last-known-value, per character+server)
        -- ============================================================

        -- Known servers
        CREATE TABLE servers (
            server_name TEXT PRIMARY KEY,
            display_name TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Session singleton: tracks active character and last login
        CREATE TABLE game_state_session (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            last_login_at TEXT,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Skills: one row per skill per character+server
        CREATE TABLE game_state_skills (
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            skill_id INTEGER NOT NULL,
            skill_name TEXT NOT NULL,
            level INTEGER NOT NULL,
            bonus_levels INTEGER NOT NULL DEFAULT 0,
            xp INTEGER NOT NULL DEFAULT 0,
            tnl INTEGER NOT NULL DEFAULT 0,
            max_level INTEGER NOT NULL DEFAULT 0,
            last_confirmed_at TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT 'log',
            PRIMARY KEY (character_name, server_name, skill_id)
        );
        CREATE INDEX idx_gs_skills_char ON game_state_skills(character_name, server_name);

        -- Active combat skills: single row per character+server
        CREATE TABLE game_state_active_skills (
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            skill1_id INTEGER NOT NULL,
            skill1_name TEXT NOT NULL,
            skill2_id INTEGER NOT NULL,
            skill2_name TEXT NOT NULL,
            last_confirmed_at TEXT NOT NULL,
            PRIMARY KEY (character_name, server_name)
        );

        -- Attributes: one row per attribute per character+server
        CREATE TABLE game_state_attributes (
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            attribute_name TEXT NOT NULL,
            value REAL NOT NULL,
            last_confirmed_at TEXT NOT NULL,
            PRIMARY KEY (character_name, server_name, attribute_name)
        );
        CREATE INDEX idx_gs_attributes_char ON game_state_attributes(character_name, server_name);

        -- Weather: singleton (world state, not per-character)
        CREATE TABLE game_state_weather (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            weather_name TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1,
            last_confirmed_at TEXT NOT NULL
        );

        -- Combat state: single row per character+server
        CREATE TABLE game_state_combat (
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            in_combat INTEGER NOT NULL DEFAULT 0,
            last_confirmed_at TEXT NOT NULL,
            PRIMARY KEY (character_name, server_name)
        );

        -- Mount state: single row per character+server
        CREATE TABLE game_state_mount (
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            is_mounted INTEGER NOT NULL DEFAULT 0,
            last_confirmed_at TEXT NOT NULL,
            PRIMARY KEY (character_name, server_name)
        );

        -- Inventory: one row per item instance per character+server
        CREATE TABLE game_state_inventory (
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            instance_id INTEGER NOT NULL,
            item_name TEXT NOT NULL,
            item_type_id INTEGER,
            stack_size INTEGER NOT NULL DEFAULT 1,
            slot_index INTEGER NOT NULL DEFAULT -1,
            last_confirmed_at TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT 'log',
            PRIMARY KEY (character_name, server_name, instance_id)
        );
        CREATE INDEX idx_gs_inventory_char ON game_state_inventory(character_name, server_name);
        CREATE INDEX idx_gs_inventory_item ON game_state_inventory(item_name);

        -- Recipes: one row per recipe per character+server (completion count)
        CREATE TABLE game_state_recipes (
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            recipe_id INTEGER NOT NULL,
            completion_count INTEGER NOT NULL DEFAULT 0,
            last_confirmed_at TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT 'log',
            PRIMARY KEY (character_name, server_name, recipe_id)
        );
        CREATE INDEX idx_gs_recipes_char ON game_state_recipes(character_name, server_name);

        -- Equipment: one row per slot per character+server
        CREATE TABLE game_state_equipment (
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            slot TEXT NOT NULL,
            appearance_key TEXT NOT NULL,
            last_confirmed_at TEXT NOT NULL,
            PRIMARY KEY (character_name, server_name, slot)
        );
        CREATE INDEX idx_gs_equipment_char ON game_state_equipment(character_name, server_name);

        -- NPC Favor: cumulative deltas from log + tier from snapshots
        CREATE TABLE game_state_favor (
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            npc_key TEXT NOT NULL,
            npc_name TEXT NOT NULL,
            cumulative_delta REAL NOT NULL DEFAULT 0,
            favor_tier TEXT,
            last_confirmed_at TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT 'log',
            PRIMARY KEY (character_name, server_name, npc_key)
        );
        CREATE INDEX idx_gs_favor_char ON game_state_favor(character_name, server_name);

        -- Currencies: snapshot-only currencies (Gold, Councils, etc.)
        CREATE TABLE game_state_currencies (
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            currency_name TEXT NOT NULL,
            amount REAL NOT NULL DEFAULT 0,
            last_confirmed_at TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT 'snapshot',
            PRIMARY KEY (character_name, server_name, currency_name)
        );
        CREATE INDEX idx_gs_currencies_char ON game_state_currencies(character_name, server_name);

        -- Active effects/buffs
        CREATE TABLE game_state_effects (
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            effect_instance_id INTEGER NOT NULL,
            effect_name TEXT,
            source_entity_id INTEGER NOT NULL DEFAULT 0,
            last_confirmed_at TEXT NOT NULL,
            PRIMARY KEY (character_name, server_name, effect_instance_id)
        );
        CREATE INDEX idx_gs_effects_char ON game_state_effects(character_name, server_name);

        -- Storage vault contents: one row per item per vault per character
        CREATE TABLE game_state_storage (
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            vault_key TEXT NOT NULL,
            instance_id INTEGER NOT NULL,
            item_name TEXT NOT NULL,
            item_type_id INTEGER,
            stack_size INTEGER NOT NULL DEFAULT 1,
            slot_index INTEGER NOT NULL DEFAULT -1,
            last_confirmed_at TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT 'log',
            PRIMARY KEY (character_name, server_name, vault_key, instance_id)
        );
        CREATE INDEX idx_gs_storage_char ON game_state_storage(character_name, server_name);
        CREATE INDEX idx_gs_storage_vault ON game_state_storage(vault_key);

        -- Tracked skills: player-curated list of skills to watch closely
        CREATE TABLE tracked_skills (
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            skill_name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (character_name, server_name, skill_name)
        );
        "
    )?;

    Ok(())
}

/// Migration V17: Gift log — tracks individual gift events for weekly gift-limit tracking.
fn migration_v17_gift_log(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE game_state_gift_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            character_name TEXT NOT NULL,
            server_name TEXT NOT NULL,
            npc_key TEXT NOT NULL,
            npc_name TEXT NOT NULL,
            gifted_at TEXT NOT NULL,
            favor_delta REAL NOT NULL
        );
        CREATE INDEX idx_gs_gift_log_char ON game_state_gift_log(character_name, server_name);
        CREATE INDEX idx_gs_gift_log_npc_week ON game_state_gift_log(character_name, server_name, npc_key, gifted_at);
        "
    )?;

    Ok(())
}

/// Migration V18: Fix all timestamps that were stored with incorrect timezone handling.
///
/// Two bugs existed:
///
/// 1) Player.log timestamps are UTC but were treated as local time. The old code did:
///      stored = actual_utc - tz_offset_seconds
///    For UTC-7 (offset=-25200): stored = utc + 25200 (7 hours too late).
///    Fix: correct = stored + offset  (player_modifier)
///
/// 2) Chat.log timestamps are local time but were stored as-is (treated as UTC).
///    For UTC-7 (offset=-25200): stored = local = utc + (-offset) = utc - 25200
///    The stored value is 7 hours too early.
///    Fix: correct = stored - offset  (chat_modifier, opposite direction)
fn migration_v18_fix_timestamps(conn: &Connection, tz_offset_seconds: Option<i32>) -> Result<()> {
    let offset = match tz_offset_seconds {
        Some(o) if o != 0 => o,
        _ => {
            // No offset known or offset is zero — nothing to fix
            return Ok(());
        }
    };

    // SQLite datetime() accepts a modifier like '+3600 seconds' or '-3600 seconds'
    let player_modifier = format!("{} seconds", offset);
    let chat_modifier = format!("{} seconds", -offset);

    // --- Player.log-derived timestamps (need +offset correction) ---

    // Tables with last_confirmed_at columns
    let last_confirmed_tables = [
        "game_state_skills",
        "game_state_active_skills",
        "game_state_attributes",
        "game_state_weather",
        "game_state_combat",
        "game_state_mount",
        "game_state_inventory",
        "game_state_equipment",
        "game_state_effects",
        "game_state_storage",
        "game_state_recipes",
        "game_state_favor",
    ];

    for table in &last_confirmed_tables {
        conn.execute(
            &format!(
                "UPDATE {} SET last_confirmed_at = datetime(last_confirmed_at, ?1)
                 WHERE last_confirmed_at IS NOT NULL",
                table
            ),
            [&player_modifier],
        )?;
    }

    // game_state_gift_log: gifted_at column
    conn.execute(
        "UPDATE game_state_gift_log SET gifted_at = datetime(gifted_at, ?1)
         WHERE gifted_at IS NOT NULL",
        [&player_modifier],
    )?;

    // item_transactions: Player.log-sourced rows
    conn.execute(
        "UPDATE item_transactions SET timestamp = datetime(timestamp, ?1)
         WHERE timestamp IS NOT NULL AND source != 'chat_status'",
        [&player_modifier],
    )?;

    // survey_session_stats: start_time and end_time
    conn.execute(
        "UPDATE survey_session_stats SET
            start_time = datetime(start_time, ?1),
            end_time = CASE WHEN end_time IS NOT NULL THEN datetime(end_time, ?1) ELSE NULL END
         WHERE start_time IS NOT NULL",
        [&player_modifier],
    )?;

    // survey_events: timestamp
    conn.execute(
        "UPDATE survey_events SET timestamp = datetime(timestamp, ?1)
         WHERE timestamp IS NOT NULL",
        [&player_modifier],
    )?;

    // --- Chat.log-derived timestamps (need -offset correction) ---

    // chat_messages: timestamp column
    conn.execute(
        "UPDATE chat_messages SET timestamp = datetime(timestamp, ?1)
         WHERE timestamp IS NOT NULL",
        [&chat_modifier],
    )?;

    // item_transactions: chat_status-sourced rows
    conn.execute(
        "UPDATE item_transactions SET timestamp = datetime(timestamp, ?1)
         WHERE timestamp IS NOT NULL AND source = 'chat_status'",
        [&chat_modifier],
    )?;

    // character_deaths: died_at (from chat combat events)
    conn.execute(
        "UPDATE character_deaths SET died_at = datetime(died_at, ?1)
         WHERE died_at IS NOT NULL",
        [&chat_modifier],
    )?;

    // death_damage_sources: timestamp (from chat combat events)
    conn.execute(
        "UPDATE death_damage_sources SET timestamp = datetime(timestamp, ?1)
         WHERE timestamp IS NOT NULL",
        [&chat_modifier],
    )?;

    eprintln!(
        "[migration_v18] Fixed timestamps: Player.log correction={}, Chat.log correction={}",
        player_modifier, chat_modifier
    );

    Ok(())
}

/// Migration V19: Price Helper — persistent crafting price quotes.
fn migration_v19_price_helper(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE price_helper_quotes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            notes TEXT NOT NULL DEFAULT '',
            fee_config TEXT NOT NULL DEFAULT '{\"per_craft_fee\":0,\"material_pct\":0,\"material_pct_basis\":\"total\",\"flat_fee\":0}',
            customer_provides TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE price_helper_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            quote_id INTEGER NOT NULL REFERENCES price_helper_quotes(id) ON DELETE CASCADE,
            recipe_id INTEGER NOT NULL,
            recipe_name TEXT NOT NULL,
            quantity INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX idx_price_helper_entries_quote ON price_helper_entries(quote_id);
        "
    )?;

    Ok(())
}

/// Migration V20: Add pricing fields to crafting projects (integrated price helper).
fn migration_v20_project_pricing(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "ALTER TABLE crafting_projects ADD COLUMN fee_config TEXT NOT NULL DEFAULT '{\"per_craft_fee\":0,\"material_pct\":0,\"material_pct_basis\":\"total\",\"flat_fee\":0}';
         ALTER TABLE crafting_projects ADD COLUMN customer_provides TEXT NOT NULL DEFAULT '{}';"
    )?;

    Ok(())
}

/// Migration V16: Survey data sharing — import tracking table + import_id on sessions.
fn migration_v16_survey_data_imports(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE survey_data_imports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            label TEXT NOT NULL,
            source_player TEXT,
            session_count INTEGER NOT NULL DEFAULT 0,
            event_count INTEGER NOT NULL DEFAULT 0,
            imported_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        ALTER TABLE survey_session_stats ADD COLUMN import_id INTEGER
            REFERENCES survey_data_imports(id) ON DELETE CASCADE;

        CREATE INDEX idx_survey_session_stats_import ON survey_session_stats(import_id);
        "
    )?;

    Ok(())
}
