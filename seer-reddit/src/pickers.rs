//! Typed picker / create-modal wiring for Reddit FK / M2M select routes.

use super::keys::{
    RedditSourceCreateModalKey, RedditSourceUnsetSelectModalKey, RedditSourceUnsetSelectTableKey,
    RedditWorkerCreateModalKey, RedditWorkerSelectModalKey, RedditWorkerSelectTableKey,
};
use super::routes::{
    RedditSourceCreateGetRouteTag, RedditSourceCreatePostRouteTag, RedditWorkerCreateGetRouteTag,
    RedditWorkerCreatePostRouteTag,
};

lariv_rs::impl_create_modal!(
    RedditWorkerCreateModalKey,
    RedditWorkerCreateGetRouteTag,
    RedditWorkerCreatePostRouteTag,
    "seer_reddit.WorkerCreateForm"
);
lariv_rs::impl_create_modal!(
    RedditSourceCreateModalKey,
    RedditSourceCreateGetRouteTag,
    RedditSourceCreatePostRouteTag,
    "seer_reddit.SourceCreateForm"
);
lariv_rs::impl_picker_modal!(RedditWorkerSelectModalKey, RedditWorkerSelectTableKey);
lariv_rs::impl_picker_modal!(
    RedditSourceUnsetSelectModalKey,
    RedditSourceUnsetSelectTableKey
);
