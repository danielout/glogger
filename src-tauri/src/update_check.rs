use serde::{Deserialize, Serialize};

const GITHUB_RELEASES_URL: &str =
    "https://api.github.com/repos/danielout/glogger-release/releases/latest";

#[derive(Debug, Serialize, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct UpdateInfo {
    pub available: bool,
    pub latest_version: String,
    pub download_url: String,
    pub release_notes: Option<String>,
}

/// Check whether a newer version is available on GitHub.
///
/// Compares the latest release tag (e.g. "v0.5.0") against the current app version.
/// Returns structured info the frontend can display.
#[tauri::command]
pub async fn check_for_update(app: tauri::AppHandle) -> Result<UpdateInfo, String> {
    let current = app
        .config()
        .version
        .clone()
        .unwrap_or_else(|| "0.0.0".into());

    let client = reqwest::Client::builder()
        .user_agent("glogger-update-check")
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let release: GitHubRelease = client
        .get(GITHUB_RELEASES_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to reach GitHub: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse release info: {e}"))?;

    let latest = release.tag_name.trim_start_matches('v').to_string();

    let available = is_newer(&latest, &current);

    Ok(UpdateInfo {
        available,
        latest_version: latest,
        download_url: release.html_url,
        release_notes: release.body,
    })
}

/// Simple semver comparison: returns true if `latest` is newer than `current`.
fn is_newer(latest: &str, current: &str) -> bool {
    let parse = |s: &str| -> Vec<u32> {
        s.split('.')
            .filter_map(|p| p.parse().ok())
            .collect()
    };
    let l = parse(latest);
    let c = parse(current);
    for i in 0..l.len().max(c.len()) {
        let lv = l.get(i).copied().unwrap_or(0);
        let cv = c.get(i).copied().unwrap_or(0);
        if lv > cv {
            return true;
        }
        if lv < cv {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer() {
        assert!(is_newer("0.5.0", "0.4.5"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(is_newer("0.4.6", "0.4.5"));
        assert!(!is_newer("0.4.5", "0.4.5"));
        assert!(!is_newer("0.4.4", "0.4.5"));
        assert!(!is_newer("0.3.0", "0.4.5"));
    }
}
