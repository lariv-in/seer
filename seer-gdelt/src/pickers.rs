//! Typed picker / create-modal wiring for GDELT FK / M2M select routes.

use super::keys::{
    GdeltSourceCreateModalKey, GdeltSourceUnsetSelectModalKey, GdeltSourceUnsetSelectTableKey,
    GdeltWorkerCreateModalKey, GdeltWorkerSelectModalKey, GdeltWorkerSelectTableKey,
};
use super::routes::{
    GdeltSourceCreateGetRouteTag, GdeltSourceCreatePostRouteTag, GdeltWorkerCreateGetRouteTag,
    GdeltWorkerCreatePostRouteTag,
};

lariv_rs::impl_create_modal!(
    GdeltWorkerCreateModalKey,
    GdeltWorkerCreateGetRouteTag,
    GdeltWorkerCreatePostRouteTag,
    "seer_gdelt.WorkerCreateForm"
);
lariv_rs::impl_create_modal!(
    GdeltSourceCreateModalKey,
    GdeltSourceCreateGetRouteTag,
    GdeltSourceCreatePostRouteTag,
    "seer_gdelt.SourceCreateForm"
);
lariv_rs::impl_picker_modal!(GdeltWorkerSelectModalKey, GdeltWorkerSelectTableKey);
lariv_rs::impl_picker_modal!(
    GdeltSourceUnsetSelectModalKey,
    GdeltSourceUnsetSelectTableKey
);
