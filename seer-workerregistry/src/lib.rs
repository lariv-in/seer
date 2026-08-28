#![feature(impl_trait_in_assoc_type)]

//! Worker run-log registry — active workers home + Start/Finish/List helpers.

pub mod apps;
pub mod entities;
pub mod handlers;
pub mod keys;
pub mod migrations;
pub mod routes;
pub mod run_logs;
pub mod state;
pub mod templates;

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

use state::WorkerRegistryState;

pub struct SeerWorkerRegistryTag;

lariv_rs::define_passthrough_cap!(
    SeerWorkerRegistryStateCap,
    SeerWorkerRegistryTag,
    WorkerRegistryState
);

lariv_rs::define_plugin_install! {
    plugin: SeerWorkerRegistryTag;
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
    L: HList + CapTagAbsent<SeerWorkerRegistryTag, TagProof>,
{
    type Output = HCons<SeerWorkerRegistryStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(WorkerRegistryState::new(conn)))
    }
}

pub use run_logs::{
    finish_worker_run_log, latest_worker_run_log, list_worker_run_logs, start_worker_run_log,
};
