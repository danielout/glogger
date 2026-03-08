use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ── Raw CDN shapes ────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct RawAbilityKeywordInfo {
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Parsed structs (app shape) ───────────────────────────────────────────────

#[derive(Debug, Serialize, Clone, Default)]
pub struct AbilityKeywordInfo {
    pub raw: serde_json::Value,
}

// ── Parse function ───────────────────────────────────────────────────────────

pub fn parse(json: &str) -> Result<HashMap<String, AbilityKeywordInfo>, String> {
    // abilitykeywords.json is an array, not a map - just store empty for now
    let _raw: Vec<serde_json::Value> = serde_json::from_str(json)
        .map_err(|e| format!("abilitykeywords.json: parse error at line {}, col {}: {e}", e.line(), e.column()))?;

    // Return empty map since we don't need this data yet
    Ok(HashMap::new())
}
