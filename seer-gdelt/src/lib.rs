#![feature(impl_trait_in_assoc_type)]

pub mod apps;
pub mod bigquery;
pub mod config;
pub mod entities;
pub mod forms;
pub mod handlers;
pub mod keys;
pub mod map_export;
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

use state::GdeltState;

pub struct SeerGdeltTag;

lariv_rs::define_passthrough_cap!(SeerGdeltStateCap, SeerGdeltTag, GdeltState);

lariv_rs::define_plugin_install! {
    plugin: SeerGdeltTag;
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
    L: HList + CapTagAbsent<SeerGdeltTag, TagProof>,
{
    type Output = HCons<SeerGdeltStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        workers::start_all_runner_pools(conn.clone());
        app.add_capability(CapStore::with_items(GdeltState::new(conn)))
    }
}
