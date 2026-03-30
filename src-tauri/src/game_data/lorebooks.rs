use serde::Serialize;

// ── Parsed structs (app shape) ───────────────────────────────────────────────

#[derive(Debug, Serialize, Clone, Default)]
pub struct LorebookData {
    pub books: serde_json::Value,
    pub info: serde_json::Value,
}

impl LorebookData {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn parse(books_json: &str, info_json: &str) -> Result<Self, String> {
        let books = serde_json::from_str(books_json).map_err(|e| format!("lorebooks.json: {e}"))?;
        let info =
            serde_json::from_str(info_json).map_err(|e| format!("lorebookinfo.json: {e}"))?;
        Ok(Self { books, info })
    }
}
