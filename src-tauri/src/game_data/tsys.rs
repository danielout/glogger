use serde::Serialize;

// ── Parsed structs (app shape) ───────────────────────────────────────────────

#[derive(Debug, Serialize, Clone, Default)]
pub struct TsysData {
    pub client_info: serde_json::Value,
    pub profiles: serde_json::Value,
}

impl TsysData {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn parse(client_info_json: &str, profiles_json: &str) -> Result<Self, String> {
        let client_info = serde_json::from_str(client_info_json)
            .map_err(|e| format!("tsysclientinfo.json: {e}"))?;
        let profiles = serde_json::from_str(profiles_json)
            .map_err(|e| format!("tsysprofiles.json: {e}"))?;
        Ok(Self { client_info, profiles })
    }
}
