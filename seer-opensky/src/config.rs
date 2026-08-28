//! Runtime OpenSky settings (loaded from DB preferences).

#[derive(Debug, Clone, Default)]
pub struct SeerOpenskyConfig {
    pub client_id: String,
    pub client_secret: String,
}
