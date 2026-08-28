use super::handlers;

lariv_rs::define_plugin_routes! {
    plugin: SeerAisstreamTag;
    routes: [
        get AisListRouteTag, "/seer-aisstream", handlers::list;
        get AisDetailRouteTag, "/seer-aisstream/{id}", handlers::detail;
        get PrefsGetRouteTag, "/seer-aisstream/preferences", handlers::preferences::get;
        post PrefsPostRouteTag, "/seer-aisstream/preferences", handlers::preferences::post;
    ]
}
