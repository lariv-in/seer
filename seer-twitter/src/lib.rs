#![feature(impl_trait_in_assoc_type)]

//! Seer twitter — Nitter RSS ingest workers.

pub mod apps;
pub mod config;
pub mod entities;
pub mod fetch;
pub mod filter_llm;
pub mod forms;
pub mod handlers;
pub mod intel_kind;
pub mod keys;
pub mod migrations;
pub mod pickers;
pub mod preferences;
pub mod routes;
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

use state::TwitterState;

pub struct SeerTwitterTag;

lariv_rs::define_passthrough_cap!(SeerTwitterStateCap, SeerTwitterTag, TwitterState);

lariv_rs::define_plugin_install! {
    plugin: SeerTwitterTag;
    steps: [
        apps(apps::Hook),
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
    L: HList + CapTagAbsent<SeerTwitterTag, TagProof>,
{
    type Output = HCons<SeerTwitterStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        intel_kind::register();
        workers::start_all_runner_pools(conn.clone());
        app.add_capability(CapStore::with_items(TwitterState::new(conn)))
    }
}
