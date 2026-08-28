//! Runtime AIS stream settings (loaded from DB preferences).

#[derive(Debug, Clone, Default)]
pub struct SeerAisstreamConfig {
    pub enabled: bool,
    pub api_key: String,
}
