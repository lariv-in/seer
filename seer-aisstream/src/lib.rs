#![feature(impl_trait_in_assoc_type)]

pub mod apps;
pub mod client;
pub mod config;
pub mod entities;
pub mod forms;
pub mod handlers;
pub mod keys;
pub mod map;
pub mod migrations;
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

use state::AisstreamState;

pub struct SeerAisstreamTag;

lariv_rs::define_passthrough_cap!(SeerAisstreamStateCap, SeerAisstreamTag, AisstreamState);

lariv_rs::define_plugin_install! {
    plugin: SeerAisstreamTag;
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
    L: HList + CapTagAbsent<SeerAisstreamTag, TagProof>,
{
    type Output = HCons<SeerAisstreamStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        client::start_client(conn.clone());
        app.add_capability(CapStore::with_items(AisstreamState::new(conn)))
    }
}
