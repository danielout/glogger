use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::parse_id_map;

// ── Raw CDN shapes ────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct RawPlayerTitleInfo {
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Parsed structs (app shape) ───────────────────────────────────────────────

#[derive(Debug, Serialize, Clone, Default)]
pub struct PlayerTitleInfo {
    pub raw: serde_json::Value,
}

// ── Parse function ───────────────────────────────────────────────────────────

pub fn parse(json: &str) -> Result<HashMap<u32, PlayerTitleInfo>, String> {
    let raw: HashMap<u32, serde_json::Value> = parse_id_map(json, "playertitles.json")?;
    Ok(raw.into_iter()
        .map(|(k, v)| (k, PlayerTitleInfo { raw: v }))
        .collect())
}
