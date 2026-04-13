use chrono::{Datelike, Local};
/// Tauri commands for CDN data management and game data queries.
/// These are the invoke() endpoints the Vue frontend calls.
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;

use serde_json::Value;

use crate::cdn;
use crate::db::DbPool;
use crate::game_data::{
    self, AbilityFamily, AbilityInfo, AreaInfo, EffectInfo, GameData, ItemInfo, NpcInfo, PlayerTitleInfo,
    QuestInfo, RecipeInfo, SkillInfo, SourceEntry, TsysClientInfo, TsysTierInfo,
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
    armor_type: Option<String>,
    effect_text: Option<String>,
    state: State<'_, GameDataState>,
) -> Result<Vec<ItemInfo>, String> {
    let q = query.to_lowercase();
    let effect_q = effect_text.as_ref().map(|s| s.to_lowercase());
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
            // Armor type filter (keyword-based: ClothArmor, LeatherArmor, MetalArmor, OrganicArmor)
            if let Some(ref at) = armor_type {
                let keyword = match at.as_str() {
                    "Cloth" => "ClothArmor",
                    "Leather" => "LeatherArmor",
                    "Metal" => "MetalArmor",
                    "Organic" => "OrganicArmor",
                    _ => return false,
                };
                if !item.keywords.iter().any(|k| k == keyword) {
                    return false;
                }
            }
            // Effect text filter (search within effect descriptions)
            if let Some(ref eq) = effect_q {
                if !eq.is_empty() {
                    let has_match = item.effect_descs.iter().any(|desc| {
                        // Check raw effect desc string
                        if desc.to_lowercase().contains(eq) {
                            return true;
                        }
                        // Also try resolving the effect for a human-readable match
                        if let Some(resolved) = resolve_single_effect(desc, &data) {
                            if resolved.formatted.to_lowercase().contains(eq)
                                || resolved.label.to_lowercase().contains(eq)
                            {
                                return true;
                            }
                        }
                        false
                    });
                    if !has_match {
                        // Also check the item description as fallback
                        let desc_match = item.description.as_ref()
                            .map(|d| d.to_lowercase().contains(eq))
                            .unwrap_or(false);
                        if !desc_match {
                            return false;
                        }
                    }
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

/// Info about a keyword used as a recipe ingredient slot.
#[derive(Debug, serde::Serialize, Clone)]
pub struct IngredientKeywordInfo {
    pub keyword: String,
    pub description: String,
}

/// Return all distinct keywords used as wildcard ingredient slots in recipes,
/// along with their player-facing descriptions (from the CDN `Desc` field).
#[tauri::command]
pub async fn get_recipe_ingredient_keywords(
    state: State<'_, GameDataState>,
) -> Result<Vec<IngredientKeywordInfo>, String> {
    let data = state.read().await;
    let mut keyword_descs: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    // Iterate recipes to collect keyword → first description mapping
    for recipe in data.recipes.values() {
        for ingredient in &recipe.ingredients {
            if ingredient.item_id.is_none() && !ingredient.item_keys.is_empty() {
                let desc = ingredient
                    .description
                    .clone()
                    .unwrap_or_else(|| ingredient.item_keys.join(", "));
                for keyword in &ingredient.item_keys {
                    keyword_descs
                        .entry(keyword.clone())
                        .or_insert_with(|| desc.clone());
                }
            }
        }
    }

    let mut results: Vec<IngredientKeywordInfo> = keyword_descs
        .into_iter()
        .map(|(keyword, description)| IngredientKeywordInfo { keyword, description })
        .collect();
    results.sort_by(|a, b| a.description.cmp(&b.description));
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

/// Get all ability families for a given skill, sorted by base ability level.
#[tauri::command]
pub async fn get_ability_families_for_skill(
    skill: String,
    include_monster: Option<bool>,
    state: State<'_, GameDataState>,
) -> Result<Vec<AbilityFamily>, String> {
    let data = state.read().await;
    let show_monster = include_monster.unwrap_or(false);

    // Resolve display name → internal name
    let internal_name = data
        .skills
        .values()
        .find(|s| s.name == skill)
        .map(|s| s.internal_name.clone());
    let match_name = internal_name.as_deref().unwrap_or(&skill);

    let mut families: Vec<AbilityFamily> = data
        .ability_families
        .values()
        .filter(|f| {
            f.skill.as_deref() == Some(match_name)
                && (show_monster || !f.is_monster_ability)
        })
        .cloned()
        .collect();

    // Sort by the level of the first (lowest) tier
    families.sort_by(|a, b| {
        let level_a = a
            .tier_ids
            .first()
            .and_then(|id| data.abilities.get(id))
            .and_then(|ab| ab.level)
            .unwrap_or(0.0);
        let level_b = b
            .tier_ids
            .first()
            .and_then(|id| data.abilities.get(id))
            .and_then(|ab| ab.level)
            .unwrap_or(0.0);
        level_a
            .partial_cmp(&level_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(families)
}

/// Given any tier's ability ID, return the family it belongs to.
#[tauri::command]
pub async fn get_ability_family(
    ability_id: u32,
    state: State<'_, GameDataState>,
) -> Result<Option<AbilityFamily>, String> {
    let data = state.read().await;
    let family = data
        .ability_to_family
        .get(&ability_id)
        .and_then(|key| data.ability_families.get(key))
        .cloned();
    Ok(family)
}

/// Search ability families by name/description across all skills (or within one skill).
/// Returns up to `limit` matching families, sorted by base name.
#[tauri::command]
pub async fn search_ability_families(
    query: String,
    skill: Option<String>,
    limit: Option<usize>,
    include_monster: Option<bool>,
    state: State<'_, GameDataState>,
) -> Result<Vec<AbilityFamily>, String> {
    let data = state.read().await;
    let lower = query.to_lowercase();
    let max = limit.unwrap_or(50);
    let show_monster = include_monster.unwrap_or(false);

    // Resolve skill display name → internal name if provided
    let skill_internal = skill.as_ref().and_then(|s| {
        data.skills
            .values()
            .find(|sk| sk.name == *s)
            .map(|sk| sk.internal_name.clone())
    });
    let skill_match = skill_internal.as_deref().or(skill.as_deref());

    let mut results: Vec<AbilityFamily> = data
        .ability_families
        .values()
        .filter(|f| {
            // Filter out monster abilities unless requested
            if !show_monster && f.is_monster_ability {
                return false;
            }
            // Filter by skill if specified
            if let Some(sm) = skill_match {
                if f.skill.as_deref() != Some(sm) {
                    return false;
                }
            }
            // Match on family base_name first (fast path)
            if f.base_name.to_lowercase().contains(&lower) {
                return true;
            }
            // Fall back to checking individual tier names and descriptions
            for tier_id in &f.tier_ids {
                if let Some(ability) = data.abilities.get(tier_id) {
                    if ability.name.to_lowercase().contains(&lower) {
                        return true;
                    }
                    if let Some(desc) = &ability.description {
                        if desc.to_lowercase().contains(&lower) {
                            return true;
                        }
                    }
                }
            }
            false
        })
        .take(max)
        .cloned()
        .collect();

    results.sort_by(|a, b| a.base_name.to_lowercase().cmp(&b.base_name.to_lowercase()));
    if results.len() > max {
        results.truncate(max);
    }
    Ok(results)
}

/// Return a map of skill display name → number of ability families for that skill.
/// Used to populate the skill filter dropdown without N+1 IPC calls.
#[tauri::command]
pub async fn get_skills_with_ability_counts(
    include_monster: Option<bool>,
    state: State<'_, GameDataState>,
) -> Result<Vec<(String, usize)>, String> {
    let data = state.read().await;
    let show_monster = include_monster.unwrap_or(false);

    // Build internal_name → display_name map
    let internal_to_display: HashMap<&str, &str> = data
        .skills
        .values()
        .map(|s| (s.internal_name.as_str(), s.name.as_str()))
        .collect();

    // Count families per skill internal name
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for family in data.ability_families.values() {
        if !show_monster && family.is_monster_ability {
            continue;
        }
        if let Some(skill) = family.skill.as_deref() {
            *counts.entry(skill).or_insert(0) += 1;
        }
    }

    // Convert to display names and sort
    let mut result: Vec<(String, usize)> = counts
        .into_iter()
        .filter_map(|(internal, count)| {
            let display = internal_to_display.get(internal).unwrap_or(&internal);
            Some((display.to_string(), count))
        })
        .filter(|(_, count)| *count > 0)
        .collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
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

/// Get all recipes that produce any of the given item IDs.
/// Used by Cook's Helper to find all food-producing recipes without hardcoding skill names.
#[tauri::command]
pub async fn get_recipes_producing_items(
    item_ids: Vec<u32>,
    state: State<'_, GameDataState>,
) -> Result<Vec<RecipeInfo>, String> {
    let data = state.read().await;
    let mut seen = std::collections::HashSet::new();
    let mut results = Vec::new();

    for item_id in &item_ids {
        if let Some(recipe_ids) = data.recipes_producing_item.get(item_id) {
            for recipe_id in recipe_ids {
                if seen.insert(*recipe_id) {
                    if let Some(recipe) = data.recipes.get(recipe_id) {
                        results.push(recipe.clone());
                    }
                }
            }
        }
    }

    Ok(results)
}

// ── Moon phase commands ──────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct MoonPhaseResult {
    /// Game phase name (e.g. "FullMoon", "WaningGibbousMoon")
    pub game_phase: String,
    /// Human-readable label (e.g. "Full Moon", "Waning Gibbous")
    pub label: String,
    /// 0-7 index into the phase cycle (0 = New Moon)
    pub phase_index: u8,
    /// Days until each future phase, in cycle order starting from the next phase
    pub days_until: Vec<DaysUntilPhase>,
}

#[derive(serde::Serialize)]
pub struct DaysUntilPhase {
    pub game_phase: String,
    pub label: String,
    pub days: u32,
}

const GAME_PHASE_NAMES: &[(&str, &str)] = &[
    ("NewMoon", "New Moon"),
    ("WaxingCrescentMoon", "Waxing Crescent"),
    ("QuarterMoon", "First Quarter"),
    ("WaxingGibbousMoon", "Waxing Gibbous"),
    ("FullMoon", "Full Moon"),
    ("WaningGibbousMoon", "Waning Gibbous"),
    ("LastQuarterMoon", "Last Quarter"),
    ("WaningCrescentMoon", "Waning Crescent"),
];

// ── Meeus lunar algorithms (ported from the game server's Moon.java) ─────────

/// Mean elongation of the moon (Meeus 47.2).
fn meeus_d(t: f64) -> f64 {
    297.8501921
        + t * (445267.1114034 + t * (-0.0018819 + t * (1.0 / 545868.0 - t / 113065000.0)))
}

/// Sun's mean anomaly (Meeus 47.3).
fn meeus_m(t: f64) -> f64 {
    357.5291092 + t * (35999.0502909 + t * (-0.0001536 + t / 24490000.0))
}

/// Moon's mean anomaly (Meeus 47.4).
fn meeus_m_prime(t: f64) -> f64 {
    134.9633964
        + t * (477198.8675055 + t * (0.0087414 + t * (1.0 / 69699.0 - t / 14712000.0)))
}

/// Phase angle between sun, moon, and earth (Meeus 48.4).
fn meeus_phase_angle(t: f64) -> f64 {
    let d = meeus_d(t);
    let m = meeus_m(t);
    let mp = meeus_m_prime(t);
    180.0 - d
        - 6.289 * mp.to_radians().sin()
        + 2.100 * m.to_radians().sin()
        - 1.274 * (2.0 * d - mp).to_radians().sin()
        - 0.658 * (2.0 * d).to_radians().sin()
        - 0.214 * (2.0 * mp).to_radians().sin()
        - 0.110 * d.to_radians().sin()
}

fn moon_is_waning(phase_angle: f64) -> bool {
    let s = phase_angle.to_radians().sin();
    if s < 0.0 {
        return true;
    }
    if s > 0.0 {
        return false;
    }
    phase_angle.to_radians().cos() > 0.0
}

fn moon_illuminated_fraction(phase_angle: f64) -> f64 {
    (1.0 + phase_angle.to_radians().cos()) * 0.5
}

/// Compute Julian Day number for a calendar date at midnight UTC (Meeus ch. 7).
fn calendar_to_jd(year: i32, month: u32, day: u32) -> f64 {
    let (y, m) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };
    let a = (y as f64 / 100.0).floor() as i32;
    let b = 2 - a + (a as f64 / 4.0).floor() as i32;
    (365.25 * (y + 4716) as f64).floor()
        + (30.6001 * (m + 1) as f64).floor()
        + day as f64
        + b as f64
        - 1524.5
}

struct MoonDay {
    illuminated: f64,
    waning: bool,
}

/// Get the moon's illumination and waning state for a calendar date at midnight UTC.
fn moon_for_date(year: i32, month: u32, day: u32) -> MoonDay {
    let jd = calendar_to_jd(year, month, day);
    let t = (jd - 2451545.0) / 36525.0; // Meeus 22.1 — centuries since J2000.0
    let i = meeus_phase_angle(t);
    MoonDay {
        illuminated: moon_illuminated_fraction(i),
        waning: moon_is_waning(i),
    }
}

/// Get the current moon phase using the game server's exact algorithm.
///
/// The server (MoonPhaseCalculator.java) builds a 31-day window of daily
/// illumination data, then assigns the four major phases (Full, New, First
/// Quarter, Last Quarter) as exactly 3-day spans centered on the transition
/// day. Remaining days are filled with crescent/gibbous based on illumination
/// and waning state.
#[tauri::command]
pub async fn get_current_moon_phase() -> Result<MoonPhaseResult, String> {
    // Get today's date in Eastern timezone (game server time)
    let now = chrono::Utc::now();
    let eastern = chrono_tz::US::Eastern;
    let eastern_now = now.with_timezone(&eastern);
    let today = eastern_now.date_naive();

    // Build 31-day window starting from yesterday, matching the server's
    // calcPhasesForDate which uses `date.plusDays(loop - 1)` for loop 0..31.
    // phases[1] = today's phase.
    let window_start = today - chrono::Duration::days(1);
    let mut days: Vec<(chrono::NaiveDate, MoonDay)> = Vec::with_capacity(31);
    for i in 0..31 {
        let date = window_start + chrono::Duration::days(i);
        let md = moon_for_date(date.year(), date.month(), date.day());
        days.push((date, md));
    }

    // Assign major phases (exactly 3 days each), matching the server's algorithm.
    let mut assigned: Vec<Option<u8>> = vec![None; 31];

    // Full Moon: transition from non-waning to waning
    for i in 0..30 {
        if !days[i].1.waning && days[i + 1].1.waning {
            if i > 0 {
                assigned[i - 1] = Some(4);
            }
            assigned[i] = Some(4);
            if i + 1 < 31 {
                assigned[i + 1] = Some(4);
            }
            break;
        }
    }

    // New Moon: transition from waning to non-waning
    for i in 0..30 {
        if days[i].1.waning && !days[i + 1].1.waning {
            if i > 0 {
                assigned[i - 1] = Some(0);
            }
            assigned[i] = Some(0);
            if i + 1 < 31 {
                assigned[i + 1] = Some(0);
            }
            break;
        }
    }

    // First Quarter: illumination crosses 0.5 upward while waxing
    for i in 0..30 {
        if !days[i].1.waning
            && !days[i + 1].1.waning
            && days[i].1.illuminated <= 0.5
            && days[i + 1].1.illuminated > 0.5
        {
            if i > 0 {
                assigned[i - 1] = Some(2);
            }
            assigned[i] = Some(2);
            if i + 1 < 31 {
                assigned[i + 1] = Some(2);
            }
            break;
        }
    }

    // Last Quarter: illumination crosses 0.5 downward while waning
    for i in 0..30 {
        if days[i].1.waning
            && days[i + 1].1.waning
            && days[i].1.illuminated >= 0.5
            && days[i + 1].1.illuminated < 0.5
        {
            if i > 0 {
                assigned[i - 1] = Some(6);
            }
            assigned[i] = Some(6);
            if i + 1 < 31 {
                assigned[i + 1] = Some(6);
            }
            break;
        }
    }

    // Fill remaining days with crescent/gibbous based on illumination
    for i in 0..31 {
        if assigned[i].is_none() {
            assigned[i] = Some(if days[i].1.waning {
                if days[i].1.illuminated >= 0.5 {
                    5
                } else {
                    7
                }
            } else if days[i].1.illuminated <= 0.5 {
                1
            } else {
                3
            });
        }
    }

    // phases[1] = today's phase
    let current_index = assigned[1].unwrap();
    let (game_name, label) = GAME_PHASE_NAMES[current_index as usize];

    // Calculate days until each future phase by scanning forward from today (index 2+)
    let mut days_until = Vec::new();
    let mut seen_phases = std::collections::HashSet::new();
    seen_phases.insert(current_index);

    // Walk forward through the window, then extend if needed
    // First pass: use the 31-day window (indices 2..31 = tomorrow through day+29)
    for i in 2..31 {
        let phase_idx = assigned[i].unwrap();
        if !seen_phases.contains(&phase_idx) {
            seen_phases.insert(phase_idx);
            let (gn, lb) = GAME_PHASE_NAMES[phase_idx as usize];
            let day_offset = (i as u32) - 1; // days from today
            days_until.push(DaysUntilPhase {
                game_phase: gn.to_string(),
                label: lb.to_string(),
                days: day_offset,
            });
        }
    }

    // If we haven't found all 7 remaining phases in the window, extend the search.
    // This handles the rare case where a phase change falls beyond 29 days out.
    if seen_phases.len() < 8 {
        for extra_day in 30..=60u32 {
            let date = today + chrono::Duration::days(extra_day as i64);
            let md = moon_for_date(date.year(), date.month(), date.day());
            // Simple classification for extended days (no 3-day span logic needed
            // since we only need the first occurrence of each missing phase)
            let phase_idx = if md.waning {
                if md.illuminated >= 0.5 {
                    5
                } else {
                    7
                }
            } else if md.illuminated <= 0.5 {
                1
            } else {
                3
            };
            if !seen_phases.contains(&phase_idx) {
                seen_phases.insert(phase_idx);
                let (gn, lb) = GAME_PHASE_NAMES[phase_idx as usize];
                days_until.push(DaysUntilPhase {
                    game_phase: gn.to_string(),
                    label: lb.to_string(),
                    days: extra_day,
                });
            }
            if seen_phases.len() >= 8 {
                break;
            }
        }
    }

    Ok(MoonPhaseResult {
        game_phase: game_name.to_string(),
        label: label.to_string(),
        phase_index: current_index,
        days_until,
    })
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

/// Return the set of item IDs that are purchasable from NPCs (Barter or Vendor sources).
/// Used by crafting projects to accurately split shopping lists.
#[tauri::command]
pub async fn get_vendor_purchasable_item_ids(
    state: State<'_, GameDataState>,
) -> Result<Vec<u32>, String> {
    let data = state.read().await;
    let mut ids = Vec::new();

    for (&item_id, source_info) in &data.sources.items {
        let is_vendor = source_info.entries.iter().any(|e| {
            e.source_type == "Barter" || e.source_type == "Vendor"
        });
        if is_vendor {
            ids.push(item_id);
        }
    }

    Ok(ids)
}

/// A compact item summary for vendor inventory display.
#[derive(serde::Serialize, Clone)]
pub struct VendorItemSummary {
    pub item_id: u32,
    pub name: String,
    pub value: Option<f32>,
    pub icon_id: Option<u32>,
}

/// Get the items sold by a specific NPC (Vendor + Barter sources).
/// Returns resolved item summaries for display.
#[tauri::command]
pub async fn get_npc_vendor_items(
    npc_key: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<VendorItemSummary>, String> {
    let data = state.read().await;

    let item_ids = match data.vendor_items_by_npc.get(&npc_key) {
        Some(ids) => ids,
        None => return Ok(vec![]),
    };

    let mut results: Vec<VendorItemSummary> = item_ids
        .iter()
        .filter_map(|&id| {
            let item = data.items.get(&id)?;
            Some(VendorItemSummary {
                item_id: id,
                name: item.name.clone(),
                value: item.value,
                icon_id: item.icon_id,
            })
        })
        .collect();

    results.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(results)
}

/// Get the count of vendor items per NPC (for showing counts in NPC list).
#[tauri::command]
pub async fn get_vendor_item_counts(
    state: State<'_, GameDataState>,
) -> Result<std::collections::HashMap<String, usize>, String> {
    let data = state.read().await;
    let counts: std::collections::HashMap<String, usize> = data
        .vendor_items_by_npc
        .iter()
        .map(|(npc_key, items)| (npc_key.clone(), items.len()))
        .collect();
    Ok(counts)
}

/// Get the NPC keys and names that sell/barter a given item (for item tooltips).
#[derive(serde::Serialize, Clone)]
pub struct VendorNpcSummary {
    pub npc_key: String,
    pub name: String,
}

#[tauri::command]
pub async fn get_vendors_for_item(
    item_id: u32,
    state: State<'_, GameDataState>,
) -> Result<Vec<VendorNpcSummary>, String> {
    let data = state.read().await;
    let npc_keys = match data.vendors_for_item.get(&item_id) {
        Some(keys) => keys,
        None => return Ok(vec![]),
    };
    let mut results: Vec<VendorNpcSummary> = npc_keys
        .iter()
        .filter_map(|key| {
            let npc = data.npcs.get(key)?;
            Some(VendorNpcSummary {
                npc_key: key.clone(),
                name: npc.name.clone(),
            })
        })
        .collect();
    results.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(results)
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

    if let Some(tier_info) = info.tiers.get(&tier_key) {
        for desc in &tier_info.effect_descs {
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
    /// Equipment slots this power can appear on
    pub slots: Vec<String>,
}

/// Get all eligible TSys powers for a given equipment slot, filtered by skills and target level.
/// Returns powers belonging to skill_primary, skill_secondary, or generic (no skill).
#[tauri::command]
pub async fn get_tsys_powers_for_slot(
    _skill_primary: Option<String>,
    _skill_secondary: Option<String>,
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
        if info.tiers.is_empty() {
            continue;
        }
        let tiers = &info.tiers;

        // Collect all available tiers for this power
        let mut available_tiers: Vec<TsysTierSummary> = Vec::new();
        for (tier_key, tier) in tiers {
            let min_level = tier.min_level.unwrap_or(0) as i64;
            let max_level = tier.max_level.unwrap_or(999) as i64;
            let min_rarity = tier.min_rarity.clone();
            let skill_level_prereq = tier.skill_level_prereq.map(|v| v as i64);

            let mut tier_icon_id: Option<u32> = None;
            let effects: Vec<String> = tier.effect_descs
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

        // Helper to resolve effects from a tier
        let resolve_tier = |tier: &TsysTierInfo, data: &GameData| -> (Vec<String>, Vec<String>, Option<u32>) {
            let raw = tier.effect_descs.clone();
            let mut icon: Option<u32> = None;
            let resolved: Vec<String> = raw.iter().map(|desc| {
                if let Some(r) = resolve_single_effect(desc, data) {
                    if icon.is_none() { icon = r.icon_id; }
                    r.formatted
                } else {
                    desc.clone()
                }
            }).collect();
            (resolved, raw, icon)
        };

        for (tier_key, tier) in tiers {
            let min_level = tier.min_level.unwrap_or(0) as i64;
            let max_level = tier.max_level.unwrap_or(999) as i64;

            if target_level >= min_level && target_level <= max_level {
                best_tier_id = Some(tier_key.clone());
                best_min_rarity = tier.min_rarity.clone();
                best_skill_prereq = tier.skill_level_prereq.map(|v| v as i64);
                let (eff, raw, icon) = resolve_tier(tier, &data);
                best_effects = eff;
                best_raw_effects = raw;
                best_icon_id = icon;
                break; // First matching tier wins
            }
        }

        // If no tier matched, try to find the highest tier at or below target level
        if best_tier_id.is_none() {
            let mut best_min: i64 = 0;
            for (tier_key, tier) in tiers {
                let min_level = tier.min_level.unwrap_or(0) as i64;
                if min_level <= target_level && min_level >= best_min {
                    best_min = min_level;
                    best_tier_id = Some(tier_key.clone());
                    best_min_rarity = tier.min_rarity.clone();
                    best_skill_prereq = tier.skill_level_prereq.map(|v| v as i64);
                    let (eff, raw, icon) = resolve_tier(tier, &data);
                    best_effects = eff;
                    best_raw_effects = raw;
                    best_icon_id = icon;
                }
            }
        }

        // Final fallback: if target level is below all tiers, pick the lowest tier
        // (higher-level mods can go on lower-level gear, raising the equip requirement)
        if best_tier_id.is_none() {
            let mut lowest_min: i64 = i64::MAX;
            for (tier_key, tier) in tiers {
                let min_level = tier.min_level.unwrap_or(0) as i64;
                if min_level < lowest_min {
                    lowest_min = min_level;
                    best_tier_id = Some(tier_key.clone());
                    best_min_rarity = tier.min_rarity.clone();
                    best_skill_prereq = tier.skill_level_prereq.map(|v| v as i64);
                    let (eff, raw, icon) = resolve_tier(tier, &data);
                    best_effects = eff;
                    best_raw_effects = raw;
                    best_icon_id = icon;
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
                slots: info.slots.iter().map(|s| {
                    // Map CDN slot names back to display names
                    match s.as_str() {
                        "Waist" => "Belt".to_string(),
                        "OffHandShield" => "Off Hand".to_string(),
                        "OffHand" => "Off Hand".to_string(),
                        "MainHand" => "Main Hand".to_string(),
                        _ => s.clone(),
                    }
                }).collect(),
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
    pub tiers: HashMap<String, TsysTierInfo>,
    pub is_unavailable: Option<bool>,
    pub is_hidden_from_transmutation: Option<bool>,
    pub tier_count: usize,
    pub raw_json: Value,
}

impl TsysBrowserEntry {
    fn from_entry(key: &str, info: &TsysClientInfo) -> Self {
        let tier_count = info.tiers.len();
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

/// Get all quests that require a specific moon phase (via RequirementsToSustain).
#[tauri::command]
pub async fn get_quests_by_moon_phase(
    moon_phase: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<QuestInfo>, String> {
    let data = state.read().await;

    fn has_moon_phase(req: &serde_json::Value, phase: &str) -> bool {
        if let Some(t) = req.get("T").and_then(|v| v.as_str()) {
            if t == "MoonPhase" {
                if let Some(mp) = req.get("MoonPhase").and_then(|v| v.as_str()) {
                    return mp == phase;
                }
            }
        }
        false
    }

    let mut results: Vec<QuestInfo> = data
        .quests
        .values()
        .filter(|quest| {
            if let Some(reqs) = quest.raw.get("RequirementsToSustain") {
                // Can be a single object or an array
                if let Some(arr) = reqs.as_array() {
                    arr.iter().any(|r| has_moon_phase(r, &moon_phase))
                } else {
                    has_moon_phase(reqs, &moon_phase)
                }
            } else {
                false
            }
        })
        .cloned()
        .collect();

    results.sort_by(|a, b| {
        let a_name = a.raw.get("DisplayName").and_then(|v| v.as_str()).unwrap_or("");
        let b_name = b.raw.get("DisplayName").and_then(|v| v.as_str()).unwrap_or("");
        a_name.cmp(b_name)
    });

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

// ── CP-consuming recipe queries ───────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct CpRecipeOption {
    pub recipe_id: u32,
    pub recipe_name: String,
    pub icon_id: Option<u32>,
    pub skill: Option<String>,
    pub skill_level_req: Option<f32>,
    pub cp_cost: u32,
    /// "shamanic_infusion" or "crafting_enhancement"
    pub effect_type: String,
    /// For shamanic: TSys power name; for enhancements: the enhancement function name
    pub effect_key: String,
    /// Human-readable description of the effect
    pub effect_description: String,
}

/// Get all CP-consuming recipes (shamanic infusion + crafting enhancements) applicable to a slot.
#[tauri::command]
pub async fn get_cp_recipes_for_slot(
    equip_slot: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<CpRecipeOption>, String> {
    let data = state.read().await;
    let mut results = Vec::new();

    // Map frontend slot name to CDN slot name
    let cdn_slot = match equip_slot.as_str() {
        "Belt" => "Waist",
        "OffHand" => "OffHand",
        _ => &equip_slot,
    };

    for recipe in data.recipes.values() {
        for effect in &recipe.result_effects {
            // Shamanic Infusion: "AddItemTSysPower(PowerName,Tier)"
            if let Some(inner) = effect.strip_prefix("AddItemTSysPower(").and_then(|s| s.strip_suffix(')')) {
                let parts: Vec<&str> = inner.split(',').collect();
                if parts.len() != 2 { continue; }
                let power_name = parts[0].trim();

                // Look up the TSys power to check skill and slot compatibility
                let tsys_entry = data.tsys.client_info.values()
                    .find(|info| info.internal_name.as_deref() == Some(power_name));

                let Some(tsys_info) = tsys_entry else { continue };

                // Must be a ShamanicInfusion power
                if tsys_info.skill.as_deref() != Some("ShamanicInfusion") { continue; }

                // Must be compatible with this equipment slot
                let slot_matches = tsys_info.slots.iter().any(|s| {
                    match cdn_slot {
                        "OffHand" => s == "OffHand" || s == "OffHandShield",
                        _ => s == cdn_slot,
                    }
                });
                if !slot_matches { continue; }

                // Resolve effect description from the power's tier data
                let tier_str = parts[1].trim();
                let tier_key = format!("id_{}", tier_str);
                let description = tsys_info.tiers.get(&tier_key)
                    .map(|tier| {
                        tier.effect_descs.iter()
                            .filter_map(|desc| resolve_single_effect(desc, &data))
                            .map(|r| r.formatted)
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_else(|| recipe.description.clone().unwrap_or_default());

                results.push(CpRecipeOption {
                    recipe_id: recipe.id,
                    recipe_name: recipe.name.clone(),
                    icon_id: recipe.icon_id,
                    skill: recipe.skill.clone(),
                    skill_level_req: recipe.skill_level_req,
                    cp_cost: 100, // All shamanic infusions cost 100 CP
                    effect_type: "shamanic_infusion".to_string(),
                    effect_key: power_name.to_string(),
                    effect_description: description,
                });
            }

            // Crafting Enhancement: "CraftingEnhanceItemXxx(value,cpCost)"
            if effect.starts_with("CraftingEnhanceItem") {
                if let Some(inner) = effect.find('(').and_then(|start| {
                    effect.strip_suffix(')').map(|s| &s[start + 1..])
                }) {
                    let parts: Vec<&str> = inner.split(',').collect();
                    if parts.len() != 2 { continue; }
                    let cp_cost: u32 = match parts[1].trim().parse() {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    // Crafting enhancements apply to armor slots only
                    let armor_slots = ["Head", "Chest", "Legs", "Hands", "Feet"];
                    if !armor_slots.contains(&cdn_slot) { continue; }

                    // Filter by item type in recipe name — recipes target specific
                    // body parts via name patterns like "...Cloth Pants", "...Leather Shirt"
                    let name_lower = recipe.name.to_lowercase();
                    let slot_matches = match cdn_slot {
                        "Head" => name_lower.contains("helmet") || name_lower.contains("head"),
                        "Chest" => name_lower.contains("shirt") || name_lower.contains("chest"),
                        "Legs" => name_lower.contains("pants") || name_lower.contains("legs"),
                        "Hands" => name_lower.contains("gloves") || name_lower.contains("hands") || name_lower.contains("gauntlets"),
                        "Feet" => name_lower.contains("boots") || name_lower.contains("feet") || name_lower.contains("shoes"),
                        _ => false,
                    };
                    if !slot_matches { continue; }

                    // Use recipe description as the effect description
                    let description = recipe.description.clone().unwrap_or_else(|| recipe.name.clone());

                    results.push(CpRecipeOption {
                        recipe_id: recipe.id,
                        recipe_name: recipe.name.clone(),
                        icon_id: recipe.icon_id,
                        skill: recipe.skill.clone(),
                        skill_level_req: recipe.skill_level_req,
                        cp_cost,
                        effect_type: "crafting_enhancement".to_string(),
                        effect_key: effect.clone(),
                        effect_description: description,
                    });
                }
            }
        }
    }

    // Sort by effect_type (shamanic first), then by name
    results.sort_by(|a, b| a.effect_type.cmp(&b.effect_type).then(a.recipe_name.cmp(&b.recipe_name)));

    Ok(results)
}

// ── Precomputed TSys ↔ Ability lookups ─────────────────────────────────────

/// Batch lookup: given a list of TSys keys or internal names, return the ability IDs each one affects.
/// Accepts either CDN keys (e.g., "power_12345") or internal names (e.g., "SwordDamageBoost").
/// Uses the precomputed index built at CDN load time — O(1) per key.
#[tauri::command]
pub async fn get_tsys_ability_map(
    tsys_keys: Vec<String>,
    state: State<'_, GameDataState>,
) -> Result<HashMap<String, Vec<u32>>, String> {
    let data = state.read().await;
    let mut result = HashMap::new();
    for key in &tsys_keys {
        // Try direct CDN key lookup first
        if let Some(ids) = data.tsys_to_abilities.get(key) {
            result.insert(key.clone(), ids.clone());
            continue;
        }
        // Try as internal_name via precomputed index — O(1)
        if let Some(cdn_key) = data.tsys_internal_name_index.get(key) {
            if let Some(ids) = data.tsys_to_abilities.get(cdn_key) {
                result.insert(key.clone(), ids.clone());
            }
        }
    }
    Ok(result)
}

// ── Ability ↔ TSys cross-reference commands (legacy, still used by data browser) ──

/// Extract all `{TOKEN}` keys from a list of effect_desc strings.
fn extract_effect_tokens(effect_descs: &[String]) -> Vec<String> {
    let mut tokens = Vec::new();
    for desc in effect_descs {
        if !desc.starts_with('{') { continue; }
        // Format: "{TOKEN}{VALUE}" — extract the TOKEN part
        let parts: Vec<&str> = desc.split('{').filter(|s| !s.is_empty()).collect();
        if let Some(first) = parts.first() {
            let token = first.trim_end_matches('}');
            if !token.is_empty() {
                tokens.push(token.to_string());
            }
        }
    }
    tokens
}

/// Extract `<icon=NNNN>` IDs from text-format effect descriptions.
fn extract_icon_ids(effect_descs: &[String]) -> std::collections::HashSet<u32> {
    let mut icons = std::collections::HashSet::new();
    for desc in effect_descs {
        // Only process text-format (non-token) descriptions
        if desc.starts_with('{') { continue; }
        let mut remaining = desc.as_str();
        while let Some(start) = remaining.find("<icon=") {
            let after = &remaining[start + 6..];
            if let Some(end) = after.find('>') {
                if let Ok(id) = after[..end].parse::<u32>() {
                    icons.insert(id);
                }
            }
            remaining = &remaining[start + 6..];
        }
    }
    icons
}

/// Collect all attribute tokens referenced by an ability's PvE/PvP combat stats,
/// including DoT attribute arrays from the raw JSON.
fn collect_ability_attribute_tokens(ability: &AbilityInfo) -> std::collections::HashSet<String> {
    let mut tokens = std::collections::HashSet::new();

    for stats in [&ability.pve, &ability.pvp].into_iter().flatten() {
        for t in &stats.attributes_that_delta_damage { tokens.insert(t.clone()); }
        for t in &stats.attributes_that_mod_base_damage { tokens.insert(t.clone()); }
        for t in &stats.attributes_that_mod_damage { tokens.insert(t.clone()); }
        for t in &stats.attributes_that_mod_crit_damage { tokens.insert(t.clone()); }
        for t in &stats.attributes_that_delta_power_cost { tokens.insert(t.clone()); }
        for t in &stats.attributes_that_mod_power_cost { tokens.insert(t.clone()); }
        for t in &stats.attributes_that_delta_rage { tokens.insert(t.clone()); }
        for t in &stats.attributes_that_mod_rage { tokens.insert(t.clone()); }
        for t in &stats.attributes_that_delta_taunt { tokens.insert(t.clone()); }
        for t in &stats.attributes_that_mod_taunt { tokens.insert(t.clone()); }

        // Also check DoTs in the extra field (e.g., DoTs[].AttributesThatDelta)
        if let Some(dots) = stats.extra.get("DoTs").and_then(|v| v.as_array()) {
            for dot in dots {
                for key in ["AttributesThatDelta", "AttributesThatMod", "AttributesThatDeltaDamage",
                            "AttributesThatModDamage", "AttributesThatModBaseDamage"] {
                    if let Some(arr) = dot.get(key).and_then(|v| v.as_array()) {
                        for val in arr {
                            if let Some(s) = val.as_str() {
                                tokens.insert(s.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    tokens
}

/// Check if `text` contains `name` as an exact ability reference, not as a prefix
/// of a longer ability name. `longer_names` contains all known ability base names
/// that start with `name + " "`.
fn text_contains_ability_name(text: &str, name: &str, longer_names: &[&str]) -> bool {
    let mut search_from = 0;
    while let Some(idx) = text[search_from..].find(name) {
        let abs_idx = search_from + idx;
        // Check: does a longer ability name also match at this position?
        let is_prefix_of_longer = longer_names.iter().any(|longer| {
            text[abs_idx..].starts_with(longer)
        });
        if !is_prefix_of_longer {
            return true;
        }
        // Move past this match and keep looking — the short name might appear
        // elsewhere in the text on its own
        search_from = abs_idx + name.len();
    }
    false
}

/// Check if a TSys mod matches an ability via any of:
/// 1. Attribute token overlap (PvE/PvP stats ↔ {TOKEN}{VALUE} effect descs)
/// 2. Icon ID match (ability icon_id in <icon=NNN> tags in text effect descs)
/// 3. Ability display name appears in text effect descs (with prefix disambiguation)
fn tsys_matches_ability(
    info: &TsysClientInfo,
    ability_tokens: &std::collections::HashSet<String>,
    ability_name: &str,
    ability_icon_id: Option<u32>,
    longer_ability_names: &[&str],
) -> bool {
    // Collect all effect_descs across tiers (check one tier for text match, all for tokens)
    for tier in info.tiers.values() {
        // 1. Token matching
        let tier_tokens = extract_effect_tokens(&tier.effect_descs);
        if tier_tokens.iter().any(|t| ability_tokens.contains(t)) {
            return true;
        }
    }

    // For text matching, only check one representative tier (they all reference the same abilities)
    if let Some(tier) = info.tiers.values().next() {
        // 2. Icon ID matching
        if let Some(icon_id) = ability_icon_id {
            let tier_icons = extract_icon_ids(&tier.effect_descs);
            if tier_icons.contains(&icon_id) {
                return true;
            }
        }

        // 3. Ability display name matching in text descriptions
        // Only match names that are 4+ chars to avoid false positives on short names
        if ability_name.len() >= 4 {
            for desc in &tier.effect_descs {
                if desc.starts_with('{') { continue; }
                let (clean, _) = strip_icon_tags(desc);
                if text_contains_ability_name(&clean, ability_name, longer_ability_names) {
                    return true;
                }
            }
        }
    }

    false
}

/// Lightweight TSys summary returned for ability cross-references.
#[derive(Debug, serde::Serialize, Clone)]
pub struct TsysAbilityXref {
    pub key: String,
    pub internal_name: Option<String>,
    pub skill: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub slots: Vec<String>,
    pub tier_count: usize,
    /// Resolved effect descriptions from the highest tier
    pub top_tier_effects: Vec<String>,
}

fn resolve_top_tier_effects(info: &TsysClientInfo, data: &GameData) -> Vec<String> {
    let top_tier = info.tiers.iter()
        .max_by_key(|(k, _)| {
            k.strip_prefix("id_").and_then(|n| n.parse::<u32>().ok()).unwrap_or(0)
        })
        .map(|(_, tier)| tier);

    top_tier
        .map(|tier| {
            tier.effect_descs.iter()
                .map(|desc| {
                    resolve_single_effect(desc, data)
                        .map(|r| r.formatted)
                        .unwrap_or_else(|| strip_icon_tags(desc).0)
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Get all TSys mods that affect a given ability.
/// Matches via attribute tokens, icon IDs, and ability name text matching.
#[tauri::command]
pub async fn get_tsys_for_ability(
    ability_id: u32,
    state: State<'_, GameDataState>,
) -> Result<Vec<TsysAbilityXref>, String> {
    let data = state.read().await;
    let ability = data.abilities.get(&ability_id).ok_or("Ability not found")?;

    // Monster abilities don't have treasure system effects
    if ability.keywords.iter().any(|k| k == "Lint_MonsterAbility") {
        return Ok(Vec::new());
    }

    let ability_tokens = collect_ability_attribute_tokens(ability);

    // Get the base ability name (without trailing number) for text matching
    // e.g., "Reckless Slam 3" → "Reckless Slam"
    let base_display_name = ability.name.trim_end_matches(|c: char| c.is_ascii_digit())
        .trim()
        .to_string();

    // Build list of longer ability names that start with this one + space
    // e.g., for "Pound" → ["Pound To Slag"]
    // This prevents "Pound" from matching text about "Pound To Slag"
    let all_base_names: Vec<String> = data.abilities.values()
        .map(|a| a.name.trim_end_matches(|c: char| c.is_ascii_digit()).trim().to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let longer_names: Vec<&str> = all_base_names.iter()
        .filter(|n| n.len() > base_display_name.len() && n.starts_with(&format!("{} ", base_display_name)))
        .map(|n| n.as_str())
        .collect();

    let mut results: Vec<TsysAbilityXref> = Vec::new();

    for (key, info) in &data.tsys.client_info {
        if tsys_matches_ability(info, &ability_tokens, &base_display_name, ability.icon_id, &longer_names) {
            let top_tier_effects = resolve_top_tier_effects(info, &data);

            results.push(TsysAbilityXref {
                key: key.clone(),
                internal_name: info.internal_name.clone(),
                skill: info.skill.clone(),
                prefix: info.prefix.clone(),
                suffix: info.suffix.clone(),
                slots: info.slots.clone(),
                tier_count: info.tiers.len(),
                top_tier_effects,
            });
        }
    }

    results.sort_by(|a, b| {
        let a_name = a.internal_name.as_deref().unwrap_or(&a.key);
        let b_name = b.internal_name.as_deref().unwrap_or(&b.key);
        a_name.cmp(b_name)
    });

    Ok(results)
}

/// Lightweight ability summary returned for TSys cross-references.
#[derive(Debug, serde::Serialize, Clone)]
pub struct AbilityTsysXref {
    pub id: u32,
    pub name: String,
    pub icon_id: Option<u32>,
    pub skill: Option<String>,
    pub level: Option<f32>,
    pub internal_name: Option<String>,
}

/// Get all abilities affected by a given TSys mod.
/// Matches via attribute tokens, icon IDs, and ability name text matching.
#[tauri::command]
pub async fn get_abilities_for_tsys(
    tsys_key: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<AbilityTsysXref>, String> {
    let data = state.read().await;
    let info = data.tsys.client_info.get(&tsys_key).ok_or("TSys entry not found")?;

    // Collect attribute tokens from {TOKEN}{VALUE} format effect descs
    let mut tsys_tokens = std::collections::HashSet::new();
    for tier in info.tiers.values() {
        for token in extract_effect_tokens(&tier.effect_descs) {
            tsys_tokens.insert(token);
        }
    }

    // Collect icon IDs from text-format effect descs
    let mut tsys_icon_ids = std::collections::HashSet::new();
    for tier in info.tiers.values() {
        tsys_icon_ids.extend(extract_icon_ids(&tier.effect_descs));
    }

    // Collect ability display names from text-format effect descs
    // Build a set of all known ability base names for matching
    let mut ability_base_names: std::collections::HashMap<String, Vec<u32>> = std::collections::HashMap::new();
    for ab in data.abilities.values() {
        let base = ab.name.trim_end_matches(|c: char| c.is_ascii_digit()).trim().to_string();
        if base.len() >= 4 {
            ability_base_names.entry(base).or_default().push(ab.id);
        }
    }

    let mut matched_ability_ids = std::collections::HashSet::new();

    // 1. Token matching: find abilities whose attribute tokens overlap
    if !tsys_tokens.is_empty() {
        for ability in data.abilities.values() {
            let ability_tokens = collect_ability_attribute_tokens(ability);
            if ability_tokens.iter().any(|t| tsys_tokens.contains(t)) {
                matched_ability_ids.insert(ability.id);
            }
        }
    }

    // 2. Icon ID matching: find abilities whose icon_id appears in text descs
    if !tsys_icon_ids.is_empty() {
        for ability in data.abilities.values() {
            if let Some(icon_id) = ability.icon_id {
                if tsys_icon_ids.contains(&icon_id) {
                    matched_ability_ids.insert(ability.id);
                }
            }
        }
    }

    // 3. Text name matching: find ability names in text effect descs
    // Sort names longest-first so longer names get priority over shorter prefixes
    let mut sorted_names: Vec<&String> = ability_base_names.keys().collect();
    sorted_names.sort_by(|a, b| b.len().cmp(&a.len()));

    if let Some(tier) = info.tiers.values().next() {
        for desc in &tier.effect_descs {
            if desc.starts_with('{') { continue; }
            let (clean, _) = strip_icon_tags(desc);
            for base_name in &sorted_names {
                // Build longer-names list for this specific name
                let longer: Vec<&str> = sorted_names.iter()
                    .filter(|n| n.len() > base_name.len() && n.starts_with(&format!("{} ", base_name)))
                    .map(|n| n.as_str())
                    .collect();
                if text_contains_ability_name(&clean, base_name, &longer) {
                    if let Some(ids) = ability_base_names.get(base_name.as_str()) {
                        for id in ids {
                            matched_ability_ids.insert(*id);
                        }
                    }
                }
            }
        }
    }

    // Deduplicate by base ability name, keep highest-level version
    let mut results: Vec<AbilityTsysXref> = Vec::new();
    let mut seen_base_names = std::collections::HashSet::new();

    for &ability_id in &matched_ability_ids {
        if let Some(ability) = data.abilities.get(&ability_id) {
            let base_name = ability.internal_name.as_deref()
                .unwrap_or(&ability.name)
                .trim_end_matches(char::is_numeric)
                .to_string();

            if seen_base_names.contains(&base_name) { continue; }

            // Find the highest-level version of this ability
            let best = data.abilities.values()
                .filter(|a| {
                    let a_base = a.internal_name.as_deref()
                        .unwrap_or(&a.name)
                        .trim_end_matches(char::is_numeric)
                        .to_string();
                    a_base == base_name
                })
                .max_by(|a, b| {
                    a.level.unwrap_or(0.0).partial_cmp(&b.level.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap_or(ability);

            seen_base_names.insert(base_name);
            results.push(AbilityTsysXref {
                id: best.id,
                name: best.name.clone(),
                icon_id: best.icon_id,
                skill: best.skill.clone(),
                level: best.level,
                internal_name: best.internal_name.clone(),
            });
        }
    }

    results.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(results)
}

// ── Recipe Item Finder ────────────────────────────────────────────────────────

/// A recipe item found in the player's inventory, with context about whether
/// the recipes it teaches are already known or if the player meets the skill
/// requirements to learn them.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RecipeItemMatch {
    /// The item's type ID from CDN
    pub item_id: u32,
    /// Display name of the item
    pub item_name: String,
    /// Icon ID for display
    pub icon_id: Option<u32>,
    /// How many copies are in inventory
    pub stack_size: i32,
    /// Recipe keys this item bestows (e.g., "recipe_1234")
    pub bestow_recipe_keys: Vec<String>,
    /// Display names of the recipes this item bestows
    pub bestow_recipe_names: Vec<String>,
    /// Whether ALL bestowed recipes are already known
    pub all_known: bool,
    /// Whether the player meets skill requirements to use this item
    pub meets_requirements: bool,
    /// Skill requirements that are NOT met (skill_name -> (required, current))
    pub unmet_requirements: Vec<UnmetRequirement>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UnmetRequirement {
    pub skill_name: String,
    pub required: u32,
    pub current: u32,
}

/// Find recipe items in the player's inventory and classify them.
/// Returns items that bestow recipes, indicating whether recipes are already
/// known and whether the player meets the skill requirements.
#[tauri::command]
pub async fn find_recipe_items_in_inventory(
    character_name: String,
    server_name: String,
    db: State<'_, DbPool>,
    state: State<'_, GameDataState>,
) -> Result<Vec<RecipeItemMatch>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;
    let data = state.read().await;

    // Get all inventory items
    let mut inv_stmt = conn.prepare(
        "SELECT item_name, item_type_id, stack_size
         FROM game_state_inventory
         WHERE character_name = ?1 AND server_name = ?2"
    ).map_err(|e| format!("Query error: {e}"))?;

    let inv_items: Vec<(String, Option<i64>, i32)> = inv_stmt
        .query_map(rusqlite::params![character_name, server_name], |row: &rusqlite::Row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<i64>>(1)?, row.get::<_, i32>(2)?))
        })
        .map_err(|e| format!("Query error: {e}"))?
        .filter_map(|r: Result<_, rusqlite::Error>| r.ok())
        .collect();

    // Get known recipes
    let mut recipe_stmt = conn.prepare(
        "SELECT recipe_id FROM game_state_recipes
         WHERE character_name = ?1 AND server_name = ?2 AND completion_count > 0"
    ).map_err(|e| format!("Query error: {e}"))?;

    let known_recipe_ids: std::collections::HashSet<u32> = recipe_stmt
        .query_map(rusqlite::params![character_name, server_name], |row: &rusqlite::Row| {
            row.get::<_, i64>(0).map(|id| id as u32)
        })
        .map_err(|e| format!("Query error: {e}"))?
        .filter_map(|r: Result<_, rusqlite::Error>| r.ok())
        .collect();

    // Get current skill levels
    let mut skill_stmt = conn.prepare(
        "SELECT skill_name, level FROM game_state_skills
         WHERE character_name = ?1 AND server_name = ?2"
    ).map_err(|e| format!("Query error: {e}"))?;

    let skill_levels: std::collections::HashMap<String, u32> = skill_stmt
        .query_map(rusqlite::params![character_name, server_name], |row: &rusqlite::Row| {
            let name: String = row.get(0)?;
            let level: i32 = row.get(1)?;
            Ok((name, level as u32))
        })
        .map_err(|e| format!("Query error: {e}"))?
        .filter_map(|r: Result<_, rusqlite::Error>| r.ok())
        .collect();

    // Aggregate by item type — sum stack sizes for same type_id
    let mut type_stacks = std::collections::HashMap::<u32, (String, i32)>::new();
    for entry in &inv_items {
        if let Some(type_id) = entry.1 {
            let id = type_id as u32;
            let ts = type_stacks.entry(id).or_insert_with(|| (entry.0.clone(), 0));
            ts.1 += entry.2;
        }
    }

    let mut results: Vec<RecipeItemMatch> = Vec::new();

    for (type_id, (_item_name, total_stack)) in &type_stacks {
        let item = match data.items.get(type_id) {
            Some(item) => item,
            None => continue,
        };

        let bestow_recipes = match &item.bestow_recipes {
            Some(recipes) if !recipes.is_empty() => recipes,
            _ => continue,
        };

        let mut recipe_keys: Vec<String> = Vec::new();
        let mut recipe_names: Vec<String> = Vec::new();
        let mut all_known = true;

        for recipe_val in bestow_recipes {
            if let Some(recipe_key) = recipe_val.as_str() {
                recipe_keys.push(recipe_key.to_string());
                // Parse recipe ID from key like "recipe_1234"
                let recipe_id: Option<u32> = recipe_key
                    .split('_')
                    .last()
                    .and_then(|s| s.parse().ok());

                if let Some(rid) = recipe_id {
                    if !known_recipe_ids.contains(&rid) {
                        all_known = false;
                    }
                    // Get recipe display name
                    if let Some(recipe_info) = data.recipes.get(&rid) {
                        recipe_names.push(recipe_info.name.clone());
                    } else {
                        recipe_names.push(recipe_key.to_string());
                    }
                }
            }
        }

        // Check skill requirements
        let mut unmet_requirements: Vec<UnmetRequirement> = Vec::new();
        if let Some(ref skill_reqs) = item.skill_reqs {
            if let Some(obj) = skill_reqs.as_object() {
                for (skill_ref, level_val) in obj {
                    let required = level_val.as_u64().unwrap_or(0) as u32;
                    // Resolve skill name
                    let skill_name = data
                        .resolve_skill(skill_ref)
                        .map(|s| s.name.clone())
                        .unwrap_or_else(|| skill_ref.clone());
                    let current = skill_levels.get(&skill_name).copied().unwrap_or(0);
                    if current < required {
                        unmet_requirements.push(UnmetRequirement {
                            skill_name,
                            required,
                            current,
                        });
                    }
                }
            }
        }

        let meets_requirements = unmet_requirements.is_empty();

        results.push(RecipeItemMatch {
            item_id: *type_id,
            item_name: item.name.clone(),
            icon_id: item.icon_id,
            stack_size: *total_stack,
            bestow_recipe_keys: recipe_keys,
            bestow_recipe_names: recipe_names,
            all_known,
            meets_requirements,
            unmet_requirements,
        });
    }

    // Sort: known duplicates first, then learnable, then others
    results.sort_by(|a, b| {
        b.all_known.cmp(&a.all_known)
            .then(b.meets_requirements.cmp(&a.meets_requirements))
            .then(a.item_name.cmp(&b.item_name))
    });

    Ok(results)
}
