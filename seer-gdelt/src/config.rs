//! Runtime GDELT settings (loaded from DB preferences).

#[derive(Debug, Clone, Default)]
pub struct SeerGdeltConfig {
    pub project_id: String,
}
