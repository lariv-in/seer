//! Typed picker / create-modal wiring for Twitter FK / M2M select routes.

use super::keys::{
    TwitterSourceCreateModalKey, TwitterSourceUnsetSelectModalKey, TwitterSourceUnsetSelectTableKey,
    TwitterWorkerCreateModalKey, TwitterWorkerSelectModalKey, TwitterWorkerSelectTableKey,
};
use super::routes::{
    TwitterSourceCreateGetRouteTag, TwitterSourceCreatePostRouteTag, TwitterWorkerCreateGetRouteTag,
    TwitterWorkerCreatePostRouteTag,
};

lariv_rs::impl_create_modal!(
    TwitterWorkerCreateModalKey,
    TwitterWorkerCreateGetRouteTag,
    TwitterWorkerCreatePostRouteTag,
    "seer_twitter.WorkerCreateForm"
);
lariv_rs::impl_create_modal!(
    TwitterSourceCreateModalKey,
    TwitterSourceCreateGetRouteTag,
    TwitterSourceCreatePostRouteTag,
    "seer_twitter.SourceCreateForm"
);
lariv_rs::impl_picker_modal!(TwitterWorkerSelectModalKey, TwitterWorkerSelectTableKey);
lariv_rs::impl_picker_modal!(
    TwitterSourceUnsetSelectModalKey,
    TwitterSourceUnsetSelectTableKey
);
