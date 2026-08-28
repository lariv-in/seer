use super::handlers;

lariv_rs::define_plugin_routes! {
    plugin: SeerOpenskyTag;
    routes: [
        get OpenskyListRouteTag, "/seer-opensky", handlers::list;
        get PrefsGetRouteTag, "/seer-opensky/preferences", handlers::preferences::get;
        post PrefsPostRouteTag, "/seer-opensky/preferences", handlers::preferences::post;
    ]
}
