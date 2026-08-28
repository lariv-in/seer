//! Singleton OpenSky preferences (`id = 1`).

use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr, EntityTrait};

use crate::config::SeerOpenskyConfig;
use crate::entities::{
    OpenskyPreferences,
    opensky_preferences::{self, Entity as PrefsEntity},
};

/// Load singleton preferences row (`id = 1`), creating it if missing.
pub async fn load_preferences(db: &DatabaseConnection) -> Result<OpenskyPreferences, DbErr> {
    if let Some(prefs) = PrefsEntity::find_by_id(1).one(db).await? {
        return Ok(prefs);
    }

    let now = Utc::now();
    let model = opensky_preferences::ActiveModel {
        id: Set(1),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        client_id: Set(String::new()),
        client_secret: Set(String::new()),
    };
    model.insert(db).await
}

/// Persist preferences fields onto the singleton row.
pub async fn save_preferences(
    db: &DatabaseConnection,
    prefs: OpenskyPreferences,
) -> Result<OpenskyPreferences, DbErr> {
    let mut am: opensky_preferences::ActiveModel = load_preferences(db).await?.into();
    am.client_id = Set(prefs.client_id);
    am.client_secret = Set(prefs.client_secret);
    am.updated_at = Set(Some(Utc::now()));
    am.update(db).await
}

/// Runtime config resolved from the preferences row.
pub async fn resolved_config(db: &DatabaseConnection) -> Result<SeerOpenskyConfig, DbErr> {
    let prefs = load_preferences(db).await?;
    Ok(config_from_prefs(&prefs))
}

pub fn config_from_prefs(prefs: &OpenskyPreferences) -> SeerOpenskyConfig {
    SeerOpenskyConfig {
        client_id: prefs.client_id.clone(),
        client_secret: prefs.client_secret.clone(),
    }
}
