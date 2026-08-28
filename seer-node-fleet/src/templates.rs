use maud::{Markup, html};

use lariv_rs::{
    http::ProvideRequestCaps,
    template::{TemplateCapability, TemplateOf, TemplateRegistrar},
};
use seer_common::templates::{list_crumbs, menu_item, seer_menu};

#[allow(unused_imports)]
use crate::SeerNodeFleetTag;
use crate::routes::FleetHomeRouteTag;

fn fleet_menu(active: &str) -> Markup {
    let home_url = FleetHomeRouteTag.url();
    seer_menu(
        "Node fleet",
        html! {
            (menu_item("Connected scrapers", &home_url, active == "scrapers"))
        },
    )
}

seer_common::seer_scaffold_page!(
    FleetHomePage,
    "Node fleet — Seer",
    fleet_menu("scrapers"),
    list_crumbs("Connected scrapers")
);

lariv_rs::define_register_items! {
    plugin: SeerNodeFleetTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [ HomeIdx: FleetHomePageTag => FleetHomePage ]
}
