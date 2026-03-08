use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ── Raw CDN shapes ────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct RawDirectedGoalInfo {
    #[serde(rename = "Id")]
    pub id: Option<u32>,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Parsed structs (app shape) ───────────────────────────────────────────────

#[derive(Debug, Serialize, Clone, Default)]
pub struct DirectedGoalInfo {
    pub raw: serde_json::Value,
}

// ── Parse function ───────────────────────────────────────────────────────────

pub fn parse(json: &str) -> Result<HashMap<String, DirectedGoalInfo>, String> {
    // directedgoals.json is an array, not a map
    let raw: Vec<serde_json::Value> = serde_json::from_str(json)
        .map_err(|e| format!("directedgoals.json: parse error at line {}, col {}: {e}", e.line(), e.column()))?;

    // Convert to HashMap keyed by Id (as string) if available, otherwise use index
    let mut result = HashMap::with_capacity(raw.len());
    for (idx, value) in raw.into_iter().enumerate() {
        let key = value.get("Id")
            .and_then(|v| v.as_u64())
            .map(|id| id.to_string())
            .unwrap_or_else(|| idx.to_string());

        result.insert(key, DirectedGoalInfo { raw: value });
    }

    Ok(result)
}
