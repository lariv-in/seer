use super::handlers;

lariv_rs::define_plugin_routes! {
    plugin: SeerIntelTag;
    routes: [
        get IntelListRouteTag, "/seer-intel", handlers::list;
        get IntelMapRouteTag, "/seer-intel/map", handlers::map::page;
        get IntelMapDataRouteTag, "/seer-intel/map/data", bare handlers::map::data_ws, raw;
        get PrefsGetRouteTag, "/seer-intel/preferences", handlers::preferences::get;
        post PrefsPostRouteTag, "/seer-intel/preferences", handlers::preferences::post;
        get IntelDetailRouteTag, "/seer-intel/{id}", handlers::detail;
    ]
}
