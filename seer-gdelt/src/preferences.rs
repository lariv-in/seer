//! Singleton GDELT preferences (`id = 1`).

use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr, EntityTrait};

use crate::config::SeerGdeltConfig;
use crate::entities::{
    GdeltPreferences,
    gdelt_preferences::{self, Entity as PrefsEntity},
};

/// Load singleton preferences row (`id = 1`), creating it if missing.
pub async fn load_preferences(db: &DatabaseConnection) -> Result<GdeltPreferences, DbErr> {
    if let Some(prefs) = PrefsEntity::find_by_id(1).one(db).await? {
        return Ok(prefs);
    }

    let now = Utc::now();
    let model = gdelt_preferences::ActiveModel {
        id: Set(1),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        project_id: Set(String::new()),
    };
    model.insert(db).await
}

/// Persist preferences fields onto the singleton row.
pub async fn save_preferences(
    db: &DatabaseConnection,
    prefs: GdeltPreferences,
) -> Result<GdeltPreferences, DbErr> {
    let mut am: gdelt_preferences::ActiveModel = load_preferences(db).await?.into();
    am.project_id = Set(prefs.project_id);
    am.updated_at = Set(Some(Utc::now()));
    am.update(db).await
}

/// Runtime config resolved from the preferences row.
pub async fn resolved_config(db: &DatabaseConnection) -> Result<SeerGdeltConfig, DbErr> {
    let prefs = load_preferences(db).await?;
    Ok(config_from_prefs(&prefs))
}

pub fn config_from_prefs(prefs: &GdeltPreferences) -> SeerGdeltConfig {
    SeerGdeltConfig {
        project_id: prefs.project_id.clone(),
    }
}
