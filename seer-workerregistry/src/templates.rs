use maud::{Markup, html};

use lariv_rs::{
    http::ProvideRequestCaps,
    template::{TemplateCapability, TemplateOf, TemplateRegistrar},
};
use seer_common::templates::{list_crumbs, menu_item, seer_menu};

#[allow(unused_imports)]
use crate::SeerWorkerRegistryTag;
use crate::routes::WorkersHomeRouteTag;

fn workers_menu(active: &str) -> Markup {
    let home_url = WorkersHomeRouteTag.url();
    seer_menu(
        "Worker Registry",
        html! {
            (menu_item("Workers", &home_url, active == "workers"))
        },
    )
}

seer_common::seer_scaffold_page!(
    WorkersHomePage,
    "Worker Registry — Seer",
    workers_menu("workers"),
    list_crumbs("Workers")
);

lariv_rs::define_register_items! {
    plugin: SeerWorkerRegistryTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        HomeIdx: WorkersHomePageTag => WorkersHomePage,
    ]
}
