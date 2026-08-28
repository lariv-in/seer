//! Intel preferences (geocoding + Gemini models).

use axum::{
    response::{IntoResponse, Response},
};

use lariv_rs::{
    html_form::HtmlFormBody,
    components::{SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireStaff,
    web::{html_built_page_or_app_layout, Htmx},
};

use crate::{
    config::{default_embedding_model, default_summary_model, default_title_model},
    entities::IntelPreferences,
    forms::PreferencesForm,
    preferences::{
        embed_model_choices, generate_model_choices, load_preferences, save_preferences,
    },
    state::IntelState,
    templates::IntelPreferencesPage,
};

async fn prefs_page(prefs: IntelPreferences, error: String) -> IntelPreferencesPage {
    let title_model = if prefs.title_model.trim().is_empty() {
        default_title_model()
    } else {
        prefs.title_model.clone()
    };
    let summary_model = if prefs.summary_model.trim().is_empty() {
        default_summary_model()
    } else {
        prefs.summary_model.clone()
    };
    let embedding_model = if prefs.embedding_model.trim().is_empty() {
        default_embedding_model()
    } else {
        prefs.embedding_model.clone()
    };

    // One list call per capability; keep each current selection in its dropdown.
    let (mut title_model_choices, generate_list_error) =
        generate_model_choices(&prefs.api_key, &title_model).await;
    let mut summary_model_choices = title_model_choices.clone();
    ensure_choice(&mut summary_model_choices, &summary_model);
    // If title was missing from the API list, generate_model_choices already inserted it;
    // still ensure when cloning order differs (no-op if already present).
    ensure_choice(&mut title_model_choices, &title_model);

    let (embedding_model_choices, embed_list_error) =
        embed_model_choices(&prefs.api_key, &embedding_model).await;

    let list_error = generate_list_error
        .or(embed_list_error)
        .unwrap_or_default();
    let error = [error, list_error]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    IntelPreferencesPage {
        geocoding_api_key: prefs.geocoding_api_key,
        api_key: prefs.api_key,
        title_model,
        title_model_choices,
        summary_model,
        summary_model_choices,
        embedding_model,
        embedding_model_choices,
        error,
    }
}

fn ensure_choice(choices: &mut Vec<(String, String)>, current: &str) {
    if !current.is_empty() && !choices.iter().any(|(id, _)| id == current) {
        choices.insert(0, (current.to_string(), current.to_string()));
    }
}

fn empty_prefs() -> IntelPreferences {
    IntelPreferences {
        id: 1,
        created_at: None,
        updated_at: None,
        geocoding_api_key: String::new(),
        title_model: default_title_model(),
        summary_model: default_summary_model(),
        embedding_model: default_embedding_model(),
        api_key: String::new(),
    }
}

/// GET `/seer-intel/preferences`
pub async fn get(
    Cap(state): Cap<IntelState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireStaff(ctx): RequireStaff,
    htmx: Htmx,
) -> Response {
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let prefs = match load_preferences(&state.db).await {
        Ok(p) => p,
        Err(e) => {
            let page = prefs_page(empty_prefs(), e.to_string()).await;
            return html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response();
        }
    };
    let page = prefs_page(prefs, String::new()).await;
    html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response()
}

/// POST `/seer-intel/preferences`
pub async fn post(
    Cap(state): Cap<IntelState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireStaff(ctx): RequireStaff,
    htmx: Htmx,
    HtmlFormBody(form): HtmlFormBody<PreferencesForm>,
) -> Response {
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let prefs = IntelPreferences {
        id: 1,
        created_at: None,
        updated_at: None,
        geocoding_api_key: form.geocoding_api_key.trim().to_string(),
        title_model: form.title_model.trim().to_string(),
        summary_model: form.summary_model.trim().to_string(),
        embedding_model: form.embedding_model.trim().to_string(),
        api_key: form.api_key.trim().to_string(),
    };

    match save_preferences(&state.db, prefs.clone()).await {
        Ok(saved) => {
            let page = prefs_page(saved, String::new()).await;
            html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response()
        }
        Err(e) => {
            let page = prefs_page(prefs, e.to_string()).await;
            html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response()
        }
    }
}
