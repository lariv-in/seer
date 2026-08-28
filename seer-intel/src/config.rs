//! Runtime intel settings (loaded from DB preferences).

#[derive(Debug, Clone)]
pub struct SeerIntelConfig {
    pub geocoding_api_key: String,
    pub title_model: String,
    pub summary_model: String,
    pub embedding_model: String,
    /// Optional Gemini API key for embeddings (falls back to GOOGLE_API_KEY / GEMINI_API_KEY).
    pub api_key: String,
}

impl Default for SeerIntelConfig {
    fn default() -> Self {
        Self {
            geocoding_api_key: String::new(),
            title_model: default_title_model(),
            summary_model: default_summary_model(),
            embedding_model: default_embedding_model(),
            api_key: String::new(),
        }
    }
}

pub fn default_title_model() -> String {
    "gemini-2.0-flash".into()
}
pub fn default_summary_model() -> String {
    "gemini-2.0-flash".into()
}
pub fn default_embedding_model() -> String {
    "gemini-embedding-001".into()
}
