//! Runtime twitter settings (loaded from DB preferences).

#[derive(Debug, Clone)]
pub struct SeerTwitterConfig {
    pub nitter_instance_url: String,
}

impl Default for SeerTwitterConfig {
    fn default() -> Self {
        Self {
            nitter_instance_url: default_nitter(),
        }
    }
}

pub fn default_nitter() -> String {
    "https://nitter.net".into()
}
