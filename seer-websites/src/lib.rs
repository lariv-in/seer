#![feature(impl_trait_in_assoc_type)]

pub mod apps;
pub mod crawl;
pub mod filter_llm;
pub mod entities;
pub mod forms;
pub mod handlers;
pub mod intel_kind;
pub mod keys;
pub mod migrations;
pub mod pickers;
pub mod routes;
pub mod rune_env;
pub mod scrape;
pub mod ssrf;
pub mod state;
pub mod templates;
pub mod workers;

use frunk::{HCons, hlist::HList};
use lariv_rs::{
    app::App,
    capability::CapStore,
    db::{DbCap, DbTag},
    hooks::AttachState,
    traits::{
        add::{AddCapability, CapTagAbsent},
        get::GetByCapTag,
    },
};

use state::WebsitesState;

pub struct SeerWebsitesTag;

lariv_rs::define_passthrough_cap!(SeerWebsitesStateCap, SeerWebsitesTag, WebsitesState);

lariv_rs::define_plugin_install! {
    plugin: SeerWebsitesTag;
    steps: [
        apps(apps::Hook),
        rune_env(rune_env::Hook),
        migrations(migrations::Hook),
        templates(templates::Hook),
        http(routes::Hook),
        state(StateHook),
    ]
}

#[derive(Clone, Copy, Default)]
pub struct StateHook;

impl<L, DbIdx, TagProof> AttachState<L, (DbIdx, TagProof)> for StateHook
where
    L: GetByCapTag<DbTag, DbIdx, Value = DbCap>,
    L: HList + CapTagAbsent<SeerWebsitesTag, TagProof>,
{
    type Output = HCons<SeerWebsitesStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        intel_kind::register();
        let state = WebsitesState::new(conn.clone());
        workers::start_all_runner_pools(conn);
        app.add_capability(CapStore::with_items(state))
    }
}

pub use scrape::{crawl_url, enqueue_scrape_url, scrape_url};
pub use ssrf::{normalize_website_url, validate_public_url};
