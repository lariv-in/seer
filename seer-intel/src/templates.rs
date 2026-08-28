use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonSubmit, FormOpts, MainContentKey, ShellChrome, button_submit, form, form_hx_post_url,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    template::{RenderAppPane, RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
};
use seer_common::templates::{
    app_scaffold, detail_crumbs, list_crumbs, menu_item, scaffold_main, scaffold_pane, seer_menu,
};

#[allow(unused_imports)]
use crate::SeerIntelTag;
use crate::{
    forms::{PreferencesForm, PreferencesFormField},
    routes::{IntelListRouteTag, IntelMapRouteTag, PrefsGetRouteTag, PrefsPostRouteTag},
};

fn intel_menu(active: &str) -> Markup {
    let list_url = IntelListRouteTag.url();
    let map_url = IntelMapRouteTag.url();
    let prefs_url = PrefsGetRouteTag.url();
    seer_menu(
        "Intel",
        html! {
            (menu_item("All Intel", &list_url, active == "intel"))
            (menu_item("Map", &map_url, active == "map"))
            (menu_item("Preferences", &prefs_url, active == "preferences"))
        },
    )
}

fn prefs_crumbs() -> Markup {
    let list_url = IntelListRouteTag.url();
    detail_crumbs("Intel", &list_url, "Preferences")
}

seer_common::seer_scaffold_page!(
    IntelListPage,
    "Intel — Seer",
    intel_menu("intel"),
    list_crumbs("Intel")
);
seer_common::seer_scaffold_page!(
    IntelMapPage,
    "Intel Map — Seer",
    intel_menu("map"),
    list_crumbs("Intel")
);
seer_common::seer_scaffold_page!(
    IntelDetailPage,
    "Intel — Seer",
    intel_menu("intel"),
    list_crumbs("Intel")
);

#[derive(Generic)]
pub struct IntelPreferencesPage {
    pub geocoding_api_key: String,
    pub api_key: String,
    pub title_model: String,
    pub title_model_choices: Vec<(String, String)>,
    pub summary_model: String,
    pub summary_model_choices: Vec<(String, String)>,
    pub embedding_model: String,
    pub embedding_model_choices: Vec<(String, String)>,
    pub error: String,
}

impl IntelPreferencesPage {
    fn body(&self) -> Markup {
        form(FormOpts {
            attrs: form_hx_post_url::<MainContentKey>(&PrefsPostRouteTag.path())
                .set("hx-swap", "outerHTML"),
            title: "Intel Preferences",
            subtitle: "Geocoding and Gemini models used for intel ingest and search",
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: PreferencesForm::render_inputs(
                &FormCtx::form::<PreferencesForm>()
                    .value(
                        PreferencesFormField::GeocodingApiKey,
                        self.geocoding_api_key.as_str(),
                    )
                    .value(PreferencesFormField::ApiKey, self.api_key.as_str())
                    .value(PreferencesFormField::TitleModel, self.title_model.as_str())
                    .choices(
                        PreferencesFormField::TitleModel,
                        &self.title_model_choices,
                    )
                    .value(
                        PreferencesFormField::SummaryModel,
                        self.summary_model.as_str(),
                    )
                    .choices(
                        PreferencesFormField::SummaryModel,
                        &self.summary_model_choices,
                    )
                    .value(
                        PreferencesFormField::EmbeddingModel,
                        self.embedding_model.as_str(),
                    )
                    .choices(
                        PreferencesFormField::EmbeddingModel,
                        &self.embedding_model_choices,
                    ),
            ),
            actions: html! {
                (button_submit(ButtonSubmit {
                    label: "Save Preferences",
                    ..Default::default()
                }))
            },
            ..Default::default()
        })
    }
}

impl RenderAppPane for IntelPreferencesPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(intel_menu("preferences"), prefs_crumbs(), self.body())
    }

    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(prefs_crumbs(), self.body())
    }
}

impl RenderTemplate for IntelPreferencesPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Intel Preferences — Seer",
            chrome,
            intel_menu("preferences"),
            prefs_crumbs(),
            self.body(),
        )
    }
}

lariv_rs::define_register_items! {
    plugin: SeerIntelTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        ListIdx: IntelListPageTag => IntelListPage,
        MapIdx: IntelMapPageTag => IntelMapPage,
        DetailIdx: IntelDetailPageTag => IntelDetailPage,
        PrefsIdx: IntelPreferencesPageTag => IntelPreferencesPage,
    ]
}
