/// Game State Manager — persists derived game state from PlayerEvents to SQLite.
///
/// Follows the SurveySessionTracker pattern: lightweight struct that receives
/// &DbPool per call, called synchronously from the coordinator's event loop.
/// Maintains "last known value" tables, not event logs.

use crate::player_event_parser::PlayerEvent;
use crate::parsers::to_utc_datetime;
use crate::game_data::GameData;
use crate::cdn_commands::GameDataState;
use crate::db::DbPool;
use chrono::{Local, Utc};

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
        startup_log!("Active character set: {} on {} (mode: {})",
            name, server, if self.live_mode { "live" } else { "replay" });
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
        ).ok();

        // Only clear transient state during live tailing — during replay/catch-up
        // we want to accumulate data for all characters, not nuke it on each login.
        if self.live_mode {
            conn.execute("DELETE FROM game_state_inventory WHERE character_name = ?1 AND server_name = ?2",
                rusqlite::params![name, server]).ok();
            conn.execute("DELETE FROM game_state_equipment WHERE character_name = ?1 AND server_name = ?2",
                rusqlite::params![name, server]).ok();
            conn.execute("DELETE FROM game_state_combat WHERE character_name = ?1 AND server_name = ?2",
                rusqlite::params![name, server]).ok();
            conn.execute("DELETE FROM game_state_mount WHERE character_name = ?1 AND server_name = ?2",
                rusqlite::params![name, server]).ok();

            // Reset favor deltas — new session starts accumulating from scratch
            conn.execute(
                "UPDATE game_state_favor SET cumulative_delta = 0, source = CASE WHEN favor_tier IS NOT NULL THEN 'snapshot' ELSE source END WHERE character_name = ?1 AND server_name = ?2",
                rusqlite::params![name, server],
            ).ok();
        }

        eprintln!("[game_state] Active character set to {name} on {server}{}", if self.live_mode { "" } else { " (replay)" });
    }

    /// Process a PlayerEvent and persist derived state to the database.
    /// Returns which domains were updated so the coordinator can notify the frontend.
    pub fn process_event(&self, event: &PlayerEvent, db: &DbPool) -> ProcessResult {
        let character = match &self.active_character {
            Some(c) => c.as_str(),
            None => return ProcessResult { domains_updated: vec![] },
        };

        let server = match &self.active_server {
            Some(s) => s.as_str(),
            None => return ProcessResult { domains_updated: vec![] },
        };

        let conn = match db.get() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[game_state] DB error on process_event: {e}");
                return ProcessResult { domains_updated: vec![] };
            }
        };

        let mut domains = Vec::new();

        // Acquire non-blocking read lock on game data for entity resolution.
        // If CDN data is being refreshed (extremely rare), we skip resolution
        // and fall back to storing raw strings with skill_id = 0.
        let game_data_guard = self.game_data.try_read().ok();

        match event {
            PlayerEvent::SkillsLoaded { timestamp, skills } => {
                let dt = self.to_utc(timestamp);
                // Full skill dump on login — replace all skills for this character+server
                conn.execute("DELETE FROM game_state_skills WHERE character_name = ?1 AND server_name = ?2",
                    rusqlite::params![character, server]).ok();

                let mut stmt = conn.prepare(
                    "INSERT INTO game_state_skills (character_name, server_name, skill_id, skill_name, level, bonus_levels, xp, tnl, max_level, last_confirmed_at, source)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'log')"
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

                        stmt.execute(rusqlite::params![
                            character,
                            server,
                            skill_id,
                            display_name,
                            skill.raw,
                            skill.bonus,
                            skill.xp,
                            skill.tnl,
                            skill.max,
                            dt,
                        ]).ok();
                    }
                }
                domains.push("skills");
            }

            PlayerEvent::ActiveSkillsChanged { timestamp, skill1, skill2 } => {
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

            PlayerEvent::AttributesChanged { timestamp, attributes, .. } => {
                let dt = self.to_utc(timestamp);
                // Batch upsert in a transaction for performance
                conn.execute("BEGIN", []).ok();
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
                                character,
                                server,
                                attr.name,
                                attr.value,
                                dt,
                            ]).ok();
                        }
                    }
                }
                conn.execute("COMMIT", []).ok();
                domains.push("attributes");
            }

            PlayerEvent::WeatherChanged { timestamp, weather_name, is_active } => {
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

            PlayerEvent::CombatStateChanged { timestamp, in_combat } => {
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

            PlayerEvent::MountStateChanged { timestamp, is_mounting, .. } => {
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

            PlayerEvent::ItemAdded { timestamp, item_name, instance_id, slot_index, .. } => {
                let dt = self.to_utc(timestamp);
                conn.execute(
                    "INSERT INTO game_state_inventory (character_name, server_name, instance_id, item_name, stack_size, slot_index, last_confirmed_at, source)
                     VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, 'log')
                     ON CONFLICT(character_name, server_name, instance_id) DO UPDATE SET
                        item_name = excluded.item_name,
                        slot_index = excluded.slot_index,
                        last_confirmed_at = excluded.last_confirmed_at",
                    rusqlite::params![character, server, *instance_id as i64, item_name, slot_index, dt],
                ).ok();
                domains.push("inventory");
            }

            PlayerEvent::ItemStackChanged { timestamp, instance_id, item_name, item_type_id, new_stack_size, .. } => {
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

            PlayerEvent::ItemDeleted { instance_id, .. } => {
                conn.execute(
                    "DELETE FROM game_state_inventory WHERE character_name = ?1 AND server_name = ?2 AND instance_id = ?3",
                    rusqlite::params![character, server, *instance_id as i64],
                ).ok();
                domains.push("inventory");
            }

            PlayerEvent::RecipeUpdated { timestamp, recipe_id, completion_count } => {
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

            PlayerEvent::FavorChanged { timestamp, npc_name, delta, .. } => {
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
                domains.push("favor");
            }

            PlayerEvent::EquipmentChanged { timestamp, equipment, .. } => {
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
                        ]).ok();
                    }
                }
                domains.push("equipment");
            }

            PlayerEvent::EffectsAdded { timestamp, source_entity_id, effect_ids, is_login_batch, .. } => {
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
                        ]).ok();
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

            PlayerEvent::EffectNameUpdated { timestamp, effect_instance_id, display_name, .. } => {
                let dt = self.to_utc(timestamp);
                conn.execute(
                    "UPDATE game_state_effects SET effect_name = ?1, last_confirmed_at = ?2
                     WHERE character_name = ?3 AND server_name = ?4 AND effect_instance_id = ?5",
                    rusqlite::params![display_name, dt, character, server, *effect_instance_id as i64],
                ).ok();
                domains.push("effects");
            }

            PlayerEvent::StorageDeposit { timestamp, vault_key, instance_id, item_name, slot, .. } => {
                if let Some(vk) = vault_key {
                    let dt = self.to_utc(timestamp);
                    // Look up item_type_id from the instance registry (via ItemStackChanged if available)
                    conn.execute(
                        "INSERT INTO game_state_storage (character_name, server_name, vault_key, instance_id, item_name, stack_size, slot_index, last_confirmed_at, source)
                         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, 'log')
                         ON CONFLICT(character_name, server_name, vault_key, instance_id) DO UPDATE SET
                            item_name = excluded.item_name,
                            slot_index = excluded.slot_index,
                            last_confirmed_at = excluded.last_confirmed_at",
                        rusqlite::params![character, server, vk, *instance_id as i64, item_name, slot, dt],
                    ).ok();
                    domains.push("storage");
                }
            }

            PlayerEvent::StorageWithdrawal { vault_key, instance_id, .. } => {
                if let Some(vk) = vault_key {
                    conn.execute(
                        "DELETE FROM game_state_storage WHERE character_name = ?1 AND server_name = ?2 AND vault_key = ?3 AND instance_id = ?4",
                        rusqlite::params![character, server, vk, *instance_id as i64],
                    ).ok();
                    domains.push("storage");
                }
            }

            // Events that don't produce game state updates (yet)
            _ => {}
        }

        ProcessResult { domains_updated: domains }
    }
}
