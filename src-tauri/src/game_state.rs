use crate::cdn_commands::GameDataState;
use crate::db::DbPool;
use crate::parsers::to_utc_datetime;
/// Game State Manager — persists derived game state from PlayerEvents to SQLite.
///
/// Follows the SurveySessionTracker pattern: lightweight struct that receives
/// &DbPool per call, called synchronously from the coordinator's event loop.
/// Maintains "last known value" tables, not event logs.
use crate::player_event_parser::{DeleteContext, PlayerEvent};
use chrono::{Local, Utc};
use rusqlite::Connection;

/// Timestamped log line for startup diagnostics.
macro_rules! startup_log {
    ($($arg:tt)*) => {
        eprintln!("[{}] {}", Local::now().format("%H:%M:%S%.3f"), format!($($arg)*));
    };
}

// to_datetime removed — use GameStateManager::to_utc(ts) instead

/// Result of processing a player event
pub struct ProcessResult {
    /// Which data domains were updated (e.g., ["skills", "attributes"])
    pub domains_updated: Vec<&'static str>,
}

/// Tracks active character and writes derived game state to the database.
pub struct GameStateManager {
    active_character: Option<String>,
    active_server: Option<String>,
    /// When true, login events will clear transient state (inventory, equipment, etc.)
    /// Starts false — set to true after the first poll cycle completes (catch-up is done).
    live_mode: bool,
    /// Reference to loaded CDN game data for entity resolution
    game_data: GameDataState,
    /// Timezone offset in seconds from UTC, used to convert Player.log HH:MM:SS to UTC
    timezone_offset_seconds: i32,
}

impl GameStateManager {
    pub fn new(game_data: GameDataState) -> Self {
        Self {
            active_character: None,
            active_server: None,
            live_mode: false,
            game_data,
            timezone_offset_seconds: 0,
        }
    }

    /// Mark that catch-up replay is complete and future logins are live.
    pub fn set_live_mode(&mut self) {
        self.live_mode = true;
    }

    /// Check if we are in live mode (catch-up complete).
    pub fn is_live(&self) -> bool {
        self.live_mode
    }

    /// Debug: get active character name
    pub fn get_active_character(&self) -> Option<&str> {
        self.active_character.as_deref()
    }

    /// Debug: get active server name
    pub fn get_active_server(&self) -> Option<&str> {
        self.active_server.as_deref()
    }

    /// Clear the active character so that events are silently dropped.
    /// Used during catch-up replay to skip events for non-selected characters.
    pub fn clear_active_character(&mut self) {
        self.active_character = None;
    }

    /// Update just the character name without touching the database or server.
    /// Used by Player.log handler which knows the character but not the server.
    pub fn set_active_character_name(&mut self, name: &str) {
        self.active_character = Some(name.to_string());
    }

    /// Update just the server name without touching the database.
    /// Used to seed from persisted settings at startup.
    pub fn set_active_server_name(&mut self, server: &str) {
        self.active_server = Some(server.to_string());
    }

    /// Set the timezone offset (seconds from UTC) for timestamp conversion.
    pub fn set_timezone_offset(&mut self, offset_seconds: i32) {
        self.timezone_offset_seconds = offset_seconds;
    }

    /// Convert a Player.log HH:MM:SS timestamp to a full UTC datetime string.
    fn to_utc(&self, ts: &str) -> String {
        to_utc_datetime(ts, self.timezone_offset_seconds)
    }

    /// Update the active character.
    /// During live mode, clears transient state so the login burst can repopulate.
    /// During replay/catch-up, skips clearing to preserve data built for each character.
    pub fn set_active_character(&mut self, name: &str, server: &str, db: &DbPool) {
        startup_log!(
            "Active character set: {} on {} (mode: {})",
            name,
            server,
            if self.live_mode { "live" } else { "replay" }
        );
        self.active_character = Some(name.to_string());
        self.active_server = Some(server.to_string());

        let conn = match db.get() {
            Ok(c) => c,
            Err(e) => {
                startup_log!("[game_state] DB error on set_active_character: {e}");
                return;
            }
        };

        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Update session singleton
        conn.execute(
            "INSERT INTO game_state_session (id, character_name, server_name, last_login_at, updated_at)
             VALUES (1, ?1, ?2, ?3, ?3)
             ON CONFLICT(id) DO UPDATE SET
                character_name = excluded.character_name,
                server_name = excluded.server_name,
                last_login_at = excluded.last_login_at,
                updated_at = excluded.updated_at",
            rusqlite::params![name, server, now],
        ).ok();

        // Auto-create server record if not exists
        conn.execute(
            "INSERT INTO servers (server_name) VALUES (?1) ON CONFLICT DO NOTHING",
            rusqlite::params![server],
        )
        .ok();

        // Only clear transient state during live tailing — during replay/catch-up
        // we want to accumulate data for all characters, not nuke it on each login.
        if self.live_mode {
            conn.execute(
                "DELETE FROM game_state_inventory WHERE character_name = ?1 AND server_name = ?2",
                rusqlite::params![name, server],
            )
            .ok();
            conn.execute(
                "DELETE FROM game_state_equipment WHERE character_name = ?1 AND server_name = ?2",
                rusqlite::params![name, server],
            )
            .ok();
            conn.execute(
                "DELETE FROM game_state_combat WHERE character_name = ?1 AND server_name = ?2",
                rusqlite::params![name, server],
            )
            .ok();
            conn.execute(
                "DELETE FROM game_state_mount WHERE character_name = ?1 AND server_name = ?2",
                rusqlite::params![name, server],
            )
            .ok();

            // Reset favor deltas — new session starts accumulating from scratch
            conn.execute(
                "UPDATE game_state_favor SET cumulative_delta = 0, source = CASE WHEN favor_tier IS NOT NULL THEN 'snapshot' ELSE source END WHERE character_name = ?1 AND server_name = ?2",
                rusqlite::params![name, server],
            ).ok();
        }

        eprintln!(
            "[game_state] Active character set to {name} on {server}{}",
            if self.live_mode { "" } else { " (replay)" }
        );
    }

    /// Process a batch of PlayerEvents in a single SQLite transaction.
    /// Reduces DB overhead during rapid-fire events (e.g., spam-crafting).
    pub fn process_events_batch(&self, events: &[PlayerEvent], db: &DbPool) -> ProcessResult {
        let character = match &self.active_character {
            Some(c) => c.clone(),
            None => return ProcessResult { domains_updated: vec![] },
        };
        let server = match &self.active_server {
            Some(s) => s.clone(),
            None => return ProcessResult { domains_updated: vec![] },
        };
        let conn = match db.get() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[game_state] DB error on process_events_batch: {e}");
                return ProcessResult { domains_updated: vec![] };
            }
        };
        let game_data_guard = self.game_data.try_read().ok();

        let mut all_domains = Vec::new();
        conn.execute("BEGIN IMMEDIATE", []).ok();
        for event in events {
            let mut domains = Vec::new();
            self.process_event_inner(event, &conn, &character, &server, &game_data_guard, &mut domains);
            all_domains.extend(domains);
        }
        conn.execute("COMMIT", []).ok();

        all_domains.sort_unstable();
        all_domains.dedup();
        ProcessResult { domains_updated: all_domains }
    }

    /// Process a single PlayerEvent. Delegates to the shared inner implementation.
    pub fn process_event(&self, event: &PlayerEvent, db: &DbPool) -> ProcessResult {
        let character = match &self.active_character {
            Some(c) => c.clone(),
            None => return ProcessResult { domains_updated: vec![] },
        };
        let server = match &self.active_server {
            Some(s) => s.clone(),
            None => return ProcessResult { domains_updated: vec![] },
        };
        let conn = match db.get() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[game_state] DB error on process_event: {e}");
                return ProcessResult { domains_updated: vec![] };
            }
        };
        let game_data_guard = self.game_data.try_read().ok();
        let mut domains = Vec::new();
        self.process_event_inner(event, &conn, &character, &server, &game_data_guard, &mut domains);
        ProcessResult { domains_updated: domains }
    }

    /// Inner implementation shared by process_event and process_events_batch.
    fn process_event_inner(
        &self,
        event: &PlayerEvent,
        conn: &rusqlite::Connection,
        character: &str,
        server: &str,
        game_data_guard: &Option<tokio::sync::RwLockReadGuard<'_, crate::game_data::GameData>>,
        domains: &mut Vec<&'static str>,
    ) {

        match event {
            PlayerEvent::SkillsLoaded { timestamp, skills } => {
                let dt = self.to_utc(timestamp);
                // Full skill dump on login — replace all skills for this character+server
                conn.execute(
                    "DELETE FROM game_state_skills WHERE character_name = ?1 AND server_name = ?2",
                    rusqlite::params![character, server],
                )
                .ok();

                let mut stmt = conn.prepare(
                    "INSERT INTO game_state_skills (character_name, server_name, skill_id, skill_name, level, base_level, bonus_levels, xp, tnl, max_level, last_confirmed_at, source)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'log')"
                ).ok();

                if let Some(stmt) = stmt.as_mut() {
                    for skill in skills {
                        // Resolve skill internal name → canonical ID + display name
                        let (skill_id, display_name) = match &game_data_guard {
                            Some(data) => match data.resolve_skill(&skill.skill_type) {
                                Some(info) => (info.id as i64, info.name.clone()),
                                None => (0i64, skill.skill_type.clone()),
                            },
                            None => (0i64, skill.skill_type.clone()),
                        };

                        let total_level = skill.raw as i32 + skill.bonus as i32;
                        stmt.execute(rusqlite::params![
                            character,
                            server,
                            skill_id,
                            display_name,
                            total_level, // level (total = base + bonus)
                            skill.raw,   // base_level (raw, without bonuses)
                            skill.bonus,
                            skill.xp,
                            skill.tnl,
                            skill.max,
                            dt,
                        ])
                        .ok();
                    }
                }
                domains.push("skills");
            }

            PlayerEvent::ActiveSkillsChanged {
                timestamp,
                skill1,
                skill2,
            } => {
                let dt = self.to_utc(timestamp);
                // Resolve skill internal names → IDs + display names
                let (skill1_id, skill1_name) = match &game_data_guard {
                    Some(data) => match data.resolve_skill(skill1) {
                        Some(info) => (info.id as i64, info.name.clone()),
                        None => (0i64, skill1.clone()),
                    },
                    None => (0i64, skill1.clone()),
                };
                let (skill2_id, skill2_name) = match &game_data_guard {
                    Some(data) => match data.resolve_skill(skill2) {
                        Some(info) => (info.id as i64, info.name.clone()),
                        None => (0i64, skill2.clone()),
                    },
                    None => (0i64, skill2.clone()),
                };

                conn.execute(
                    "INSERT INTO game_state_active_skills (character_name, server_name, skill1_id, skill1_name, skill2_id, skill2_name, last_confirmed_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                     ON CONFLICT(character_name, server_name) DO UPDATE SET
                        skill1_id = excluded.skill1_id,
                        skill1_name = excluded.skill1_name,
                        skill2_id = excluded.skill2_id,
                        skill2_name = excluded.skill2_name,
                        last_confirmed_at = excluded.last_confirmed_at",
                    rusqlite::params![character, server, skill1_id, skill1_name, skill2_id, skill2_name, dt],
                ).ok();
                domains.push("active_skills");
            }

            PlayerEvent::AttributesChanged {
                timestamp,
                attributes,
                ..
            } => {
                let dt = self.to_utc(timestamp);
                // Batch upsert in a savepoint for performance (works nested or standalone)
                conn.execute("SAVEPOINT attributes_batch", []).ok();
                {
                    let mut stmt = conn.prepare(
                        "INSERT INTO game_state_attributes (character_name, server_name, attribute_name, value, last_confirmed_at)
                         VALUES (?1, ?2, ?3, ?4, ?5)
                         ON CONFLICT(character_name, server_name, attribute_name) DO UPDATE SET
                            value = excluded.value,
                            last_confirmed_at = excluded.last_confirmed_at"
                    ).ok();

                    if let Some(stmt) = stmt.as_mut() {
                        for attr in attributes {
                            stmt.execute(rusqlite::params![
                                character, server, attr.name, attr.value, dt,
                            ])
                            .ok();
                        }
                    }
                }
                conn.execute("RELEASE attributes_batch", []).ok();
                domains.push("attributes");
            }

            PlayerEvent::WeatherChanged {
                timestamp,
                weather_name,
                is_active,
            } => {
                let dt = self.to_utc(timestamp);
                conn.execute(
                    "INSERT INTO game_state_weather (id, weather_name, is_active, last_confirmed_at)
                     VALUES (1, ?1, ?2, ?3)
                     ON CONFLICT(id) DO UPDATE SET
                        weather_name = excluded.weather_name,
                        is_active = excluded.is_active,
                        last_confirmed_at = excluded.last_confirmed_at",
                    rusqlite::params![weather_name, *is_active as i32, dt],
                ).ok();
                domains.push("weather");
            }

            PlayerEvent::CombatStateChanged {
                timestamp,
                in_combat,
            } => {
                let dt = self.to_utc(timestamp);
                conn.execute(
                    "INSERT INTO game_state_combat (character_name, server_name, in_combat, last_confirmed_at)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(character_name, server_name) DO UPDATE SET
                        in_combat = excluded.in_combat,
                        last_confirmed_at = excluded.last_confirmed_at",
                    rusqlite::params![character, server, *in_combat as i32, dt],
                ).ok();
                domains.push("combat");
            }

            PlayerEvent::MountStateChanged {
                timestamp,
                is_mounting,
                ..
            } => {
                let dt = self.to_utc(timestamp);
                conn.execute(
                    "INSERT INTO game_state_mount (character_name, server_name, is_mounted, last_confirmed_at)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(character_name, server_name) DO UPDATE SET
                        is_mounted = excluded.is_mounted,
                        last_confirmed_at = excluded.last_confirmed_at",
                    rusqlite::params![character, server, *is_mounting as i32, dt],
                ).ok();
                domains.push("mount");
            }

            PlayerEvent::ItemAdded {
                timestamp,
                item_name,
                instance_id,
                slot_index,
                is_new,
            } => {
                let dt = self.to_utc(timestamp);
                // Resolve item_type_id and display name from CDN game data
                let resolved = game_data_guard
                    .as_ref()
                    .and_then(|gd| gd.resolve_item(item_name));
                let item_type_id: Option<i64> = resolved.map(|info| info.id as i64);
                let display_name = resolved.map(|info| info.name.as_str());
                conn.execute(
                    "INSERT INTO game_state_inventory (character_name, server_name, instance_id, item_name, item_type_id, stack_size, slot_index, last_confirmed_at, source)
                     VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, 'log')
                     ON CONFLICT(character_name, server_name, instance_id) DO UPDATE SET
                        item_name = excluded.item_name,
                        item_type_id = COALESCE(excluded.item_type_id, game_state_inventory.item_type_id),
                        slot_index = excluded.slot_index,
                        last_confirmed_at = excluded.last_confirmed_at",
                    rusqlite::params![character, server, *instance_id as i64, item_name, item_type_id, slot_index, dt],
                ).ok();
                // Record transaction (only for genuinely new items, not login reloads)
                if *is_new {
                    Self::record_transaction(
                        &conn,
                        character,
                        server,
                        &dt,
                        display_name.unwrap_or(item_name),
                        Some(item_name),
                        item_type_id,
                        1, // initial stack_size; corrected later by chat or ItemStackChanged
                        "loot",
                        "player_log",
                        Some(*instance_id),
                        None,
                    );
                }
                domains.push("inventory");
            }

            PlayerEvent::ItemStackChanged {
                timestamp,
                instance_id,
                item_name,
                item_type_id,
                new_stack_size,
                ..
            } => {
                let dt = self.to_utc(timestamp);
                // Update existing row, or insert if ItemAdded hasn't arrived yet
                let name = item_name.as_deref().unwrap_or("Unknown Item");
                conn.execute(
                    "INSERT INTO game_state_inventory (character_name, server_name, instance_id, item_name, item_type_id, stack_size, last_confirmed_at, source)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'log')
                     ON CONFLICT(character_name, server_name, instance_id) DO UPDATE SET
                        stack_size = excluded.stack_size,
                        item_type_id = COALESCE(excluded.item_type_id, game_state_inventory.item_type_id),
                        last_confirmed_at = excluded.last_confirmed_at",
                    rusqlite::params![character, server, *instance_id as i64, name, *item_type_id as i32, new_stack_size, dt],
                ).ok();
                domains.push("inventory");
            }

            PlayerEvent::ItemDeleted {
                timestamp,
                instance_id,
                item_name,
                context,
            } => {
                // Look up stack_size before deleting so we can record accurate quantity
                let (del_item_name, del_stack_size): (String, i32) = conn
                    .query_row(
                        "SELECT item_name, stack_size FROM game_state_inventory
                         WHERE character_name = ?1 AND server_name = ?2 AND instance_id = ?3",
                        rusqlite::params![character, server, *instance_id as i64],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .unwrap_or_else(|_| {
                        (
                            item_name.clone().unwrap_or_else(|| "Unknown".to_string()),
                            1,
                        )
                    });

                conn.execute(
                    "DELETE FROM game_state_inventory WHERE character_name = ?1 AND server_name = ?2 AND instance_id = ?3",
                    rusqlite::params![character, server, *instance_id as i64],
                ).ok();

                let tx_context = match context {
                    DeleteContext::StorageTransfer => "storage_deposit",
                    DeleteContext::VendorSale => "vendor_sell",
                    DeleteContext::Consumed => "consumed",
                    DeleteContext::Unknown => "unknown",
                };
                let dt = self.to_utc(timestamp);
                Self::record_transaction(
                    &conn,
                    character,
                    server,
                    &dt,
                    &del_item_name,
                    item_name.as_deref(),
                    None,
                    -del_stack_size,
                    tx_context,
                    "player_log",
                    Some(*instance_id),
                    None,
                );
                domains.push("inventory");
            }

            PlayerEvent::RecipeUpdated {
                timestamp,
                recipe_id,
                completion_count,
            } => {
                let dt = self.to_utc(timestamp);
                conn.execute(
                    "INSERT INTO game_state_recipes (character_name, server_name, recipe_id, completion_count, last_confirmed_at, source)
                     VALUES (?1, ?2, ?3, ?4, ?5, 'log')
                     ON CONFLICT(character_name, server_name, recipe_id) DO UPDATE SET
                        completion_count = excluded.completion_count,
                        last_confirmed_at = excluded.last_confirmed_at,
                        source = excluded.source",
                    rusqlite::params![character, server, recipe_id, completion_count, dt],
                ).ok();
                domains.push("recipes");
            }

            PlayerEvent::FavorChanged {
                timestamp,
                npc_name,
                delta,
                is_gift,
                ..
            } => {
                let dt = self.to_utc(timestamp);
                // Resolve NPC display name → CDN key (e.g., "Kalaba" → "NPC_Kalaba")
                let (npc_key, display_name) = match &game_data_guard {
                    Some(data) => match data.resolve_npc(npc_name) {
                        Some(info) => (info.key.clone(), info.name.clone()),
                        None => (npc_name.clone(), npc_name.clone()),
                    },
                    None => (npc_name.clone(), npc_name.clone()),
                };

                conn.execute(
                    "INSERT INTO game_state_favor (character_name, server_name, npc_key, npc_name, cumulative_delta, last_confirmed_at, source)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'log')
                     ON CONFLICT(character_name, server_name, npc_key) DO UPDATE SET
                        cumulative_delta = game_state_favor.cumulative_delta + excluded.cumulative_delta,
                        npc_name = excluded.npc_name,
                        last_confirmed_at = excluded.last_confirmed_at,
                        source = CASE WHEN game_state_favor.source = 'snapshot' THEN 'both' ELSE 'log' END",
                    rusqlite::params![character, server, npc_key, display_name, *delta as f64, dt],
                ).ok();

                // Log individual gift events for weekly gift-limit tracking
                if *is_gift {
                    conn.execute(
                        "INSERT INTO game_state_gift_log (character_name, server_name, npc_key, npc_name, gifted_at, favor_delta)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        rusqlite::params![character, server, npc_key, display_name, dt, *delta as f64],
                    ).ok();
                }

                domains.push("favor");
            }

            PlayerEvent::EquipmentChanged {
                timestamp,
                equipment,
                ..
            } => {
                let dt = self.to_utc(timestamp);
                // Full equipment state — replace all slots
                conn.execute("DELETE FROM game_state_equipment WHERE character_name = ?1 AND server_name = ?2",
                    rusqlite::params![character, server]).ok();

                let mut stmt = conn.prepare(
                    "INSERT INTO game_state_equipment (character_name, server_name, slot, appearance_key, last_confirmed_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)"
                ).ok();

                if let Some(stmt) = stmt.as_mut() {
                    for slot in equipment {
                        stmt.execute(rusqlite::params![
                            character,
                            server,
                            slot.slot,
                            slot.appearance_key,
                            dt,
                        ])
                        .ok();
                    }
                }
                domains.push("equipment");
            }

            PlayerEvent::EffectsAdded {
                timestamp,
                source_entity_id,
                effect_ids,
                is_login_batch,
                ..
            } => {
                let dt = self.to_utc(timestamp);

                // Login batch = full state dump: clear existing effects first
                if *is_login_batch {
                    conn.execute(
                        "DELETE FROM game_state_effects WHERE character_name = ?1 AND server_name = ?2",
                        rusqlite::params![character, server],
                    ).ok();
                }

                let mut stmt = conn.prepare(
                    "INSERT INTO game_state_effects (character_name, server_name, effect_instance_id, source_entity_id, last_confirmed_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(character_name, server_name, effect_instance_id) DO UPDATE SET
                        source_entity_id = excluded.source_entity_id,
                        last_confirmed_at = excluded.last_confirmed_at"
                ).ok();

                if let Some(stmt) = stmt.as_mut() {
                    for id in effect_ids {
                        stmt.execute(rusqlite::params![
                            character,
                            server,
                            *id as i64,
                            *source_entity_id as i64,
                            dt,
                        ])
                        .ok();
                    }
                }
                domains.push("effects");
            }

            PlayerEvent::EffectsRemoved { .. } => {
                // ProcessRemoveEffects prints opaque System.Int32[] — we can't determine
                // which effects were removed. Emit domain update so frontend can re-query.
                // The stale entries will be cleaned up on next login batch.
                domains.push("effects");
            }

            PlayerEvent::EffectNameUpdated {
                timestamp,
                effect_instance_id,
                display_name,
                ..
            } => {
                let dt = self.to_utc(timestamp);
                conn.execute(
                    "UPDATE game_state_effects SET effect_name = ?1, last_confirmed_at = ?2
                     WHERE character_name = ?3 AND server_name = ?4 AND effect_instance_id = ?5",
                    rusqlite::params![
                        display_name,
                        dt,
                        character,
                        server,
                        *effect_instance_id as i64
                    ],
                )
                .ok();
                domains.push("effects");
            }

            PlayerEvent::StorageDeposit {
                timestamp,
                vault_key,
                instance_id,
                item_name,
                slot,
                ..
            } => {
                if let Some(vk) = vault_key {
                    let dt = self.to_utc(timestamp);
                    let display_name = game_data_guard
                        .as_ref()
                        .and_then(|gd| gd.resolve_item(item_name))
                        .map(|info| info.name.clone());
                    conn.execute(
                        "INSERT INTO game_state_storage (character_name, server_name, vault_key, instance_id, item_name, stack_size, slot_index, last_confirmed_at, source)
                         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, 'log')
                         ON CONFLICT(character_name, server_name, vault_key, instance_id) DO UPDATE SET
                            item_name = excluded.item_name,
                            slot_index = excluded.slot_index,
                            last_confirmed_at = excluded.last_confirmed_at",
                        rusqlite::params![character, server, vk, *instance_id as i64, item_name, slot, dt],
                    ).ok();
                    Self::record_transaction(
                        &conn,
                        character,
                        server,
                        &dt,
                        display_name.as_deref().unwrap_or(item_name),
                        Some(item_name),
                        None,
                        1, // stack_size=1 initially; corrected later
                        "storage_deposit",
                        "player_log",
                        Some(*instance_id),
                        Some(vk),
                    );
                    domains.push("storage");
                }
            }

            PlayerEvent::StorageWithdrawal {
                timestamp,
                vault_key,
                instance_id,
                quantity,
                ..
            } => {
                if let Some(vk) = vault_key {
                    // Look up item name before deleting
                    let stored_name: String = conn
                        .query_row(
                            "SELECT item_name FROM game_state_storage
                             WHERE character_name = ?1 AND server_name = ?2 AND vault_key = ?3 AND instance_id = ?4",
                            rusqlite::params![character, server, vk, *instance_id as i64],
                            |row| row.get(0),
                        )
                        .unwrap_or_else(|_| "Unknown".to_string());

                    conn.execute(
                        "DELETE FROM game_state_storage WHERE character_name = ?1 AND server_name = ?2 AND vault_key = ?3 AND instance_id = ?4",
                        rusqlite::params![character, server, vk, *instance_id as i64],
                    ).ok();

                    let dt = self.to_utc(timestamp);
                    Self::record_transaction(
                        &conn,
                        character,
                        server,
                        &dt,
                        &stored_name,
                        None,
                        None,
                        -(*quantity as i32),
                        "storage_withdraw",
                        "player_log",
                        Some(*instance_id),
                        Some(vk),
                    );
                    domains.push("storage");
                }
            }

            // Events that don't produce game state updates (yet)
            _ => {}
        }
    }

    /// Correct inventory/storage stack sizes using chat status data.
    ///
    /// Player.log's ProcessAddItem always records stack_size=1. The chat log's
    /// "[Status] Item x5 added to inventory." gives us the real quantity.
    /// This finds recent rows with stack_size=1 for the matching item and
    /// updates them, returning which domains were corrected.
    pub fn correct_stack_from_chat(
        &self,
        display_name: &str,
        quantity: u32,
        db: &DbPool,
    ) -> Vec<&'static str> {
        if quantity <= 1 {
            return vec![];
        }

        let (character, server) = match (&self.active_character, &self.active_server) {
            (Some(c), Some(s)) => (c.as_str(), s.as_str()),
            _ => return vec![],
        };

        let conn = match db.get() {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        // Resolve display name to internal name via CDN data
        let internal_name = self
            .game_data
            .try_read()
            .ok()
            .and_then(|gd| gd.resolve_item(display_name).and_then(|info| info.internal_name.clone()));

        let mut domains = Vec::new();

        // Try to correct game_state_inventory — match by internal_name or display_name
        let inv_corrected = correct_stack_in_table(
            &conn,
            "game_state_inventory",
            "character_name = ?1 AND server_name = ?2",
            character,
            server,
            None, // no vault_key filter
            internal_name.as_deref(),
            display_name,
            quantity,
        );
        if inv_corrected {
            domains.push("inventory");
        }

        // Try to correct game_state_storage — same logic, any vault
        let storage_corrected = correct_stack_in_table(
            &conn,
            "game_state_storage",
            "character_name = ?1 AND server_name = ?2",
            character,
            server,
            None,
            internal_name.as_deref(),
            display_name,
            quantity,
        );
        if storage_corrected {
            domains.push("storage");
        }

        domains
    }

    /// Record an item transaction in the ledger.
    fn record_transaction(
        conn: &Connection,
        character: &str,
        server: &str,
        dt: &str,
        item_name: &str,
        internal_name: Option<&str>,
        item_type_id: Option<i64>,
        quantity: i32,
        context: &str,
        source: &str,
        instance_id: Option<u64>,
        vault_key: Option<&str>,
    ) {
        conn.execute(
            "INSERT INTO item_transactions (timestamp, character_name, server_name, item_name, internal_name, item_type_id, quantity, context, source, instance_id, vault_key)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                dt,
                character,
                server,
                item_name,
                internal_name,
                item_type_id,
                quantity,
                context,
                source,
                instance_id.map(|id| id as i64),
                vault_key,
            ],
        )
        .ok();
    }
}

/// Correct a single stack_size=1 row in an inventory/storage table.
/// Returns true if a row was updated.
fn correct_stack_in_table(
    conn: &Connection,
    table: &str,
    base_where: &str,
    character: &str,
    server: &str,
    _vault_key: Option<&str>,
    internal_name: Option<&str>,
    display_name: &str,
    quantity: u32,
) -> bool {
    // Build the item name match: prefer internal_name, fall back to display_name
    let name_to_match = internal_name.unwrap_or(display_name);

    // Find the most recent row with stack_size=1 for this item
    // (rowid ordering gives us recency since rows are inserted in order)
    let query = format!(
        "UPDATE {table} SET stack_size = ?1
         WHERE rowid = (
             SELECT rowid FROM {table}
             WHERE {base_where} AND item_name = ?3 AND stack_size = 1
             ORDER BY rowid DESC LIMIT 1
         )"
    );

    let updated = conn
        .execute(
            &query,
            rusqlite::params![quantity, character, server, name_to_match],
        )
        .unwrap_or(0);

    if updated > 0 {
        eprintln!(
            "[game-state] Corrected {table} stack: {display_name} → {quantity} (matched {name_to_match})"
        );
        return true;
    }

    // If internal_name didn't match and we have a display_name to try as fallback
    if internal_name.is_some() {
        let updated = conn
            .execute(
                &format!(
                    "UPDATE {table} SET stack_size = ?1
                     WHERE rowid = (
                         SELECT rowid FROM {table}
                         WHERE {base_where} AND item_name = ?3 AND stack_size = 1
                         ORDER BY rowid DESC LIMIT 1
                     )"
                ),
                rusqlite::params![quantity, character, server, display_name],
            )
            .unwrap_or(0);
        if updated > 0 {
            eprintln!(
                "[game-state] Corrected {table} stack: {display_name} → {quantity} (matched by display name)"
            );
            return true;
        }
    }

    false
}
