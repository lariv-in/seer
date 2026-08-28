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
use crate::SeerOpenskyTag;
use crate::{
    forms::{PreferencesForm, PreferencesFormField},
    routes::{OpenskyListRouteTag, PrefsGetRouteTag, PrefsPostRouteTag},
};

fn opensky_menu(active: &str) -> Markup {
    let list_url = OpenskyListRouteTag.url();
    let prefs_url = PrefsGetRouteTag.url();
    seer_menu(
        "OpenSky",
        html! {
            (menu_item("States", &list_url, active == "states"))
            (menu_item("Preferences", &prefs_url, active == "preferences"))
        },
    )
}

fn prefs_crumbs() -> Markup {
    let list_url = OpenskyListRouteTag.url();
    detail_crumbs("OpenSky", &list_url, "Preferences")
}

seer_common::seer_scaffold_page!(
    OpenskyListPage,
    "OpenSky — Seer",
    opensky_menu("states"),
    list_crumbs("States")
);

#[derive(Generic)]
pub struct OpenskyPreferencesPage {
    pub client_id: String,
    pub client_secret: String,
    pub error: String,
}

impl OpenskyPreferencesPage {
    fn body(&self) -> Markup {
        form(FormOpts {
            attrs: form_hx_post_url::<MainContentKey>(&PrefsPostRouteTag.path())
                .set("hx-swap", "outerHTML"),
            title: "OpenSky Preferences",
            subtitle: "OAuth client credentials for the OpenSky Network API",
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: PreferencesForm::render_inputs(
                &FormCtx::form::<PreferencesForm>()
                    .value(PreferencesFormField::ClientId, self.client_id.as_str())
                    .value(
                        PreferencesFormField::ClientSecret,
                        self.client_secret.as_str(),
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

impl RenderAppPane for OpenskyPreferencesPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(opensky_menu("preferences"), prefs_crumbs(), self.body())
    }

    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(prefs_crumbs(), self.body())
    }
}

impl RenderTemplate for OpenskyPreferencesPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "OpenSky Preferences — Seer",
            chrome,
            opensky_menu("preferences"),
            prefs_crumbs(),
            self.body(),
        )
    }
}

lariv_rs::define_register_items! {
    plugin: SeerOpenskyTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        ListIdx: OpenskyListPageTag => OpenskyListPage,
        PrefsIdx: OpenskyPreferencesPageTag => OpenskyPreferencesPage,
    ]
}
