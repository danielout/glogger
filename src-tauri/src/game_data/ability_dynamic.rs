use serde::Serialize;

// ── Parsed structs (app shape) ───────────────────────────────────────────────

#[derive(Debug, Serialize, Clone, Default)]
pub struct AbilityDynamicData {
    pub dots: serde_json::Value,
    pub special_values: serde_json::Value,
}

impl AbilityDynamicData {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn parse(dots_json: &str, special_json: &str) -> Result<Self, String> {
        let dots = serde_json::from_str(dots_json)
            .map_err(|e| format!("abilitydynamicdots.json: {e}"))?;
        let special_values = serde_json::from_str(special_json)
            .map_err(|e| format!("abilitydynamicspecialvalues.json: {e}"))?;
        Ok(Self { dots, special_values })
    }
}
