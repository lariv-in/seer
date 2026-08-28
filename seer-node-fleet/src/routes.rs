use super::handlers;

lariv_rs::define_plugin_routes! {
    plugin: SeerNodeFleetTag;
    routes: [
        get FleetHomeRouteTag, "/fleet", handlers::home;
        get FleetWsRouteTag, "/fleet/websocket", bare handlers::fleet_ws, raw;
    ]
}
