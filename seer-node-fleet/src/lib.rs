
//! Node scraper fleet — websocket registry + DispatchCommand.

pub mod apps;
pub mod dispatch;
pub mod handlers;
pub mod keys;
pub mod messages;
pub mod routes;
pub mod templates;

pub struct SeerNodeFleetTag;

lariv_rs::define_plugin_install! {
    plugin: SeerNodeFleetTag;
    steps: [
        apps(apps::Hook),
        templates(templates::Hook),
        http(routes::Hook),
    ]
}

pub use dispatch::{connected_nodes, dispatch_command, ConnectedNode, DispatchCommand};
pub use messages::*;
