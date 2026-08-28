use super::{
    handlers,
    keys::{
        TwitterSourceUnsetSelectModalKey, TwitterSourceUnsetSelectTableKey,
        TwitterWorkerSelectModalKey, TwitterWorkerSelectTableKey,
    },
};

lariv_rs::define_plugin_routes! {
    plugin: SeerTwitterTag;
    routes: [
        get TwitterListRouteTag, "/seer-twitter", handlers::list;
        get PrefsGetRouteTag, "/seer-twitter/preferences", handlers::preferences::get;
        post PrefsPostRouteTag, "/seer-twitter/preferences", handlers::preferences::post;
        get TwitterPostDetailRouteTag, "/seer-twitter/posts/{id}", handlers::post_detail;

        get TwitterWorkersListRouteTag, "/seer-twitter/workers", handlers::workers::list;
        get TwitterWorkerCreateGetRouteTag, "/seer-twitter/workers/create", handlers::workers::create_get, modal;
        post TwitterWorkerCreatePostRouteTag, "/seer-twitter/workers/create", handlers::workers::create_post;
        get TwitterWorkerDetailRouteTag, "/seer-twitter/workers/{id}", handlers::workers::detail;
        get TwitterWorkerEditGetRouteTag, "/seer-twitter/workers/{id}/edit", handlers::workers::edit_get;
        post TwitterWorkerEditPostRouteTag, "/seer-twitter/workers/{id}/edit", bare handlers::workers::edit_post, raw;
        post TwitterWorkerDeletePostRouteTag, "/seer-twitter/workers/{id}/delete", bare handlers::workers::delete_post, raw;
        post TwitterWorkerPoolStartRouteTag, "/seer-twitter/workers/{id}/worker-pool/start", bare handlers::workers::pool_start, raw;
        post TwitterWorkerPoolStopRouteTag, "/seer-twitter/workers/{id}/worker-pool/stop", bare handlers::workers::pool_stop, raw;
        get TwitterWorkerSelectRouteTag, "/seer-twitter/workers/select", handlers::workers::select, fk_select(TwitterWorkerSelectTableKey, TwitterWorkerSelectModalKey);

        get TwitterSourcesListRouteTag, "/seer-twitter/sources", handlers::sources::list;
        get TwitterSourceCreateGetRouteTag, "/seer-twitter/sources/create", handlers::sources::create_get, modal;
        post TwitterSourceCreatePostRouteTag, "/seer-twitter/sources/create", handlers::sources::create_post;
        get TwitterSourceDetailRouteTag, "/seer-twitter/sources/{id}", handlers::sources::detail;
        get TwitterSourceEditGetRouteTag, "/seer-twitter/sources/{id}/edit", handlers::sources::edit_get;
        post TwitterSourceEditPostRouteTag, "/seer-twitter/sources/{id}/edit", bare handlers::sources::edit_post, raw;
        post TwitterSourceDeletePostRouteTag, "/seer-twitter/sources/{id}/delete", bare handlers::sources::delete_post, raw;
        get TwitterSourceUnsetSelectRouteTag, "/seer-twitter/sources/unset/select", handlers::sources::unset_select, multi_select(TwitterSourceUnsetSelectTableKey, TwitterSourceUnsetSelectModalKey);
    ]
}
