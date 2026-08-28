//! Singleton Twitter preferences (`id = 1`).

use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr, EntityTrait};

use crate::config::{default_nitter, SeerTwitterConfig};
use crate::entities::{
    TwitterPreferences,
    twitter_preferences::{self, Entity as PrefsEntity},
};

/// Load singleton preferences row (`id = 1`), creating it if missing.
pub async fn load_preferences(db: &DatabaseConnection) -> Result<TwitterPreferences, DbErr> {
    if let Some(prefs) = PrefsEntity::find_by_id(1).one(db).await? {
        return Ok(prefs);
    }

    let now = Utc::now();
    let model = twitter_preferences::ActiveModel {
        id: Set(1),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        nitter_instance_url: Set(default_nitter()),
    };
    model.insert(db).await
}

/// Persist preferences fields onto the singleton row.
pub async fn save_preferences(
    db: &DatabaseConnection,
    prefs: TwitterPreferences,
) -> Result<TwitterPreferences, DbErr> {
    let mut am: twitter_preferences::ActiveModel = load_preferences(db).await?.into();
    am.nitter_instance_url = Set(nitter_or_default(&prefs.nitter_instance_url));
    am.updated_at = Set(Some(Utc::now()));
    am.update(db).await
}

/// Runtime config resolved from the preferences row.
pub async fn resolved_config(db: &DatabaseConnection) -> Result<SeerTwitterConfig, DbErr> {
    let prefs = load_preferences(db).await?;
    Ok(config_from_prefs(&prefs))
}

pub fn config_from_prefs(prefs: &TwitterPreferences) -> SeerTwitterConfig {
    SeerTwitterConfig {
        nitter_instance_url: nitter_or_default(&prefs.nitter_instance_url),
    }
}

fn nitter_or_default(raw: &str) -> String {
    let url = raw.trim();
    if url.is_empty() {
        default_nitter()
    } else {
        url.to_string()
    }
}
