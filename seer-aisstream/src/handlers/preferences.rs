//! AIS stream preferences (enabled + API key).

use axum::{
    response::{IntoResponse, Response},
};

use lariv_rs::{
    html_form::HtmlFormBody,
    components::{SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireStaff,
    web::{Htmx, html_built_page_or_app_layout},
};

use crate::{
    entities::AisstreamPreferences,
    forms::PreferencesForm,
    preferences::{load_preferences, save_preferences},
    state::AisstreamState,
    templates::AisPreferencesPage,
};

fn prefs_page(prefs: AisstreamPreferences, error: String) -> AisPreferencesPage {
    AisPreferencesPage {
        enabled: prefs.enabled,
        api_key: prefs.api_key,
        error,
    }
}

fn empty_prefs() -> AisstreamPreferences {
    AisstreamPreferences {
        id: 1,
        created_at: None,
        updated_at: None,
        enabled: false,
        api_key: String::new(),
    }
}

/// GET `/seer-aisstream/preferences`
pub async fn get(
    Cap(state): Cap<AisstreamState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireStaff(ctx): RequireStaff,
    htmx: Htmx,
) -> Response {
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let prefs = match load_preferences(&state.db).await {
        Ok(p) => p,
        Err(e) => {
            let page = prefs_page(empty_prefs(), e.to_string());
            return html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response();
        }
    };
    let page = prefs_page(prefs, String::new());
    html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response()
}

/// POST `/seer-aisstream/preferences`
pub async fn post(
    Cap(state): Cap<AisstreamState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireStaff(ctx): RequireStaff,
    htmx: Htmx,
    HtmlFormBody(form): HtmlFormBody<PreferencesForm>,
) -> Response {
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let prefs = AisstreamPreferences {
        id: 1,
        created_at: None,
        updated_at: None,
        enabled: form.enabled,
        api_key: form.api_key.trim().to_string(),
    };

    match save_preferences(&state.db, prefs.clone()).await {
        Ok(saved) => {
            let page = prefs_page(saved, String::new());
            html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response()
        }
        Err(e) => {
            let page = prefs_page(prefs, e.to_string());
            html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response()
        }
    }
}
