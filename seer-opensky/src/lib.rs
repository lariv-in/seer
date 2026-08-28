#![feature(impl_trait_in_assoc_type)]

pub mod apps;
pub mod client;
pub mod config;
pub mod entities;
pub mod forms;
pub mod handlers;
pub mod keys;
pub mod migrations;
pub mod poller;
pub mod preferences;
pub mod routes;
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

use state::OpenskyState;

pub struct SeerOpenskyTag;

lariv_rs::define_passthrough_cap!(SeerOpenskyStateCap, SeerOpenskyTag, OpenskyState);

lariv_rs::define_plugin_install! {
    plugin: SeerOpenskyTag;
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
    L: HList + CapTagAbsent<SeerOpenskyTag, TagProof>,
{
    type Output = HCons<SeerOpenskyStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        poller::start_poller(conn.clone());
        app.add_capability(CapStore::with_items(OpenskyState::new(conn)))
    }
}
