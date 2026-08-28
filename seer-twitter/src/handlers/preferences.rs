//! Twitter preferences (Nitter instance URL).

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
    config::default_nitter,
    entities::TwitterPreferences,
    forms::PreferencesForm,
    preferences::{load_preferences, save_preferences},
    state::TwitterState,
    templates::TwitterPreferencesPage,
};

fn prefs_page(prefs: TwitterPreferences, error: String) -> TwitterPreferencesPage {
    TwitterPreferencesPage {
        nitter_instance_url: if prefs.nitter_instance_url.trim().is_empty() {
            default_nitter()
        } else {
            prefs.nitter_instance_url
        },
        error,
    }
}

fn empty_prefs() -> TwitterPreferences {
    TwitterPreferences {
        id: 1,
        created_at: None,
        updated_at: None,
        nitter_instance_url: default_nitter(),
    }
}

/// GET `/seer-twitter/preferences`
pub async fn get(
    Cap(state): Cap<TwitterState>,
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

/// POST `/seer-twitter/preferences`
pub async fn post(
    Cap(state): Cap<TwitterState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireStaff(ctx): RequireStaff,
    htmx: Htmx,
    HtmlFormBody(form): HtmlFormBody<PreferencesForm>,
) -> Response {
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let prefs = TwitterPreferences {
        id: 1,
        created_at: None,
        updated_at: None,
        nitter_instance_url: form.nitter_instance_url.trim().to_string(),
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
