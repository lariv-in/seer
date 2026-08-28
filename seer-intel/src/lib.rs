#![feature(impl_trait_in_assoc_type)]

//! Seer intel — entities, ingest worker, embeddings, vector search.

pub mod apps;
pub mod config;
pub mod embed;
pub mod entities;
pub mod filter;
pub mod forms;
pub mod generate;
pub mod geocode;
pub mod handlers;
pub mod ingest;
pub mod keys;
pub mod kind;
pub mod map;
pub mod migrations;
pub mod preferences;
pub mod routes;
pub mod search;
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

use state::IntelState;

pub struct SeerIntelTag;

lariv_rs::define_passthrough_cap!(SeerIntelStateCap, SeerIntelTag, IntelState);

lariv_rs::define_plugin_install! {
    plugin: SeerIntelTag;
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
    L: HList + CapTagAbsent<SeerIntelTag, TagProof>,
{
    type Output = HCons<SeerIntelStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        let state = IntelState::new(conn);
        state.spawn_ingest_worker();
        app.add_capability(CapStore::with_items(state))
    }
}

pub use ingest::{enqueue_intel, IngestRequest};
pub use kind::{register_intel_kind, DynIntelKind, IntelKind, IntelKindLoader};
pub use search::search_intel_by_similarity;
