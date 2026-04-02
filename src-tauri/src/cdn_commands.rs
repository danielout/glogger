use chrono::Local;
/// Tauri commands for CDN data management and game data queries.
/// These are the invoke() endpoints the Vue frontend calls.
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;

use serde_json::Value;

use crate::cdn;
use crate::game_data::{
    self, AbilityInfo, AreaInfo, EffectInfo, GameData, ItemInfo, NpcInfo, PlayerTitleInfo,
    QuestInfo, RecipeInfo, SkillInfo, SourceEntry, TsysClientInfo,
};

/// Timestamped log line for startup diagnostics.
macro_rules! startup_log {
    ($($arg:tt)*) => {
        eprintln!("[{}] {}", Local::now().format("%H:%M:%S%.3f"), format!($($arg)*));
    };
}

// ── Managed state ─────────────────────────────────────────────────────────────

/// Shared, hot-swappable game data. Wrapped in Arc<RwLock<>> so commands can
/// read concurrently and the refresh command can swap in a new GameData.
pub type GameDataState = Arc<RwLock<GameData>>;

// ── Directory helpers ─────────────────────────────────────────────────────────

fn data_cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Cannot resolve app data dir: {e}"))?;
    let dir = base.join("data");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot create data dir: {e}"))?;
    Ok(dir)
}

fn icon_cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Cannot resolve app data dir: {e}"))?;
    let dir = base.join("icons");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot create icons dir: {e}"))?;
    Ok(dir)
}

// ── Init (called once at startup, not exposed as a command) ───────────────────

/// Check the CDN version, download if stale, load into managed state.
/// Runs at app startup. Non-fatal: on network failure, loads from cache if available.
pub async fn init_game_data(app: &AppHandle, state: &GameDataState) -> Result<(), String> {
    let data_dir = data_cache_dir(app)?;

    let cached_version = cdn::read_cached_version(&data_dir).await;

    // Try to fetch remote version; fall back to cache-only if offline
    startup_log!("Checking CDN version...");
    let remote_version = match cdn::fetch_remote_version().await {
        Ok(v) => v,
        Err(e) => {
            startup_log!("CDN version check failed (offline?): {e}");
            if let Some(cached) = cached_version {
                startup_log!("Loading game data from cache (v{cached})");
                match game_data::load_from_cache(&data_dir, cached).await {
                    Ok(data) => {
                        *state.write().await = data;
                        return Ok(());
                    }
                    Err(e2) => return Err(format!("Offline and cache load failed: {e2}")),
                }
            } else {
                return Err(format!("Offline and no cache available: {e}"));
            }
        }
    };

    let needs_download = cached_version.map_or(true, |cv| cv != remote_version);

    if needs_download {
        startup_log!("Downloading CDN data v{remote_version}...");
        cdn::download_all_data_files(remote_version, &data_dir).await?;
        cdn::write_cached_version(&data_dir, remote_version).await?;
        startup_log!("CDN download complete");
    } else {
        startup_log!("CDN data up to date (v{remote_version}), loading from cache");
    }

    let data = game_data::load_from_cache(&data_dir, remote_version).await?;
    *state.write().await = data;

    Ok(())
}

// ── CDN management commands ───────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct CacheStatus {
    pub cached_version: Option<u32>,
    pub remote_version: Option<u32>,
    pub up_to_date: bool,
    pub item_count: usize,
    pub skill_count: usize,
}

/// Returns current cache status and whether a CDN update is available.
#[tauri::command]
pub async fn get_cache_status(
    app: AppHandle,
    state: State<'_, GameDataState>,
) -> Result<CacheStatus, String> {
    let data_dir = data_cache_dir(&app)?;
    let cached_version = cdn::read_cached_version(&data_dir).await;
    let remote_version: Option<u32> = cdn::fetch_remote_version().await.ok();

    let data = state.read().await;
    let up_to_date = match (cached_version, remote_version) {
        (Some(c), Some(r)) => c == r,
        _ => false,
    };

    Ok(CacheStatus {
        cached_version,
        remote_version,
        up_to_date,
        item_count: data.items.len(),
        skill_count: data.skills.len(),
    })
}

/// Force a full re-download of CDN data, regardless of version.
#[tauri::command]
pub async fn force_refresh_cdn(
    app: AppHandle,
    state: State<'_, GameDataState>,
) -> Result<CacheStatus, String> {
    let data_dir = data_cache_dir(&app)?;

    let remote_version = cdn::fetch_remote_version().await?;
    cdn::download_all_data_files(remote_version, &data_dir).await?;
    cdn::write_cached_version(&data_dir, remote_version).await?;

    let data = game_data::load_from_cache(&data_dir, remote_version).await?;
    let item_count = data.items.len();
    let skill_count = data.skills.len();
    *state.write().await = data;

    Ok(CacheStatus {
        cached_version: Some(remote_version),
        remote_version: Some(remote_version),
        up_to_date: true,
        item_count,
        skill_count,
    })
}

// ── Unified entity resolve commands ──────────────────────────────────────────
// Each accepts any known reference form (numeric ID, display name, internal
// name, CDN key) and resolves it to the canonical entity.

/// Resolve any reference to an item (numeric ID, display name, or internal name).
#[tauri::command]
pub async fn resolve_item(
    reference: String,
    state: State<'_, GameDataState>,
) -> Result<Option<ItemInfo>, String> {
    Ok(state.read().await.resolve_item(&reference).cloned())
}

/// Resolve a batch of item references.
#[tauri::command]
pub async fn resolve_items_batch(
    references: Vec<String>,
    state: State<'_, GameDataState>,
) -> Result<std::collections::HashMap<String, ItemInfo>, String> {
    let data = state.read().await;
    let mut result = std::collections::HashMap::new();
    for ref_str in references {
        if let Some(item) = data.resolve_item(&ref_str) {
            result.insert(ref_str, item.clone());
        }
    }
    Ok(result)
}

/// Resolve any reference to a skill.
#[tauri::command]
pub async fn resolve_skill(
    reference: String,
    state: State<'_, GameDataState>,
) -> Result<Option<SkillInfo>, String> {
    Ok(state.read().await.resolve_skill(&reference).cloned())
}

/// Resolve any reference to a recipe.
#[tauri::command]
pub async fn resolve_recipe(
    reference: String,
    state: State<'_, GameDataState>,
) -> Result<Option<RecipeInfo>, String> {
    Ok(state.read().await.resolve_recipe(&reference).cloned())
}

/// Resolve any reference to a quest.
#[tauri::command]
pub async fn resolve_quest(
    reference: String,
    state: State<'_, GameDataState>,
) -> Result<Option<QuestInfo>, String> {
    Ok(state.read().await.resolve_quest(&reference).cloned())
}

/// Resolve any reference to an NPC.
#[tauri::command]
pub async fn resolve_npc(
    reference: String,
    state: State<'_, GameDataState>,
) -> Result<Option<NpcInfo>, String> {
    Ok(state.read().await.resolve_npc(&reference).cloned())
}

/// Resolve any reference to an ability (numeric ID or display name).
#[tauri::command]
pub async fn resolve_ability(
    reference: String,
    state: State<'_, GameDataState>,
) -> Result<Option<AbilityInfo>, String> {
    Ok(state.read().await.resolve_ability(&reference).cloned())
}

/// Resolve any reference to an area.
#[tauri::command]
pub async fn resolve_area(
    reference: String,
    state: State<'_, GameDataState>,
) -> Result<Option<AreaInfo>, String> {
    Ok(state.read().await.resolve_area(&reference).cloned())
}

/// Search items whose name contains the query string (case-insensitive).
/// Optional filters: equip_slot (exact match), min/max crafting_target_level.
#[tauri::command]
pub async fn search_items(
    query: String,
    equip_slot: Option<String>,
    level_min: Option<u32>,
    level_max: Option<u32>,
    state: State<'_, GameDataState>,
) -> Result<Vec<ItemInfo>, String> {
    let q = query.to_lowercase();
    let data = state.read().await;
    let mut results: Vec<ItemInfo> = data
        .items
        .values()
        .filter(|item| {
            // Name filter (always applied, but empty query matches all)
            if !q.is_empty() && !item.name.to_lowercase().contains(&q) {
                return false;
            }
            // Equip slot filter (maps build planner slot IDs to CDN equip_slot values)
            if let Some(ref slot) = equip_slot {
                match &item.equip_slot {
                    Some(s) => {
                        let matches = match slot.as_str() {
                            "OffHand" => s == "OffHand" || s == "OffHandShield",
                            "Belt" => s == "Waist",
                            _ => s == slot,
                        };
                        if !matches {
                            return false;
                        }
                    }
                    _ => return false,
                }
            }
            // Level range filter (on crafting_target_level)
            if let Some(min) = level_min {
                match item.crafting_target_level {
                    Some(lvl) if lvl >= min => {}
                    _ => return false,
                }
            }
            if let Some(max) = level_max {
                match item.crafting_target_level {
                    Some(lvl) if lvl <= max => {}
                    _ => return false,
                }
            }
            true
        })
        .cloned()
        .collect();
    results.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(results)
}

/// Return all items whose keywords list contains the given keyword.
#[tauri::command]
pub async fn get_items_by_keyword(
    keyword: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<ItemInfo>, String> {
    let data = state.read().await;
    let mut results: Vec<ItemInfo> = data
        .items
        .values()
        .filter(|item| item.keywords.contains(&keyword))
        .cloned()
        .collect();
    results.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(results)
}

/// Return a sorted list of all distinct keyword values across all items.
#[tauri::command]
pub async fn get_all_item_keywords(state: State<'_, GameDataState>) -> Result<Vec<String>, String> {
    let data = state.read().await;
    let mut keywords: Vec<String> = data
        .items
        .values()
        .flat_map(|item| item.keywords.iter().cloned())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    keywords.sort();
    Ok(keywords)
}

/// Return a sorted list of all distinct equip_slot values across all items.
#[tauri::command]
pub async fn get_equip_slots(state: State<'_, GameDataState>) -> Result<Vec<String>, String> {
    let data = state.read().await;
    let mut slots: Vec<String> = data
        .items
        .values()
        .filter_map(|item| item.equip_slot.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    slots.sort();
    Ok(slots)
}

// ── Skill query commands ──────────────────────────────────────────────────────

/// Get all skills as a flat list.
#[tauri::command]
pub async fn get_all_skills(state: State<'_, GameDataState>) -> Result<Vec<SkillInfo>, String> {
    let data = state.read().await;
    let mut skills: Vec<SkillInfo> = data.skills.values().cloned().collect();
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

/// Get the XP amounts array for a skill's XP table.
/// Returns the cumulative XP needed per level, or empty if not found.
#[tauri::command]
pub async fn get_xp_table_for_skill(
    skill_name: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<u64>, String> {
    let data = state.read().await;

    // Find the skill
    let skill = data
        .skill_by_name(&skill_name)
        .ok_or_else(|| format!("Skill not found: {skill_name}"))?;

    // Get the XP table name
    let table_name = skill
        .xp_table
        .as_deref()
        .ok_or_else(|| format!("Skill {skill_name} has no XP table"))?;

    // Find the matching XP table by internal name
    for table in data.xp_tables.values() {
        if table.internal_name.as_deref() == Some(table_name) {
            return Ok(table.xp_amounts.clone());
        }
    }

    Err(format!(
        "XP table '{table_name}' not found for skill {skill_name}"
    ))
}

// ── Ability query commands ────────────────────────────────────────────────────

/// Get all abilities for a given skill name.
#[tauri::command]
pub async fn get_abilities_for_skill(
    skill: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<AbilityInfo>, String> {
    let data = state.read().await;

    // Abilities store skill as internal name (e.g. "FireMagic"), but callers
    // may pass the display name ("Fire Magic"). Try both.
    let internal_name = data
        .skills
        .values()
        .find(|s| s.name == skill)
        .map(|s| s.internal_name.clone());
    let match_name = internal_name.as_deref().unwrap_or(&skill);

    let mut abilities: Vec<AbilityInfo> = data
        .abilities
        .values()
        .filter(|a| a.skill.as_deref() == Some(match_name))
        .cloned()
        .collect();
    abilities.sort_by(|a, b| {
        a.level
            .unwrap_or(0.0)
            .partial_cmp(&b.level.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(abilities)
}

// ── Recipe query commands ─────────────────────────────────────────────────────

/// Get all recipes that produce a given item ID.
#[tauri::command]
pub async fn get_recipes_for_item(
    item_id: u32,
    state: State<'_, GameDataState>,
) -> Result<Vec<RecipeInfo>, String> {
    let data = state.read().await;
    let results: Vec<RecipeInfo> = data
        .recipes
        .values()
        .filter(|r| r.result_item_ids.contains(&item_id))
        .cloned()
        .collect();
    Ok(results)
}

/// Get all recipes that require a given item ID as an ingredient.
#[tauri::command]
pub async fn get_recipes_using_item(
    item_id: u32,
    state: State<'_, GameDataState>,
) -> Result<Vec<RecipeInfo>, String> {
    let data = state.read().await;
    let results: Vec<RecipeInfo> = data
        .recipes
        .values()
        .filter(|r| r.ingredient_item_ids.contains(&item_id))
        .cloned()
        .collect();
    Ok(results)
}

/// Search recipes by name.
#[tauri::command]
pub async fn search_recipes(
    query: String,
    limit: Option<usize>,
    state: State<'_, GameDataState>,
) -> Result<Vec<RecipeInfo>, String> {
    let data = state.read().await;
    let q = query.to_lowercase();
    let limit = limit.unwrap_or(50);

    let mut results: Vec<RecipeInfo> = data
        .recipes
        .values()
        .filter(|r| r.name.to_lowercase().contains(&q))
        .cloned()
        .collect();

    results.sort_by(|a, b| a.name.cmp(&b.name));
    results.truncate(limit);
    Ok(results)
}

/// Get all recipes for a given skill.
#[tauri::command]
pub async fn get_recipes_for_skill(
    skill: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<RecipeInfo>, String> {
    let data = state.read().await;

    let recipe_ids = data.recipes_by_skill.get(&skill);
    let results: Vec<RecipeInfo> = match recipe_ids {
        Some(ids) => ids
            .iter()
            .filter_map(|id| data.recipes.get(id))
            .cloned()
            .collect(),
        None => vec![],
    };

    Ok(results)
}

// ── Quest query commands ──────────────────────────────────────────────────────

/// Get all quests.
#[tauri::command]
pub async fn get_all_quests(state: State<'_, GameDataState>) -> Result<Vec<QuestInfo>, String> {
    let data = state.read().await;
    let mut results: Vec<QuestInfo> = data.quests.values().cloned().collect();
    results.sort_by(|a, b| {
        // Sort by internal_name for now since we don't have parsed display names yet
        let a_name = a
            .raw
            .get("DisplayName")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let b_name = b
            .raw
            .get("DisplayName")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        a_name.cmp(b_name)
    });
    Ok(results)
}

/// Search quests by name.
#[tauri::command]
pub async fn search_quests(
    query: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<QuestInfo>, String> {
    let data = state.read().await;
    let q = query.to_lowercase();

    let mut results: Vec<QuestInfo> = data
        .quests
        .values()
        .filter(|quest| {
            // Search in DisplayName, InternalName, or Description
            let display_name = quest
                .raw
                .get("DisplayName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();
            let description = quest
                .raw
                .get("Description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();

            display_name.contains(&q) || description.contains(&q)
        })
        .cloned()
        .collect();

    results.sort_by(|a, b| {
        let a_name = a
            .raw
            .get("DisplayName")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let b_name = b
            .raw
            .get("DisplayName")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        a_name.cmp(b_name)
    });

    Ok(results)
}

// ── NPC query commands ────────────────────────────────────────────────────────

/// Get all NPCs.
#[tauri::command]
pub async fn get_all_npcs(state: State<'_, GameDataState>) -> Result<Vec<NpcInfo>, String> {
    let data = state.read().await;
    let mut results: Vec<NpcInfo> = data.npcs.values().cloned().collect();
    results.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(results)
}

/// Search NPCs by name or description.
#[tauri::command]
pub async fn search_npcs(
    query: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<NpcInfo>, String> {
    let data = state.read().await;
    let q = query.to_lowercase();

    let mut results: Vec<NpcInfo> = data
        .npcs
        .values()
        .filter(|npc| {
            let name_match = npc.name.to_lowercase().contains(&q);
            let desc_match = npc
                .desc
                .as_ref()
                .map(|d| d.to_lowercase().contains(&q))
                .unwrap_or(false);
            name_match || desc_match
        })
        .cloned()
        .collect();

    results.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(results)
}

/// Get all NPCs in a specific area.
#[tauri::command]
pub async fn get_npcs_in_area(
    area: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<NpcInfo>, String> {
    let data = state.read().await;

    let mut results: Vec<NpcInfo> = data
        .npcs
        .values()
        .filter(|npc| npc.area_name.as_ref().map(|a| a == &area).unwrap_or(false))
        .cloned()
        .collect();

    results.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(results)
}

// ── Effect query commands ────────────────────────────────────────────────────

/// Search effects by name or description.
#[tauri::command]
pub async fn search_effects(
    query: String,
    limit: Option<usize>,
    state: State<'_, GameDataState>,
) -> Result<Vec<EffectInfo>, String> {
    let data = state.read().await;
    let q = query.to_lowercase();
    let limit = limit.unwrap_or(50);

    let mut results: Vec<EffectInfo> = data
        .effects
        .values()
        .filter(|e| {
            let name_match = e
                .name
                .as_ref()
                .map(|n| n.to_lowercase().contains(&q))
                .unwrap_or(false);
            let desc_match = e
                .desc
                .as_ref()
                .map(|d| d.to_lowercase().contains(&q))
                .unwrap_or(false);
            name_match || desc_match
        })
        .cloned()
        .collect();

    results.sort_by(|a, b| {
        let a_name = a.name.as_deref().unwrap_or("");
        let b_name = b.name.as_deref().unwrap_or("");
        a_name.cmp(b_name)
    });
    results.truncate(limit);
    Ok(results)
}

/// Get a single effect by ID.
#[tauri::command]
pub async fn get_effect(
    id: u32,
    state: State<'_, GameDataState>,
) -> Result<Option<EffectInfo>, String> {
    Ok(state.read().await.effects.get(&id).cloned())
}

// ── Player Title query commands ─────────────────────────────────────────────

/// Get all player titles.
#[tauri::command]
pub async fn get_all_player_titles(
    state: State<'_, GameDataState>,
) -> Result<Vec<PlayerTitleInfo>, String> {
    let data = state.read().await;
    let mut results: Vec<PlayerTitleInfo> = data.player_titles.values().cloned().collect();
    results.sort_by(|a, b| {
        let a_title = a.title.as_deref().unwrap_or("");
        let b_title = b.title.as_deref().unwrap_or("");
        a_title.cmp(b_title)
    });
    Ok(results)
}

/// Search player titles by title text or tooltip.
#[tauri::command]
pub async fn search_player_titles(
    query: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<PlayerTitleInfo>, String> {
    let data = state.read().await;
    let q = query.to_lowercase();

    let mut results: Vec<PlayerTitleInfo> = data
        .player_titles
        .values()
        .filter(|t| {
            let title_match = t
                .title
                .as_ref()
                .map(|n| n.to_lowercase().contains(&q))
                .unwrap_or(false);
            let tooltip_match = t
                .tooltip
                .as_ref()
                .map(|d| d.to_lowercase().contains(&q))
                .unwrap_or(false);
            title_match || tooltip_match
        })
        .cloned()
        .collect();

    results.sort_by(|a, b| {
        let a_title = a.title.as_deref().unwrap_or("");
        let b_title = b.title.as_deref().unwrap_or("");
        a_title.cmp(b_title)
    });
    Ok(results)
}

// ── Icon commands ─────────────────────────────────────────────────────────────

/// Returns the local filesystem path to an icon, fetching it from CDN if not cached.
/// The frontend can use this path with Tauri's asset protocol.
#[tauri::command]
pub async fn get_icon_path(
    icon_id: u32,
    app: AppHandle,
    state: State<'_, GameDataState>,
) -> Result<String, String> {
    let icon_dir = icon_cache_dir(&app)?;
    let version = state.read().await.version;

    if version == 0 {
        return Err("Game data not loaded yet".to_string());
    }

    let path: std::path::PathBuf = cdn::get_or_fetch_icon(version, icon_id, &icon_dir).await?;
    Ok(path.to_string_lossy().to_string())
}

// ── Source query commands ────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct QuestSummary {
    pub key: String,
    pub name: String,
}

#[derive(serde::Serialize)]
pub struct EntitySources {
    pub cdn_sources: Vec<SourceEntry>,
    pub bestowed_by_items: Vec<ItemInfo>,
    pub rewarded_by_quests: Vec<QuestSummary>,
}

/// Get all known sources for an ability.
#[tauri::command]
pub async fn get_ability_sources(
    id: u32,
    state: State<'_, GameDataState>,
) -> Result<EntitySources, String> {
    let data = state.read().await;

    let cdn_sources = data
        .sources
        .abilities
        .get(&id)
        .map(|s| s.entries.clone())
        .unwrap_or_default();

    // Find items that bestow this ability via bestow_ability field.
    // bestow_ability stores the internal name like "ability_1002"
    let ability_key = format!("ability_{id}");
    let bestowed_by_items = data
        .items_bestowing_ability
        .get(&ability_key)
        .map(|item_ids| {
            item_ids
                .iter()
                .filter_map(|iid| data.items.get(iid))
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    Ok(EntitySources {
        cdn_sources,
        bestowed_by_items,
        rewarded_by_quests: vec![],
    })
}

/// Get all known sources for an item.
#[tauri::command]
pub async fn get_item_sources(
    id: u32,
    state: State<'_, GameDataState>,
) -> Result<EntitySources, String> {
    let data = state.read().await;

    let cdn_sources = data
        .sources
        .items
        .get(&id)
        .map(|s| s.entries.clone())
        .unwrap_or_default();

    // Find quests that reward this item
    let item_key = format!("item_{id}");
    let rewarded_by_quests = data
        .quests_rewarding_item
        .get(&item_key)
        .map(|quest_keys| {
            quest_keys
                .iter()
                .filter_map(|qk| {
                    let quest = data.quests.get(qk)?;
                    let name = quest
                        .raw
                        .get("Name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(qk.as_str())
                        .to_string();
                    Some(QuestSummary {
                        key: qk.clone(),
                        name,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(EntitySources {
        cdn_sources,
        bestowed_by_items: vec![],
        rewarded_by_quests,
    })
}

/// Get all known sources for a recipe.
#[tauri::command]
pub async fn get_recipe_sources(
    id: u32,
    state: State<'_, GameDataState>,
) -> Result<EntitySources, String> {
    let data = state.read().await;

    let cdn_sources = data
        .sources
        .recipes
        .get(&id)
        .map(|s| s.entries.clone())
        .unwrap_or_default();

    // Find items that bestow this recipe
    let recipe_key = format!("recipe_{id}");
    let bestowed_by_items = data
        .items_bestowing_recipe
        .get(&recipe_key)
        .map(|item_ids| {
            item_ids
                .iter()
                .filter_map(|iid| data.items.get(iid))
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    Ok(EntitySources {
        cdn_sources,
        bestowed_by_items,
        rewarded_by_quests: vec![],
    })
}

/// Get all known sources for a quest (items that bestow it).
#[tauri::command]
pub async fn get_quest_sources(
    key: String,
    state: State<'_, GameDataState>,
) -> Result<EntitySources, String> {
    let data = state.read().await;

    let bestowed_by_items = data
        .items_bestowing_quest
        .get(&key)
        .map(|item_ids| {
            item_ids
                .iter()
                .filter_map(|iid| data.items.get(iid))
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    Ok(EntitySources {
        cdn_sources: vec![],
        bestowed_by_items,
        rewarded_by_quests: vec![],
    })
}

// ── Effect description resolution ───────────────────────────────────────────

#[derive(serde::Serialize, Clone)]
pub struct ResolvedEffect {
    pub label: String,
    pub value: String,
    pub display_type: String,
    pub formatted: String,
    pub icon_id: Option<u32>,
}

/// Resolve `{TOKEN}{VALUE}` style effect descriptions into human-readable text.
#[tauri::command]
pub async fn resolve_effect_descs(
    descs: Vec<String>,
    state: State<'_, GameDataState>,
) -> Result<Vec<ResolvedEffect>, String> {
    let data = state.read().await;
    let mut results = Vec::new();

    for desc in &descs {
        if let Some(resolved) = resolve_single_effect(desc, &data) {
            results.push(resolved);
        } else {
            // Fallback: return the raw string
            results.push(ResolvedEffect {
                label: desc.clone(),
                value: String::new(),
                display_type: String::new(),
                formatted: desc.clone(),
                icon_id: None,
            });
        }
    }

    Ok(results)
}

/// Strip `<icon=XXXX>` tags from text and return (cleaned_text, first_icon_id)
fn strip_icon_tags(text: &str) -> (String, Option<u32>) {
    let mut first_icon: Option<u32> = None;
    let mut result = String::with_capacity(text.len());
    let mut remaining = text;

    while let Some(start) = remaining.find("<icon=") {
        result.push_str(&remaining[..start]);
        if let Some(end) = remaining[start..].find('>') {
            let icon_str = &remaining[start + 6..start + end];
            if first_icon.is_none() {
                first_icon = icon_str.parse::<u32>().ok();
            }
            remaining = &remaining[start + end + 1..];
        } else {
            // Malformed tag, just keep it
            result.push_str(&remaining[start..start + 6]);
            remaining = &remaining[start + 6..];
        }
    }
    result.push_str(remaining);

    (result, first_icon)
}

fn resolve_single_effect(desc: &str, data: &GameData) -> Option<ResolvedEffect> {
    // Parse "{TOKEN}{VALUE}" format
    let parts: Vec<&str> = desc.split('{').filter(|s| !s.is_empty()).collect();
    if parts.len() >= 2 {
        let token = parts[0].trim_end_matches('}');
        let value_str = parts[1].trim_end_matches('}');

        if let Some(attr) = data.attributes.get(token) {
            if let Some(label) = attr.raw.get("Label").and_then(|v| v.as_str()) {
                let label = label.to_string();
                let display_type = attr
                    .raw
                    .get("DisplayType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("AsInt")
                    .to_string();
                let icon_id = attr
                    .raw
                    .get("IconIds")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32);

                let value: f64 = value_str.parse().unwrap_or(0.0);
                let formatted = format_effect_value(&label, value, &display_type);

                return Some(ResolvedEffect {
                    label,
                    value: value_str.to_string(),
                    display_type,
                    formatted,
                    icon_id,
                });
            }
        }
    }

    // Handle "<icon=XXXX>text" format (pre-rendered effect descriptions)
    if desc.contains("<icon=") {
        let (cleaned, icon_id) = strip_icon_tags(desc);
        let cleaned = cleaned.trim().to_string();
        return Some(ResolvedEffect {
            label: cleaned.clone(),
            value: String::new(),
            display_type: String::new(),
            formatted: cleaned,
            icon_id,
        });
    }

    None
}

fn format_effect_value(label: &str, value: f64, display_type: &str) -> String {
    match display_type {
        "AsPercent" => format!("{} +{}%", label, (value * 100.0).round() as i64),
        "AsBuffDelta" => format!("{} +{}", label, value.round() as i64),
        "AsDebuffDelta" => format!("{} {}", label, value.round() as i64),
        "AsBuffMod" => format!("{} +{}%", label, (value * 100.0).round() as i64),
        "AsDebuffMod" => format!("{} -{}%", label, (value * 100.0).abs().round() as i64),
        "AsInt" => format!("{} +{}", label, value.round() as i64),
        "AsDouble" | "AsDoubleTimes100" => {
            if value.fract() == 0.0 {
                format!("{} +{}", label, value as i64)
            } else {
                format!("{} +{:.2}", label, value)
            }
        }
        "AsBool" => label.to_string(),
        _ => format!("{} {}", label, value),
    }
}

// ── TSys power lookup ───────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct TsysPowerInfo {
    pub internal_name: String,
    pub skill: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub slots: Vec<String>,
    pub tier_effects: Vec<String>,
    /// Structured effect data for aggregation (label, numeric value, display_type)
    pub tier_effects_structured: Vec<ResolvedEffect>,
    /// Icon ID from the first effect's attribute definition
    pub icon_id: Option<u32>,
}

/// Look up a TSys power by internal name and tier, returning human-readable info.
#[tauri::command]
pub async fn get_tsys_power_info(
    power_name: String,
    tier: i64,
    state: State<'_, GameDataState>,
) -> Result<Option<TsysPowerInfo>, String> {
    let data = state.read().await;

    // Find the power in tsys client_info by internal_name
    let entry = data
        .tsys
        .client_info
        .values()
        .find(|info| info.internal_name.as_deref() == Some(&power_name));

    let Some(info) = entry else {
        return Ok(None);
    };

    // Get the effect descriptions for the requested tier
    let tier_key = format!("id_{}", tier);
    let mut tier_effects = Vec::new();
    let mut tier_effects_structured = Vec::new();
    let mut icon_id: Option<u32> = None;

    if let Some(descs) = info
        .tiers
        .as_ref()
        .and_then(|tiers| tiers.get(&tier_key))
        .and_then(|t| t.get("EffectDescs"))
        .and_then(|descs| descs.as_array())
    {
        for desc_val in descs {
            if let Some(desc) = desc_val.as_str() {
                if let Some(resolved) = resolve_single_effect(desc, &data) {
                    if icon_id.is_none() {
                        icon_id = resolved.icon_id;
                    }
                    tier_effects.push(resolved.formatted.clone());
                    tier_effects_structured.push(resolved);
                } else {
                    tier_effects.push(desc.to_string());
                    tier_effects_structured.push(ResolvedEffect {
                        label: desc.to_string(),
                        value: String::new(),
                        display_type: String::new(),
                        formatted: desc.to_string(),
                        icon_id: None,
                    });
                }
            }
        }
    }

    // Resolve skill internal name to display name
    let resolved_skill = info.skill.as_deref().and_then(|s| {
        data.resolve_skill(s).map(|si| si.name.clone())
    }).or_else(|| info.skill.clone());

    Ok(Some(TsysPowerInfo {
        internal_name: power_name,
        skill: resolved_skill,
        prefix: info.prefix.clone(),
        suffix: info.suffix.clone(),
        slots: info.slots.clone(),
        tier_effects,
        tier_effects_structured,
        icon_id,
    }))
}

// ── Storage Vault query commands ─────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct StorageVaultZoneInfo {
    pub vault_key: String,
    pub area_key: Option<String>,
    pub area_name: Option<String>,
    pub npc_friendly_name: Option<String>,
    pub num_slots: Option<u32>,
}

/// Get zone/area mappings for all storage vaults.
#[tauri::command]
pub async fn get_storage_vault_zones(
    state: State<'_, GameDataState>,
) -> Result<Vec<StorageVaultZoneInfo>, String> {
    let data = state.read().await;
    let mut results: Vec<StorageVaultZoneInfo> = data
        .storage_vaults
        .iter()
        .map(|(key, vault)| {
            let area_key = vault.area.clone();
            let area_name = area_key
                .as_ref()
                .and_then(|ak| data.areas.get(ak))
                .and_then(|a| a.short_friendly_name.clone().or(a.friendly_name.clone()));

            StorageVaultZoneInfo {
                vault_key: key.clone(),
                area_key,
                area_name,
                npc_friendly_name: vault.npc_friendly_name.clone(),
                num_slots: vault.num_slots,
            }
        })
        .collect();
    results.sort_by(|a, b| a.vault_key.cmp(&b.vault_key));
    Ok(results)
}

#[derive(serde::Serialize)]
pub struct StorageVaultDetail {
    pub key: String,
    pub id: u32,
    pub npc_friendly_name: Option<String>,
    pub area: Option<String>,
    pub area_name: Option<String>,
    pub grouping: Option<String>,
    pub grouping_name: Option<String>,
    pub num_slots: Option<u32>,
    pub levels: Option<std::collections::HashMap<String, u32>>,
    pub slot_attribute: Option<String>,
    pub required_item_keywords: Option<Vec<String>>,
    pub requirement_description: Option<String>,
    pub num_slots_script_atomic_max: Option<u32>,
}

/// Get full metadata for all storage vaults (for the Storage Tracker feature).
#[tauri::command]
pub async fn get_storage_vault_metadata(
    state: State<'_, GameDataState>,
) -> Result<Vec<StorageVaultDetail>, String> {
    let data = state.read().await;
    let mut results: Vec<StorageVaultDetail> = data
        .storage_vaults
        .iter()
        .map(|(key, vault)| {
            let area_name = vault
                .area
                .as_ref()
                .and_then(|ak| data.areas.get(ak))
                .and_then(|a| a.short_friendly_name.clone().or(a.friendly_name.clone()));
            let grouping_area = vault.grouping.as_ref().or(vault.area.as_ref());
            let grouping_name = grouping_area
                .and_then(|gk| data.areas.get(gk))
                .and_then(|a| a.short_friendly_name.clone().or(a.friendly_name.clone()));

            StorageVaultDetail {
                key: key.clone(),
                id: vault.id,
                npc_friendly_name: vault.npc_friendly_name.clone(),
                area: vault.area.clone(),
                area_name,
                grouping: vault.grouping.clone(),
                grouping_name,
                num_slots: vault.num_slots,
                levels: vault.levels.clone(),
                slot_attribute: vault.slot_attribute.clone(),
                required_item_keywords: vault.required_item_keywords.clone(),
                requirement_description: vault.requirement_description.clone(),
                num_slots_script_atomic_max: vault.num_slots_script_atomic_max,
            }
        })
        .collect();
    results.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(results)
}

// ── Build Planner CDN queries ───────────────────────────────────────────────

/// Get all combat skills (where combat == true in CDN data).
#[tauri::command]
pub async fn get_combat_skills(state: State<'_, GameDataState>) -> Result<Vec<SkillInfo>, String> {
    let data = state.read().await;
    let mut skills: Vec<SkillInfo> = data
        .skills
        .values()
        .filter(|s| s.combat == Some(true))
        .cloned()
        .collect();
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

/// Summary of a single tier within a TSys power.
#[derive(serde::Serialize, Clone)]
pub struct TsysTierSummary {
    pub tier_id: String,
    pub min_level: i64,
    pub max_level: i64,
    pub min_rarity: Option<String>,
    pub skill_level_prereq: Option<i64>,
    pub effects: Vec<String>,
    pub icon_id: Option<u32>,
}

/// A single TSys power eligible for a specific equipment slot, with resolved effect text.
#[derive(serde::Serialize)]
pub struct SlotTsysPower {
    /// The CDN key for this power (e.g. "power_12345")
    pub key: String,
    /// Internal name (e.g. "SwordBoost")
    pub internal_name: Option<String>,
    /// Which skill this mod belongs to, or None for generic
    pub skill: Option<String>,
    /// Display prefix (e.g. "Sword Damage")
    pub prefix: Option<String>,
    /// Display suffix
    pub suffix: Option<String>,
    /// The tier index that matched the target level
    pub tier_id: Option<String>,
    /// Resolved effect descriptions at this tier
    pub effects: Vec<String>,
    /// Raw effect descriptions (unresolved tokens)
    pub raw_effects: Vec<String>,
    /// Minimum rarity required for this tier
    pub min_rarity: Option<String>,
    /// Skill level prerequisite for this tier
    pub skill_level_prereq: Option<i64>,
    /// Icon ID from the first effect's attribute definition
    pub icon_id: Option<u32>,
    /// All available tiers for this power (for tier selection UI)
    pub available_tiers: Vec<TsysTierSummary>,
}

/// Get all eligible TSys powers for a given equipment slot, filtered by skills and target level.
/// Returns powers belonging to skill_primary, skill_secondary, or generic (no skill).
#[tauri::command]
pub async fn get_tsys_powers_for_slot(
    skill_primary: Option<String>,
    skill_secondary: Option<String>,
    equip_slot: String,
    target_level: i64,
    state: State<'_, GameDataState>,
) -> Result<Vec<SlotTsysPower>, String> {
    let data = state.read().await;
    let mut results: Vec<SlotTsysPower> = Vec::new();

    for (key, info) in &data.tsys.client_info {
        // Skip unavailable powers
        if info.is_unavailable == Some(true) {
            continue;
        }

        // Must include this equipment slot (map build planner IDs to CDN slot names)
        let slot_matches = info.slots.iter().any(|s| {
            match equip_slot.as_str() {
                "OffHand" => s == "OffHand" || s == "OffHandShield",
                "Belt" => s == "Waist",
                _ => s == &equip_slot,
            }
        });
        if !slot_matches {
            continue;
        }

        // No skill filter — return all mods for this slot.
        // The frontend filters by skill per-column.

        // Find the best tier for the target level
        let tiers = match &info.tiers {
            Some(Value::Object(map)) => map,
            _ => continue,
        };

        // Collect all available tiers for this power
        let mut available_tiers: Vec<TsysTierSummary> = Vec::new();
        for (tier_key, tier_val) in tiers {
            let min_level = tier_val.get("MinLevel").and_then(|v| v.as_i64()).unwrap_or(0);
            let max_level = tier_val.get("MaxLevel").and_then(|v| v.as_i64()).unwrap_or(999);
            let min_rarity = tier_val.get("MinRarity").and_then(|v| v.as_str()).map(String::from);
            let skill_level_prereq = tier_val.get("SkillLevelPrereq").and_then(|v| v.as_i64());

            let raw_descs: Vec<String> = tier_val
                .get("EffectDescs")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            let mut tier_icon_id: Option<u32> = None;
            let effects: Vec<String> = raw_descs
                .iter()
                .map(|desc| {
                    if let Some(resolved) = resolve_single_effect(desc, &data) {
                        if tier_icon_id.is_none() {
                            tier_icon_id = resolved.icon_id;
                        }
                        resolved.formatted
                    } else {
                        desc.clone()
                    }
                })
                .collect();

            available_tiers.push(TsysTierSummary {
                tier_id: tier_key.clone(),
                min_level,
                max_level,
                min_rarity,
                skill_level_prereq,
                effects,
                icon_id: tier_icon_id,
            });
        }
        // Sort tiers by min_level ascending
        available_tiers.sort_by_key(|t| t.min_level);

        let mut best_tier_id: Option<String> = None;
        let mut best_effects: Vec<String> = Vec::new();
        let mut best_raw_effects: Vec<String> = Vec::new();
        let mut best_min_rarity: Option<String> = None;
        let mut best_skill_prereq: Option<i64> = None;
        let mut best_icon_id: Option<u32> = None;

        for (tier_key, tier_val) in tiers {
            let min_level = tier_val
                .get("MinLevel")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let max_level = tier_val
                .get("MaxLevel")
                .and_then(|v| v.as_i64())
                .unwrap_or(999);

            if target_level >= min_level && target_level <= max_level {
                best_tier_id = Some(tier_key.clone());
                best_min_rarity = tier_val
                    .get("MinRarity")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                best_skill_prereq = tier_val.get("SkillLevelPrereq").and_then(|v| v.as_i64());

                let raw_descs: Vec<String> = tier_val
                    .get("EffectDescs")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                best_raw_effects = raw_descs.clone();
                best_icon_id = None;
                best_effects = raw_descs
                    .iter()
                    .map(|desc| {
                        if let Some(resolved) = resolve_single_effect(desc, &data) {
                            if best_icon_id.is_none() {
                                best_icon_id = resolved.icon_id;
                            }
                            resolved.formatted
                        } else {
                            desc.clone()
                        }
                    })
                    .collect();
                break; // First matching tier wins
            }
        }

        // If no tier matched, try to find the highest tier at or below target level
        if best_tier_id.is_none() {
            let mut best_min: i64 = 0;
            for (tier_key, tier_val) in tiers {
                let min_level = tier_val
                    .get("MinLevel")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                if min_level <= target_level && min_level >= best_min {
                    best_min = min_level;
                    best_tier_id = Some(tier_key.clone());
                    best_min_rarity = tier_val
                        .get("MinRarity")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    best_skill_prereq = tier_val.get("SkillLevelPrereq").and_then(|v| v.as_i64());

                    let raw_descs: Vec<String> = tier_val
                        .get("EffectDescs")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();

                    best_raw_effects = raw_descs.clone();
                    best_icon_id = None;
                    best_effects = raw_descs
                        .iter()
                        .map(|desc| {
                            if let Some(resolved) = resolve_single_effect(desc, &data) {
                                if best_icon_id.is_none() {
                                    best_icon_id = resolved.icon_id;
                                }
                                resolved.formatted
                            } else {
                                desc.clone()
                            }
                        })
                        .collect();
                }
            }
        }

        // Final fallback: if target level is below all tiers, pick the lowest tier
        // (higher-level mods can go on lower-level gear, raising the equip requirement)
        if best_tier_id.is_none() {
            let mut lowest_min: i64 = i64::MAX;
            for (tier_key, tier_val) in tiers {
                let min_level = tier_val
                    .get("MinLevel")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                if min_level < lowest_min {
                    lowest_min = min_level;
                    best_tier_id = Some(tier_key.clone());
                    best_min_rarity = tier_val
                        .get("MinRarity")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    best_skill_prereq = tier_val.get("SkillLevelPrereq").and_then(|v| v.as_i64());

                    let raw_descs: Vec<String> = tier_val
                        .get("EffectDescs")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();

                    best_raw_effects = raw_descs.clone();
                    best_icon_id = None;
                    best_effects = raw_descs
                        .iter()
                        .map(|desc| {
                            if let Some(resolved) = resolve_single_effect(desc, &data) {
                                if best_icon_id.is_none() {
                                    best_icon_id = resolved.icon_id;
                                }
                                resolved.formatted
                            } else {
                                desc.clone()
                            }
                        })
                        .collect();
                }
            }
        }

        if best_tier_id.is_some() {
            // Resolve skill internal name to display name (e.g. "FireMagic" -> "Fire Magic")
            let resolved_skill = info.skill.as_deref().and_then(|s| {
                data.resolve_skill(s).map(|si| si.name.clone())
            }).or_else(|| info.skill.clone());

            results.push(SlotTsysPower {
                key: key.clone(),
                internal_name: info.internal_name.clone(),
                skill: resolved_skill,
                prefix: info.prefix.clone(),
                suffix: info.suffix.clone(),
                tier_id: best_tier_id,
                effects: best_effects,
                raw_effects: best_raw_effects,
                min_rarity: best_min_rarity,
                skill_level_prereq: best_skill_prereq,
                icon_id: best_icon_id,
                available_tiers: if available_tiers.len() > 1 { available_tiers } else { vec![] },
            });
        }
    }

    // Sort: skill-specific first (grouped by skill), then generic, then by name
    results.sort_by(|a, b| {
        let a_is_generic = a.skill.is_none();
        let b_is_generic = b.skill.is_none();
        if a_is_generic != b_is_generic {
            return a_is_generic.cmp(&b_is_generic);
        }
        let skill_cmp = a.skill.cmp(&b.skill);
        if skill_cmp != std::cmp::Ordering::Equal {
            return skill_cmp;
        }
        let a_name = a
            .prefix
            .as_deref()
            .or(a.suffix.as_deref())
            .or(a.internal_name.as_deref())
            .unwrap_or("");
        let b_name = b
            .prefix
            .as_deref()
            .or(b.suffix.as_deref())
            .or(b.internal_name.as_deref())
            .unwrap_or("");
        a_name.cmp(b_name)
    });

    Ok(results)
}

// ── TSys browser commands ────────────────────────────────────────────────────

/// Flattened TSys entry for the frontend browser (includes the CDN key).
#[derive(Debug, serde::Serialize, Clone)]
pub struct TsysBrowserEntry {
    pub key: String,
    pub internal_name: Option<String>,
    pub skill: Option<String>,
    pub slots: Vec<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub tiers: Option<Value>,
    pub is_unavailable: Option<bool>,
    pub is_hidden_from_transmutation: Option<bool>,
    pub tier_count: usize,
    pub raw_json: Value,
}

impl TsysBrowserEntry {
    fn from_entry(key: &str, info: &TsysClientInfo) -> Self {
        let tier_count = info
            .tiers
            .as_ref()
            .and_then(|t| t.as_object())
            .map(|m| m.len())
            .unwrap_or(0);
        Self {
            key: key.to_string(),
            internal_name: info.internal_name.clone(),
            skill: info.skill.clone(),
            slots: info.slots.clone(),
            prefix: info.prefix.clone(),
            suffix: info.suffix.clone(),
            tiers: info.tiers.clone(),
            is_unavailable: info.is_unavailable,
            is_hidden_from_transmutation: info.is_hidden_from_transmutation,
            tier_count,
            raw_json: info.raw_json.clone(),
        }
    }
}

/// Get all TSys client info entries for the browser.
#[tauri::command]
pub async fn get_all_tsys(
    state: State<'_, GameDataState>,
) -> Result<Vec<TsysBrowserEntry>, String> {
    let data = state.read().await;
    let mut results: Vec<TsysBrowserEntry> = data
        .tsys
        .client_info
        .iter()
        .map(|(key, info)| TsysBrowserEntry::from_entry(key, info))
        .collect();
    results.sort_by(|a, b| {
        let a_name = a.internal_name.as_deref().unwrap_or(&a.key);
        let b_name = b.internal_name.as_deref().unwrap_or(&b.key);
        a_name.cmp(b_name)
    });
    Ok(results)
}

/// Search TSys entries by name, skill, prefix, suffix, or key.
#[tauri::command]
pub async fn search_tsys(
    query: String,
    limit: Option<usize>,
    state: State<'_, GameDataState>,
) -> Result<Vec<TsysBrowserEntry>, String> {
    let data = state.read().await;
    let q = query.to_lowercase();
    let limit = limit.unwrap_or(100);

    let mut results: Vec<TsysBrowserEntry> = data
        .tsys
        .client_info
        .iter()
        .filter(|(key, info)| {
            let key_match = key.to_lowercase().contains(&q);
            let name_match = info
                .internal_name
                .as_ref()
                .map(|n| n.to_lowercase().contains(&q))
                .unwrap_or(false);
            let skill_match = info
                .skill
                .as_ref()
                .map(|s| s.to_lowercase().contains(&q))
                .unwrap_or(false);
            let prefix_match = info
                .prefix
                .as_ref()
                .map(|p| p.to_lowercase().contains(&q))
                .unwrap_or(false);
            let suffix_match = info
                .suffix
                .as_ref()
                .map(|s| s.to_lowercase().contains(&q))
                .unwrap_or(false);
            let slot_match = info
                .slots
                .iter()
                .any(|s| s.to_lowercase().contains(&q));
            key_match || name_match || skill_match || prefix_match || suffix_match || slot_match
        })
        .map(|(key, info)| TsysBrowserEntry::from_entry(key, info))
        .collect();

    results.sort_by(|a, b| {
        let a_name = a.internal_name.as_deref().unwrap_or(&a.key);
        let b_name = b.internal_name.as_deref().unwrap_or(&b.key);
        a_name.cmp(b_name)
    });
    results.truncate(limit);
    Ok(results)
}

/// Get TSys profiles (raw JSON).
#[tauri::command]
pub async fn get_tsys_profiles(
    state: State<'_, GameDataState>,
) -> Result<Value, String> {
    let data = state.read().await;
    Ok(data.tsys.profiles.clone())
}

// ── Cross-reference commands for data browser linking ──────────────────────

/// NPC favor entry returned to the frontend.
#[derive(serde::Serialize, Clone)]
pub struct NpcFavorEntry {
    pub npc_key: String,
    pub npc_name: String,
    pub desire: String,
    pub pref: f32,
    pub match_type: String,
}

/// Get all NPCs that want a given item (by name match or keyword match).
#[tauri::command]
pub async fn get_npcs_wanting_item(
    item_id: u32,
    state: State<'_, GameDataState>,
) -> Result<Vec<NpcFavorEntry>, String> {
    let data = state.read().await;
    let item = data.items.get(&item_id).ok_or("Item not found")?;

    let mut results: Vec<NpcFavorEntry> = Vec::new();
    let mut seen_npc_keys = std::collections::HashSet::new();

    // Check name-based matches
    let name_lower = item.name.to_lowercase();
    if let Some(entries) = data.npc_favor_by_item_name.get(&name_lower) {
        for (npc_key, desire, pref) in entries {
            if seen_npc_keys.insert(npc_key.clone()) {
                let npc_name = data.npcs.get(npc_key).map(|n| n.name.clone()).unwrap_or_default();
                results.push(NpcFavorEntry {
                    npc_key: npc_key.clone(),
                    npc_name,
                    desire: desire.clone(),
                    pref: *pref,
                    match_type: "name".to_string(),
                });
            }
        }
    }

    // Check keyword-based matches
    for keyword in &item.keywords {
        if let Some(entries) = data.npc_favor_by_keyword.get(keyword) {
            for (npc_key, desire, pref) in entries {
                if seen_npc_keys.insert(npc_key.clone()) {
                    let npc_name = data.npcs.get(npc_key).map(|n| n.name.clone()).unwrap_or_default();
                    results.push(NpcFavorEntry {
                        npc_key: npc_key.clone(),
                        npc_name,
                        desire: desire.clone(),
                        pref: *pref,
                        match_type: format!("keyword:{}", keyword),
                    });
                }
            }
        }
    }

    // Sort by pref value descending
    results.sort_by(|a, b| b.pref.partial_cmp(&a.pref).unwrap_or(std::cmp::Ordering::Equal));
    Ok(results)
}

/// Get all NPCs that train a given skill.
#[tauri::command]
pub async fn get_npcs_training_skill(
    skill: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<NpcInfo>, String> {
    let data = state.read().await;

    // npcs_by_skill is keyed by skill internal name from training data
    let npc_keys = data.npcs_by_skill.get(&skill).cloned().unwrap_or_default();
    let results: Vec<NpcInfo> = npc_keys
        .iter()
        .filter_map(|key| data.npcs.get(key).cloned())
        .collect();
    Ok(results)
}

/// Get all quests associated with a given NPC (via FavorNpc field).
#[tauri::command]
pub async fn get_quests_for_npc(
    npc_key: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<QuestInfo>, String> {
    let data = state.read().await;
    let quest_keys = data.quests_by_npc.get(&npc_key).cloned().unwrap_or_default();
    let results: Vec<QuestInfo> = quest_keys
        .iter()
        .filter_map(|key| data.quests.get(key).cloned())
        .collect();
    Ok(results)
}

/// Get all work order quests for a given skill.
#[tauri::command]
pub async fn get_quests_for_skill(
    skill: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<QuestInfo>, String> {
    let data = state.read().await;
    let quest_keys = data.quests_by_work_order_skill.get(&skill).cloned().unwrap_or_default();
    let results: Vec<QuestInfo> = quest_keys
        .iter()
        .filter_map(|key| data.quests.get(key).cloned())
        .collect();
    Ok(results)
}

/// Get all recipes that accept a given keyword as an ingredient.
#[tauri::command]
pub async fn get_recipes_for_keyword(
    keyword: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<RecipeInfo>, String> {
    let data = state.read().await;
    let recipe_ids = data.recipes_by_ingredient_keyword.get(&keyword).cloned().unwrap_or_default();
    let results: Vec<RecipeInfo> = recipe_ids
        .iter()
        .filter_map(|id| data.recipes.get(id).cloned())
        .collect();
    Ok(results)
}
