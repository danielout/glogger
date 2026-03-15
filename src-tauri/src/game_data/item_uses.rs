use std::collections::HashMap;
use serde::Serialize;
use serde_json::Value;

// ── Parsed structs (app shape) ───────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct ItemUseInfo {
    pub recipes_that_use_item: Vec<u32>,

    // Full raw JSON
    pub raw_json: Value,
}

// ── Parse function ───────────────────────────────────────────────────────────

pub fn parse(json: &str) -> Result<HashMap<String, ItemUseInfo>, String> {
    let raw: HashMap<String, Value> = serde_json::from_str(json).map_err(|e| {
        format!("itemuses.json: parse error at line {}, col {}: {e}", e.line(), e.column())
    })?;

    Ok(raw.into_iter()
        .map(|(key, value)| {
            let recipes = value.get("RecipesThatUseItem")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_u64().map(|n| n as u32)).collect())
                .unwrap_or_default();

            (key, ItemUseInfo {
                recipes_that_use_item: recipes,
                raw_json: value,
            })
        })
        .collect())
}
