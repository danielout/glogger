/// Tauri commands for CDN data management and game data queries.
/// These are the invoke() endpoints the Vue frontend calls.

use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;

use crate::cdn;
use crate::game_data::{self, GameData, ItemInfo, SkillInfo, AbilityInfo, RecipeInfo};

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
#[tauri::command]
pub async fn search_items(
    query: String,
    limit: Option<usize>,
    state: State<'_, GameDataState>,
) -> Result<Vec<ItemInfo>, String> {
    let limit = limit.unwrap_or(20);
    let q = query.to_lowercase();
    let data = state.read().await;
    let mut results: Vec<ItemInfo> = data
        .items
        .values()
        .filter(|item| item.name.to_lowercase().contains(&q))
        .take(limit)
        .cloned()
        .collect();
    results.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(results)
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