use super::handlers;

lariv_rs::define_plugin_routes! {
    plugin: SeerWorkerRegistryTag;
    routes: [
        get WorkersHomeRouteTag, "/seer-workers", handlers::home;
    ]
}
