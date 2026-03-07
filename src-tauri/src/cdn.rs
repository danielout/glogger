/// CDN access for Project: Gorgon data files and icons.
///
/// Version check:  GET http://client.projectgorgon.com/fileversion.txt  → integer
/// Data files:     https://cdn.projectgorgon.com/v{ver}/data/{file}.json
/// Icons:          https://cdn.projectgorgon.com/v{ver}/icons/icon_{id}.png

use std::path::{Path, PathBuf};
use tokio::fs;

const VERSION_URL: &str = "http://client.projectgorgon.com/fileversion.txt";
const CDN_BASE: &str = "https://cdn.projectgorgon.com";

/// The JSON files we want to download and cache locally.
pub const DATA_FILES: &[&str] = &[
    "items",
    "skills",
    "abilities",
    "recipes",
    "effects",
    "tsysclientinfo",
    "areas",
    "npcs",
];

// ── Version helpers ──────────────────────────────────────────────────────────

/// Fetch the current live version number from the game's version endpoint.
pub async fn fetch_remote_version() -> Result<u32, String> {
    let text = reqwest::get(VERSION_URL)
        .await
        .map_err(|e| format!("Version fetch failed: {e}"))?
        .text()
        .await
        .map_err(|e| format!("Version read failed: {e}"))?;

    text.trim()
        .parse::<u32>()
        .map_err(|e| format!("Version parse failed '{text}': {e}"))
}

/// Read the cached version number we last downloaded, if any.
pub async fn read_cached_version(cache_dir: &Path) -> Option<u32> {
    let path = cache_dir.join("version.txt");
    let text = fs::read_to_string(&path).await.ok()?;
    text.trim().parse().ok()
}

/// Write the version number we just downloaded to disk.
pub async fn write_cached_version(cache_dir: &Path, version: u32) -> Result<(), String> {
    let path = cache_dir.join("version.txt");
    fs::write(&path, version.to_string())
        .await
        .map_err(|e| format!("Failed to write cached version: {e}"))
}

// ── Data file helpers ─────────────────────────────────────────────────────────

/// Download one JSON data file and write it to `cache_dir/{name}.json`.
pub async fn download_data_file(
    version: u32,
    name: &str,
    cache_dir: &Path,
) -> Result<(), String> {
    let url = format!("{CDN_BASE}/v{version}/data/{name}.json");
    let bytes = reqwest::get(&url)
        .await
        .map_err(|e| format!("Download failed for {name}: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("Read failed for {name}: {e}"))?;

    let dest = cache_dir.join(format!("{name}.json"));
    fs::write(&dest, &bytes)
        .await
        .map_err(|e| format!("Write failed for {name}: {e}"))
}

/// Download all DATA_FILES for the given version into `cache_dir`.
/// Runs downloads concurrently.
pub async fn download_all_data_files(version: u32, cache_dir: &Path) -> Result<(), String> {
    let tasks: Vec<_> = DATA_FILES
        .iter()
        .map(|name| {
            let cache_dir = cache_dir.to_path_buf();
            let name = *name;
            tokio::spawn(async move { download_data_file(version, name, &cache_dir).await })
        })
        .collect();

    for task in tasks {
        task.await
            .map_err(|e| format!("Task panicked: {e}"))?
            .map_err(|e| e)?;
    }

    Ok(())
}

/// Path for a cached data file.
#[allow(dead_code)]
pub fn data_file_path(cache_dir: &Path, name: &str) -> PathBuf {
    cache_dir.join(format!("{name}.json"))
}

// ── Icon helpers ──────────────────────────────────────────────────────────────

/// Path for a cached icon file.
pub fn icon_path(icon_dir: &Path, icon_id: u32) -> PathBuf {
    icon_dir.join(format!("icon_{icon_id}.png"))
}

/// Returns the local path for an icon, fetching and caching it first if needed.
pub async fn get_or_fetch_icon(
    version: u32,
    icon_id: u32,
    icon_dir: &Path,
) -> Result<PathBuf, String> {
    let path = icon_path(icon_dir, icon_id);

    if path.exists() {
        return Ok(path);
    }

    let url = format!("{CDN_BASE}/v{version}/icons/icon_{icon_id}.png");
    let bytes = reqwest::get(&url)
        .await
        .map_err(|e| format!("Icon fetch failed for {icon_id}: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("Icon read failed for {icon_id}: {e}"))?;

    fs::write(&path, &bytes)
        .await
        .map_err(|e| format!("Icon write failed for {icon_id}: {e}"))?;

    Ok(path)
}