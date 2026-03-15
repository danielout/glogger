/// Tauri commands for CDN data management and game data queries.
/// These are the invoke() endpoints the Vue frontend calls.

use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;

use crate::cdn;
use crate::game_data::{self, GameData, ItemInfo, SkillInfo, AbilityInfo, RecipeInfo, QuestInfo, NpcInfo, EffectInfo, PlayerTitleInfo, SourceEntry};

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
    let remote_version = match cdn::fetch_remote_version().await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("CDN version check failed (offline?): {e}");
            if let Some(cached) = cached_version {
                eprintln!("Loading from cache (v{cached})");
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
        eprintln!("Downloading CDN data v{remote_version}...");
        cdn::download_all_data_files(remote_version, &data_dir).await?;
        cdn::write_cached_version(&data_dir, remote_version).await?;
        eprintln!("CDN data downloaded.");
    } else {
        eprintln!("CDN data up to date (v{remote_version}).");
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

// ── Item query commands ───────────────────────────────────────────────────────

/// Look up a single item by its integer ID.
#[tauri::command]
pub async fn get_item(
    id: u32,
    state: State<'_, GameDataState>,
) -> Result<Option<ItemInfo>, String> {
    Ok(state.read().await.items.get(&id).cloned())
}

/// Look up a single item by its display name (exact match).
#[tauri::command]
pub async fn get_item_by_name(
    name: String,
    state: State<'_, GameDataState>,
) -> Result<Option<ItemInfo>, String> {
    Ok(state.read().await.item_by_name(&name).cloned())
}

/// Search items whose name contains the query string (case-insensitive).
/// Returns up to `limit` results (default 20).
/// Optional filters: equip_slot (exact match), min/max crafting_target_level.
#[tauri::command]
pub async fn search_items(
    query: String,
    limit: Option<usize>,
    equip_slot: Option<String>,
    level_min: Option<u32>,
    level_max: Option<u32>,
    state: State<'_, GameDataState>,
) -> Result<Vec<ItemInfo>, String> {
    let limit = limit.unwrap_or(20);
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
            // Equip slot filter
            if let Some(ref slot) = equip_slot {
                match &item.equip_slot {
                    Some(s) if s == slot => {}
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
        .take(limit)
        .cloned()
        .collect();
    results.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(results)
}

/// Return a sorted list of all distinct equip_slot values across all items.
#[tauri::command]
pub async fn get_equip_slots(
    state: State<'_, GameDataState>,
) -> Result<Vec<String>, String> {
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
pub async fn get_all_skills(
    state: State<'_, GameDataState>,
) -> Result<Vec<SkillInfo>, String> {
    let data = state.read().await;
    let mut skills: Vec<SkillInfo> = data.skills.values().cloned().collect();
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

/// Look up a single skill by name (exact match, case-sensitive as per CDN).
#[tauri::command]
pub async fn get_skill_by_name(
    name: String,
    state: State<'_, GameDataState>,
) -> Result<Option<SkillInfo>, String> {
    Ok(state.read().await.skill_by_name(&name).cloned())
}

// ── Ability query commands ────────────────────────────────────────────────────

/// Get all abilities for a given skill name.
#[tauri::command]
pub async fn get_abilities_for_skill(
    skill: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<AbilityInfo>, String> {
    let data = state.read().await;
    let mut abilities: Vec<AbilityInfo> = data
        .abilities
        .values()
        .filter(|a| a.skill.as_deref() == Some(&skill))
        .cloned()
        .collect();
    abilities.sort_by(|a, b| a.level.unwrap_or(0.0).partial_cmp(&b.level.unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal));
    Ok(abilities)
}

// ── Recipe query commands ─────────────────────────────────────────────────────

/// Look up a single recipe by name (exact match).
#[tauri::command]
pub async fn get_recipe_by_name(
    name: String,
    state: State<'_, GameDataState>,
) -> Result<Option<RecipeInfo>, String> {
    Ok(state.read().await.recipe_by_name(&name).cloned())
}

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

/// Get multiple items by their IDs (for efficient batch lookup).
#[tauri::command]
pub async fn get_items_batch(
    ids: Vec<u32>,
    state: State<'_, GameDataState>,
) -> Result<std::collections::HashMap<u32, ItemInfo>, String> {
    let data = state.read().await;
    let mut result = std::collections::HashMap::new();

    for id in ids {
        if let Some(item) = data.items.get(&id) {
            result.insert(id, item.clone());
        }
    }

    Ok(result)
}

// ── Quest query commands ──────────────────────────────────────────────────────

/// Get all quests.
#[tauri::command]
pub async fn get_all_quests(
    state: State<'_, GameDataState>,
) -> Result<Vec<QuestInfo>, String> {
    let data = state.read().await;
    let mut results: Vec<QuestInfo> = data.quests.values().cloned().collect();
    results.sort_by(|a, b| {
        // Sort by internal_name for now since we don't have parsed display names yet
        let a_name = a.raw.get("DisplayName")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let b_name = b.raw.get("DisplayName")
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
            let display_name = quest.raw.get("DisplayName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();
            let description = quest.raw.get("Description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();

            display_name.contains(&q) || description.contains(&q)
        })
        .cloned()
        .collect();

    results.sort_by(|a, b| {
        let a_name = a.raw.get("DisplayName")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let b_name = b.raw.get("DisplayName")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        a_name.cmp(b_name)
    });

    Ok(results)
}

/// Get a quest by its internal key.
#[tauri::command]
pub async fn get_quest_by_key(
    key: String,
    state: State<'_, GameDataState>,
) -> Result<Option<QuestInfo>, String> {
    let data = state.read().await;
    Ok(data.quests.get(&key).cloned())
}

// ── NPC query commands ────────────────────────────────────────────────────────

/// Get all NPCs.
#[tauri::command]
pub async fn get_all_npcs(
    state: State<'_, GameDataState>,
) -> Result<Vec<NpcInfo>, String> {
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
            let desc_match = npc.desc
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
        .filter(|npc| {
            npc.area_name.as_ref().map(|a| a == &area).unwrap_or(false)
        })
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
            let name_match = e.name.as_ref()
                .map(|n| n.to_lowercase().contains(&q))
                .unwrap_or(false);
            let desc_match = e.desc.as_ref()
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
            let title_match = t.title.as_ref()
                .map(|n| n.to_lowercase().contains(&q))
                .unwrap_or(false);
            let tooltip_match = t.tooltip.as_ref()
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

    let cdn_sources = data.sources.abilities
        .get(&id)
        .map(|s| s.entries.clone())
        .unwrap_or_default();

    // Find items that bestow this ability via bestow_ability field.
    // bestow_ability stores the internal name like "ability_1002"
    let ability_key = format!("ability_{id}");
    let bestowed_by_items = data.items_bestowing_ability
        .get(&ability_key)
        .map(|item_ids| {
            item_ids.iter()
                .filter_map(|iid| data.items.get(iid))
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    Ok(EntitySources { cdn_sources, bestowed_by_items, rewarded_by_quests: vec![] })
}

/// Get all known sources for an item.
#[tauri::command]
pub async fn get_item_sources(
    id: u32,
    state: State<'_, GameDataState>,
) -> Result<EntitySources, String> {
    let data = state.read().await;

    let cdn_sources = data.sources.items
        .get(&id)
        .map(|s| s.entries.clone())
        .unwrap_or_default();

    // Find quests that reward this item
    let item_key = format!("item_{id}");
    let rewarded_by_quests = data.quests_rewarding_item
        .get(&item_key)
        .map(|quest_keys| {
            quest_keys.iter()
                .filter_map(|qk| {
                    let quest = data.quests.get(qk)?;
                    let name = quest.raw.get("Name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(qk.as_str())
                        .to_string();
                    Some(QuestSummary { key: qk.clone(), name })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(EntitySources { cdn_sources, bestowed_by_items: vec![], rewarded_by_quests })
}

/// Get all known sources for a recipe.
#[tauri::command]
pub async fn get_recipe_sources(
    id: u32,
    state: State<'_, GameDataState>,
) -> Result<EntitySources, String> {
    let data = state.read().await;

    let cdn_sources = data.sources.recipes
        .get(&id)
        .map(|s| s.entries.clone())
        .unwrap_or_default();

    // Find items that bestow this recipe
    let recipe_key = format!("recipe_{id}");
    let bestowed_by_items = data.items_bestowing_recipe
        .get(&recipe_key)
        .map(|item_ids| {
            item_ids.iter()
                .filter_map(|iid| data.items.get(iid))
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    Ok(EntitySources { cdn_sources, bestowed_by_items, rewarded_by_quests: vec![] })
}

/// Get all known sources for a quest (items that bestow it).
#[tauri::command]
pub async fn get_quest_sources(
    key: String,
    state: State<'_, GameDataState>,
) -> Result<EntitySources, String> {
    let data = state.read().await;

    let bestowed_by_items = data.items_bestowing_quest
        .get(&key)
        .map(|item_ids| {
            item_ids.iter()
                .filter_map(|iid| data.items.get(iid))
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    Ok(EntitySources { cdn_sources: vec![], bestowed_by_items, rewarded_by_quests: vec![] })
}

// ── Effect description resolution ───────────────────────────────────────────

#[derive(serde::Serialize)]
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

fn resolve_single_effect(desc: &str, data: &GameData) -> Option<ResolvedEffect> {
    // Parse "{TOKEN}{VALUE}" format
    let parts: Vec<&str> = desc.split('{').filter(|s| !s.is_empty()).collect();
    if parts.len() < 2 {
        return None;
    }

    let token = parts[0].trim_end_matches('}');
    let value_str = parts[1].trim_end_matches('}');

    let attr = data.attributes.get(token)?;
    let label = attr.raw.get("Label")?.as_str()?.to_string();
    let display_type = attr.raw.get("DisplayType")
        .and_then(|v| v.as_str())
        .unwrap_or("AsInt")
        .to_string();
    let icon_id = attr.raw.get("IconIds")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let value: f64 = value_str.parse().unwrap_or(0.0);
    let formatted = format_effect_value(&label, value, &display_type);

    Some(ResolvedEffect {
        label,
        value: value_str.to_string(),
        display_type,
        formatted,
        icon_id,
    })
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
    let entry = data.tsys.client_info.values()
        .find(|info| info.internal_name.as_deref() == Some(&power_name));

    let Some(info) = entry else {
        return Ok(None);
    };

    // Get the effect descriptions for the requested tier
    let tier_key = format!("id_{}", tier);
    let tier_effects: Vec<String> = info.tiers.as_ref()
        .and_then(|tiers| tiers.get(&tier_key))
        .and_then(|t| t.get("EffectDescs"))
        .and_then(|descs| descs.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|desc| {
                    // Try to resolve the effect desc to human-readable
                    resolve_single_effect(desc, &data)
                        .map(|r| r.formatted)
                        .unwrap_or_else(|| desc.to_string())
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(Some(TsysPowerInfo {
        internal_name: power_name,
        skill: info.skill.clone(),
        prefix: info.prefix.clone(),
        suffix: info.suffix.clone(),
        slots: info.slots.clone(),
        tier_effects,
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
            let area_key = vault.raw.get("Area")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let area_name = area_key.as_ref()
                .and_then(|ak| data.areas.get(ak))
                .and_then(|a| a.short_friendly_name.clone().or(a.friendly_name.clone()));
            let npc_friendly_name = vault.raw.get("NpcFriendlyName")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let num_slots = vault.raw.get("NumSlots")
                .and_then(|v| v.as_u64())
                .map(|n| n as u32);

            StorageVaultZoneInfo {
                vault_key: key.clone(),
                area_key,
                area_name,
                npc_friendly_name,
                num_slots,
            }
        })
        .collect();
    results.sort_by(|a, b| a.vault_key.cmp(&b.vault_key));
    Ok(results)
}