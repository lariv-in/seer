use super::{
    handlers,
    keys::{
        GdeltSourceUnsetSelectModalKey, GdeltSourceUnsetSelectTableKey, GdeltWorkerSelectModalKey,
        GdeltWorkerSelectTableKey,
    },
};

lariv_rs::define_plugin_routes! {
    plugin: SeerGdeltTag;
    routes: [
        get GdeltListRouteTag, "/seer-gdelt", handlers::list;
        get PrefsGetRouteTag, "/seer-gdelt/preferences", handlers::preferences::get;
        post PrefsPostRouteTag, "/seer-gdelt/preferences", handlers::preferences::post;

        get GdeltWorkersListRouteTag, "/seer-gdelt/workers", handlers::workers::list;
        get GdeltWorkerCreateGetRouteTag, "/seer-gdelt/workers/create", handlers::workers::create_get, modal;
        post GdeltWorkerCreatePostRouteTag, "/seer-gdelt/workers/create", handlers::workers::create_post;
        get GdeltWorkerDetailRouteTag, "/seer-gdelt/workers/{id}", handlers::workers::detail;
        get GdeltWorkerEditGetRouteTag, "/seer-gdelt/workers/{id}/edit", handlers::workers::edit_get;
        post GdeltWorkerEditPostRouteTag, "/seer-gdelt/workers/{id}/edit", bare handlers::workers::edit_post, raw;
        post GdeltWorkerDeletePostRouteTag, "/seer-gdelt/workers/{id}/delete", bare handlers::workers::delete_post, raw;
        post GdeltWorkerPoolStartRouteTag, "/seer-gdelt/workers/{id}/worker-pool/start", bare handlers::workers::pool_start, raw;
        post GdeltWorkerPoolStopRouteTag, "/seer-gdelt/workers/{id}/worker-pool/stop", bare handlers::workers::pool_stop, raw;
        get GdeltWorkerSelectRouteTag, "/seer-gdelt/workers/select", handlers::workers::select, fk_select(GdeltWorkerSelectTableKey, GdeltWorkerSelectModalKey);

        get GdeltSourcesListRouteTag, "/seer-gdelt/sources", handlers::sources::list;
        get GdeltSourceCreateGetRouteTag, "/seer-gdelt/sources/create", handlers::sources::create_get, modal;
        post GdeltSourceCreatePostRouteTag, "/seer-gdelt/sources/create", handlers::sources::create_post;
        get GdeltSourceDetailRouteTag, "/seer-gdelt/sources/{id}", handlers::sources::detail;
        get GdeltSourceEditGetRouteTag, "/seer-gdelt/sources/{id}/edit", handlers::sources::edit_get;
        post GdeltSourceEditPostRouteTag, "/seer-gdelt/sources/{id}/edit", bare handlers::sources::edit_post, raw;
        post GdeltSourceDeletePostRouteTag, "/seer-gdelt/sources/{id}/delete", bare handlers::sources::delete_post, raw;
        get GdeltSourceUnsetSelectRouteTag, "/seer-gdelt/sources/unset/select", handlers::sources::unset_select, multi_select(GdeltSourceUnsetSelectTableKey, GdeltSourceUnsetSelectModalKey);
    ]
}
