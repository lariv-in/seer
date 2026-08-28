//! Singleton Intel preferences (`id = 1`).

use chrono::Utc;
use lariv_rs::genai::GenaiClient;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr, EntityTrait};

use crate::config::{
    default_embedding_model, default_summary_model, default_title_model, SeerIntelConfig,
};
use crate::entities::{
    IntelPreferences,
    intel_preferences::{self, Entity as PrefsEntity},
};

/// Load singleton preferences row (`id = 1`), creating it if missing.
pub async fn load_preferences(db: &DatabaseConnection) -> Result<IntelPreferences, DbErr> {
    if let Some(prefs) = PrefsEntity::find_by_id(1).one(db).await? {
        return Ok(prefs);
    }

    let now = Utc::now();
    let model = intel_preferences::ActiveModel {
        id: Set(1),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        geocoding_api_key: Set(String::new()),
        title_model: Set(default_title_model()),
        summary_model: Set(default_summary_model()),
        embedding_model: Set(default_embedding_model()),
        api_key: Set(String::new()),
    };
    model.insert(db).await
}

/// Persist preferences fields onto the singleton row.
pub async fn save_preferences(
    db: &DatabaseConnection,
    prefs: IntelPreferences,
) -> Result<IntelPreferences, DbErr> {
    let mut am: intel_preferences::ActiveModel = load_preferences(db).await?.into();
    am.geocoding_api_key = Set(prefs.geocoding_api_key);
    am.title_model = Set(model_or_default(&prefs.title_model, &default_title_model()));
    am.summary_model = Set(model_or_default(
        &prefs.summary_model,
        &default_summary_model(),
    ));
    am.embedding_model = Set(model_or_default(
        &prefs.embedding_model,
        &default_embedding_model(),
    ));
    am.api_key = Set(prefs.api_key);
    am.updated_at = Set(Some(Utc::now()));
    am.update(db).await
}

/// Runtime config resolved from the preferences row.
pub async fn resolved_config(db: &DatabaseConnection) -> Result<SeerIntelConfig, DbErr> {
    let prefs = load_preferences(db).await?;
    Ok(config_from_prefs(&prefs))
}

pub fn config_from_prefs(prefs: &IntelPreferences) -> SeerIntelConfig {
    SeerIntelConfig {
        geocoding_api_key: prefs.geocoding_api_key.clone(),
        title_model: model_or_default(&prefs.title_model, &default_title_model()),
        summary_model: model_or_default(&prefs.summary_model, &default_summary_model()),
        embedding_model: model_or_default(&prefs.embedding_model, &default_embedding_model()),
        api_key: prefs.api_key.clone(),
    }
}

fn model_or_default(raw: &str, fallback: &str) -> String {
    let model = raw.trim();
    if model.is_empty() {
        fallback.to_string()
    } else {
        model.to_string()
    }
}

/// Gemini API key: preferences row first, then `GOOGLE_API_KEY` / `GEMINI_API_KEY`.
pub fn api_key_from_prefs_or_env(prefs_key: &str) -> String {
    let key = prefs_key.trim();
    if !key.is_empty() {
        return key.to_string();
    }
    std::env::var("GOOGLE_API_KEY")
        .or_else(|_| std::env::var("GEMINI_API_KEY"))
        .unwrap_or_default()
}

/// `(id, display_name)` pairs for generateContent models, plus an optional list error.
pub async fn generate_model_choices(
    api_key: &str,
    current: &str,
) -> (Vec<(String, String)>, Option<String>) {
    gemini_model_choices(api_key, current, true).await
}

/// `(id, display_name)` pairs for embedContent models, plus an optional list error.
pub async fn embed_model_choices(
    api_key: &str,
    current: &str,
) -> (Vec<(String, String)>, Option<String>) {
    gemini_model_choices(api_key, current, false).await
}

async fn gemini_model_choices(
    api_key: &str,
    current: &str,
    generate: bool,
) -> (Vec<(String, String)>, Option<String>) {
    let key = api_key_from_prefs_or_env(api_key);
    let (mut choices, list_error) = if key.is_empty() {
        (
            Vec::new(),
            Some("Save a Gemini API key to load the model list.".to_string()),
        )
    } else {
        let client = GenaiClient::new(key, String::new());
        let listed = if generate {
            client.list_generate_content_models().await
        } else {
            client.list_embed_content_models().await
        };
        match listed {
            Ok(models) => (models, None),
            Err(e) => (
                Vec::new(),
                Some(format!("Could not list Gemini models: {e}")),
            ),
        }
    };
    if !current.is_empty() && !choices.iter().any(|(id, _)| id == current) {
        choices.insert(0, (current.to_string(), current.to_string()));
    }
    (choices, list_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn empty_api_key_does_not_list_models() {
        let (choices, err) = generate_model_choices("", "gemini-2.0-flash").await;
        assert_eq!(
            choices,
            vec![("gemini-2.0-flash".to_string(), "gemini-2.0-flash".to_string())]
        );
        assert!(
            err.as_deref()
                .is_some_and(|e| e.contains("Save a Gemini API key")),
            "{err:?}"
        );
    }
}
