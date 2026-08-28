use super::{
    handlers,
    keys::{
        RedditSourceUnsetSelectModalKey, RedditSourceUnsetSelectTableKey, RedditWorkerSelectModalKey,
        RedditWorkerSelectTableKey,
    },
};

lariv_rs::define_plugin_routes! {
    plugin: SeerRedditTag;
    routes: [
        get RedditListRouteTag, "/seer-reddit", handlers::list;
        get RedditPostDetailRouteTag, "/seer-reddit/posts/{id}", handlers::post_detail;

        get RedditWorkersListRouteTag, "/seer-reddit/workers", handlers::workers::list;
        get RedditWorkerCreateGetRouteTag, "/seer-reddit/workers/create", handlers::workers::create_get, modal;
        post RedditWorkerCreatePostRouteTag, "/seer-reddit/workers/create", handlers::workers::create_post;
        get RedditWorkerDetailRouteTag, "/seer-reddit/workers/{id}", handlers::workers::detail;
        get RedditWorkerEditGetRouteTag, "/seer-reddit/workers/{id}/edit", handlers::workers::edit_get;
        post RedditWorkerEditPostRouteTag, "/seer-reddit/workers/{id}/edit", bare handlers::workers::edit_post, raw;
        post RedditWorkerDeletePostRouteTag, "/seer-reddit/workers/{id}/delete", bare handlers::workers::delete_post, raw;
        post RedditWorkerPoolStartRouteTag, "/seer-reddit/workers/{id}/worker-pool/start", bare handlers::workers::pool_start, raw;
        post RedditWorkerPoolStopRouteTag, "/seer-reddit/workers/{id}/worker-pool/stop", bare handlers::workers::pool_stop, raw;
        get RedditWorkerSelectRouteTag, "/seer-reddit/workers/select", handlers::workers::select, fk_select(RedditWorkerSelectTableKey, RedditWorkerSelectModalKey);

        get RedditSourcesListRouteTag, "/seer-reddit/sources", handlers::sources::list;
        get RedditSourceCreateGetRouteTag, "/seer-reddit/sources/create", handlers::sources::create_get, modal;
        post RedditSourceCreatePostRouteTag, "/seer-reddit/sources/create", handlers::sources::create_post;
        get RedditSourceDetailRouteTag, "/seer-reddit/sources/{id}", handlers::sources::detail;
        get RedditSourceEditGetRouteTag, "/seer-reddit/sources/{id}/edit", handlers::sources::edit_get;
        post RedditSourceEditPostRouteTag, "/seer-reddit/sources/{id}/edit", bare handlers::sources::edit_post, raw;
        post RedditSourceDeletePostRouteTag, "/seer-reddit/sources/{id}/delete", bare handlers::sources::delete_post, raw;
        get RedditSourceUnsetSelectRouteTag, "/seer-reddit/sources/unset/select", handlers::sources::unset_select, multi_select(RedditSourceUnsetSelectTableKey, RedditSourceUnsetSelectModalKey);
    ]
}
