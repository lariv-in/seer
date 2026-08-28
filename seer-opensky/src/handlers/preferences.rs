//! OpenSky preferences (OAuth client credentials).

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
    entities::OpenskyPreferences,
    forms::PreferencesForm,
    preferences::{load_preferences, save_preferences},
    state::OpenskyState,
    templates::OpenskyPreferencesPage,
};

fn prefs_page(prefs: OpenskyPreferences, error: String) -> OpenskyPreferencesPage {
    OpenskyPreferencesPage {
        client_id: prefs.client_id,
        client_secret: prefs.client_secret,
        error,
    }
}

fn empty_prefs() -> OpenskyPreferences {
    OpenskyPreferences {
        id: 1,
        created_at: None,
        updated_at: None,
        client_id: String::new(),
        client_secret: String::new(),
    }
}

/// GET `/seer-opensky/preferences`
pub async fn get(
    Cap(state): Cap<OpenskyState>,
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

/// POST `/seer-opensky/preferences`
pub async fn post(
    Cap(state): Cap<OpenskyState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireStaff(ctx): RequireStaff,
    htmx: Htmx,
    HtmlFormBody(form): HtmlFormBody<PreferencesForm>,
) -> Response {
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let prefs = OpenskyPreferences {
        id: 1,
        created_at: None,
        updated_at: None,
        client_id: form.client_id.trim().to_string(),
        client_secret: form.client_secret.trim().to_string(),
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
