//! GDELT preferences (BigQuery project ID).

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
    entities::GdeltPreferences,
    forms::PreferencesForm,
    preferences::{load_preferences, save_preferences},
    state::GdeltState,
    templates::GdeltPreferencesPage,
};

fn prefs_page(prefs: GdeltPreferences, error: String) -> GdeltPreferencesPage {
    GdeltPreferencesPage {
        project_id: prefs.project_id,
        error,
    }
}

fn empty_prefs() -> GdeltPreferences {
    GdeltPreferences {
        id: 1,
        created_at: None,
        updated_at: None,
        project_id: String::new(),
    }
}

/// GET `/seer-gdelt/preferences`
pub async fn get(
    Cap(state): Cap<GdeltState>,
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

/// POST `/seer-gdelt/preferences`
pub async fn post(
    Cap(state): Cap<GdeltState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireStaff(ctx): RequireStaff,
    htmx: Htmx,
    HtmlFormBody(form): HtmlFormBody<PreferencesForm>,
) -> Response {
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let prefs = GdeltPreferences {
        id: 1,
        created_at: None,
        updated_at: None,
        project_id: form.project_id.trim().to_string(),
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
