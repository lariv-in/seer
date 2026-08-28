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
use crate::SeerAisstreamTag;
use crate::{
    forms::{PreferencesForm, PreferencesFormField},
    routes::{AisListRouteTag, PrefsGetRouteTag, PrefsPostRouteTag},
};

fn ais_menu(active: &str) -> Markup {
    let list_url = AisListRouteTag.url();
    let prefs_url = PrefsGetRouteTag.url();
    seer_menu(
        "AISstream",
        html! {
            (menu_item("Messages", &list_url, active == "messages"))
            (menu_item("Preferences", &prefs_url, active == "preferences"))
        },
    )
}

fn prefs_crumbs() -> Markup {
    let list_url = AisListRouteTag.url();
    detail_crumbs("AISstream", &list_url, "Preferences")
}

seer_common::seer_scaffold_page!(
    AisListPage,
    "AISstream — Seer",
    ais_menu("messages"),
    list_crumbs("Messages")
);
seer_common::seer_scaffold_page!(
    AisDetailPage,
    "AISstream — Seer",
    ais_menu("messages"),
    list_crumbs("Messages")
);

#[derive(Generic)]
pub struct AisPreferencesPage {
    pub enabled: bool,
    pub api_key: String,
    pub error: String,
}

impl AisPreferencesPage {
    fn body(&self) -> Markup {
        form(FormOpts {
            attrs: form_hx_post_url::<MainContentKey>(&PrefsPostRouteTag.path())
                .set("hx-swap", "outerHTML"),
            title: "AISstream Preferences",
            subtitle: "Enable the client and set the AIS stream API key",
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: PreferencesForm::render_inputs(
                &FormCtx::form::<PreferencesForm>()
                    .checked(PreferencesFormField::Enabled, self.enabled)
                    .value(PreferencesFormField::ApiKey, self.api_key.as_str()),
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

impl RenderAppPane for AisPreferencesPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(ais_menu("preferences"), prefs_crumbs(), self.body())
    }

    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(prefs_crumbs(), self.body())
    }
}

impl RenderTemplate for AisPreferencesPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "AISstream Preferences — Seer",
            chrome,
            ais_menu("preferences"),
            prefs_crumbs(),
            self.body(),
        )
    }
}

lariv_rs::define_register_items! {
    plugin: SeerAisstreamTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        ListIdx: AisListPageTag => AisListPage,
        DetailIdx: AisDetailPageTag => AisDetailPage,
        PrefsIdx: AisPreferencesPageTag => AisPreferencesPage,
    ]
}
