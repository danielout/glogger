use serde::Serialize;

// ── Parsed structs (app shape) ───────────────────────────────────────────────

#[derive(Debug, Serialize, Clone, Default)]
pub struct SourcesData {
    pub abilities: serde_json::Value,
    pub items: serde_json::Value,
    pub recipes: serde_json::Value,
}

impl SourcesData {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn parse(abilities_json: &str, items_json: &str, recipes_json: &str) -> Result<Self, String> {
        let abilities = serde_json::from_str(abilities_json)
            .map_err(|e| format!("sources_abilities.json: {e}"))?;
        let items = serde_json::from_str(items_json)
            .map_err(|e| format!("sources_items.json: {e}"))?;
        let recipes = serde_json::from_str(recipes_json)
            .map_err(|e| format!("sources_recipes.json: {e}"))?;
        Ok(Self { abilities, items, recipes })
    }
}
