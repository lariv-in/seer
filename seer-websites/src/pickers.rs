//! Typed picker / create-modal wiring for Websites FK / M2M select routes.

use super::keys::{
    WebsiteSourceCreateModalKey, WebsiteSourceUnsetSelectModalKey, WebsiteSourceUnsetSelectTableKey,
    WebsiteWorkerCreateModalKey, WebsiteWorkerSelectModalKey, WebsiteWorkerSelectTableKey,
};
use super::routes::{
    WebsiteSourceCreateGetRouteTag, WebsiteSourceCreatePostRouteTag, WebsiteWorkerCreateGetRouteTag,
    WebsiteWorkerCreatePostRouteTag,
};

lariv_rs::impl_create_modal!(
    WebsiteWorkerCreateModalKey,
    WebsiteWorkerCreateGetRouteTag,
    WebsiteWorkerCreatePostRouteTag,
    "seer_websites.WorkerCreateForm"
);
lariv_rs::impl_create_modal!(
    WebsiteSourceCreateModalKey,
    WebsiteSourceCreateGetRouteTag,
    WebsiteSourceCreatePostRouteTag,
    "seer_websites.SourceCreateForm"
);
lariv_rs::impl_picker_modal!(WebsiteWorkerSelectModalKey, WebsiteWorkerSelectTableKey);
lariv_rs::impl_picker_modal!(
    WebsiteSourceUnsetSelectModalKey,
    WebsiteSourceUnsetSelectTableKey
);
