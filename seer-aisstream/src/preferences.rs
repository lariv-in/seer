//! Singleton AIS stream preferences (`id = 1`).

use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr, EntityTrait};

use crate::config::SeerAisstreamConfig;
use crate::entities::{
    AisstreamPreferences,
    aisstream_preferences::{self, Entity as PrefsEntity},
};

/// Load singleton preferences row (`id = 1`), creating it if missing.
pub async fn load_preferences(db: &DatabaseConnection) -> Result<AisstreamPreferences, DbErr> {
    if let Some(prefs) = PrefsEntity::find_by_id(1).one(db).await? {
        return Ok(prefs);
    }

    let now = Utc::now();
    let model = aisstream_preferences::ActiveModel {
        id: Set(1),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        enabled: Set(false),
        api_key: Set(String::new()),
    };
    model.insert(db).await
}

/// Persist preferences fields onto the singleton row.
pub async fn save_preferences(
    db: &DatabaseConnection,
    prefs: AisstreamPreferences,
) -> Result<AisstreamPreferences, DbErr> {
    let mut am: aisstream_preferences::ActiveModel = load_preferences(db).await?.into();
    am.enabled = Set(prefs.enabled);
    am.api_key = Set(prefs.api_key);
    am.updated_at = Set(Some(Utc::now()));
    am.update(db).await
}

/// Runtime config resolved from the preferences row.
pub async fn resolved_config(db: &DatabaseConnection) -> Result<SeerAisstreamConfig, DbErr> {
    let prefs = load_preferences(db).await?;
    Ok(config_from_prefs(&prefs))
}

pub fn config_from_prefs(prefs: &AisstreamPreferences) -> SeerAisstreamConfig {
    SeerAisstreamConfig {
        enabled: prefs.enabled,
        api_key: prefs.api_key.clone(),
    }
}
