use super::{
    handlers,
    keys::{
        WebsiteSourceUnsetSelectModalKey, WebsiteSourceUnsetSelectTableKey,
        WebsiteWorkerSelectModalKey, WebsiteWorkerSelectTableKey,
    },
};

lariv_rs::define_plugin_routes! {
    plugin: SeerWebsitesTag;
    routes: [
        get WebsitesListRouteTag, "/seer-websites", handlers::list;
        get WebsiteDetailRouteTag, "/seer-websites/pages/{id}", handlers::detail;

        get WebsiteWorkersListRouteTag, "/seer-websites/workers", handlers::workers::list;
        get WebsiteWorkerCreateGetRouteTag, "/seer-websites/workers/create", handlers::workers::create_get, modal;
        post WebsiteWorkerCreatePostRouteTag, "/seer-websites/workers/create", handlers::workers::create_post;
        get WebsiteWorkerDetailRouteTag, "/seer-websites/workers/{id}", handlers::workers::detail;
        get WebsiteWorkerEditGetRouteTag, "/seer-websites/workers/{id}/edit", handlers::workers::edit_get;
        post WebsiteWorkerEditPostRouteTag, "/seer-websites/workers/{id}/edit", bare handlers::workers::edit_post, raw;
        post WebsiteWorkerDeletePostRouteTag, "/seer-websites/workers/{id}/delete", bare handlers::workers::delete_post, raw;
        post WebsiteWorkerPoolStartRouteTag, "/seer-websites/workers/{id}/worker-pool/start", bare handlers::workers::pool_start, raw;
        post WebsiteWorkerPoolStopRouteTag, "/seer-websites/workers/{id}/worker-pool/stop", bare handlers::workers::pool_stop, raw;
        get WebsiteWorkerSelectRouteTag, "/seer-websites/workers/select", handlers::workers::select, fk_select(WebsiteWorkerSelectTableKey, WebsiteWorkerSelectModalKey);

        get WebsiteSourcesListRouteTag, "/seer-websites/sources", handlers::sources::list;
        get WebsiteSourceCreateGetRouteTag, "/seer-websites/sources/create", handlers::sources::create_get, modal;
        post WebsiteSourceCreatePostRouteTag, "/seer-websites/sources/create", handlers::sources::create_post;
        get WebsiteSourceDetailRouteTag, "/seer-websites/sources/{id}", handlers::sources::detail;
        get WebsiteSourceEditGetRouteTag, "/seer-websites/sources/{id}/edit", handlers::sources::edit_get;
        post WebsiteSourceEditPostRouteTag, "/seer-websites/sources/{id}/edit", bare handlers::sources::edit_post, raw;
        post WebsiteSourceDeletePostRouteTag, "/seer-websites/sources/{id}/delete", bare handlers::sources::delete_post, raw;
        get WebsiteSourceUnsetSelectRouteTag, "/seer-websites/sources/unset/select", handlers::sources::unset_select, multi_select(WebsiteSourceUnsetSelectTableKey, WebsiteSourceUnsetSelectModalKey);
    ]
}
