use frunk::Generic;
use maud::{html, Markup};

use lariv_rs::{
    components::{
        button_clear, button_submit, container_row, data_table_list_refresh, field_text, form,
        form_hx_get_picker_route, form_hx_post_url, modal_keyed, row_attr_select,
        row_attr_select_multi, table_button_filter, ButtonClear, ButtonSubmit, FieldText, FormOpts,
        MainContentKey, ObjectList, ShellChrome, TableButtonFilter, TableColumnHeader, TableRow,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    picker::{RenderPickerSelect, picker_create_button},
    template::{RenderAppPane, RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::{CreateModal, modal_create_post_query},
};
use seer_common::templates::{
    app_scaffold, detail_crumbs, list_crumbs, menu_item, scaffold_main, scaffold_pane, seer_menu,
};

#[allow(unused_imports)]
use crate::SeerWebsitesTag;
use crate::{
    forms::{
        NameFilterForm, NameFilterFormField, SourceForm, SourceFormField, WorkerForm,
        WorkerFormField,
    },
    handlers::{sources::SourceSelectRow, workers::WorkerSelectRow},
    keys::{
        WebsiteSourceCreateModalKey, WebsiteSourceUnsetSelectModalKey,
        WebsiteSourceUnsetSelectTableKey, WebsiteWorkerCreateModalKey, WebsiteWorkerSelectModalKey,
        WebsiteWorkerSelectTableKey,
    },
    routes::{
        WebsiteSourceCreatePostRouteTag, WebsiteSourceEditPostRouteTag,
        WebsiteSourceUnsetSelectRouteTag, WebsiteSourcesListRouteTag, WebsiteWorkerCreatePostRouteTag,
        WebsiteWorkerEditPostRouteTag, WebsiteWorkerSelectRouteTag, WebsiteWorkersListRouteTag,
        WebsitesListRouteTag,
    },
};

fn websites_menu(active: &str) -> Markup {
    let list_url = WebsitesListRouteTag.url();
    let workers_url = WebsiteWorkersListRouteTag.url();
    let sources_url = WebsiteSourcesListRouteTag.url();
    seer_menu(
        "Websites",
        html! {
            (menu_item("Saved pages", &list_url, active == "pages"))
            (menu_item("Workers", &workers_url, active == "workers"))
            (menu_item("Sources", &sources_url, active == "sources"))
        },
    )
}

seer_common::seer_scaffold_page!(
    WebsitesListPage,
    "Websites — Seer",
    websites_menu("pages"),
    list_crumbs("Saved pages")
);
seer_common::seer_scaffold_page!(
    WebsiteDetailPage,
    "Websites — Seer",
    websites_menu("pages"),
    list_crumbs("Saved pages")
);
seer_common::seer_scaffold_page!(
    WebsiteWorkersListPage,
    "Websites — Seer",
    websites_menu("workers"),
    list_crumbs("Workers")
);
seer_common::seer_scaffold_page!(
    WebsiteWorkerDetailPage,
    "Websites — Seer",
    websites_menu("workers"),
    list_crumbs("Workers")
);
seer_common::seer_scaffold_page!(
    WebsiteSourcesListPage,
    "Websites — Seer",
    websites_menu("sources"),
    list_crumbs("Sources")
);
seer_common::seer_scaffold_page!(
    WebsiteSourceDetailPage,
    "Websites — Seer",
    websites_menu("sources"),
    list_crumbs("Sources")
);

fn worker_form_body(
    name: &str,
    duration: &str,
    sources: &[lariv_rs::components::ManyToManyItem],
    error: &str,
    post_path: &str,
) -> Markup {
    let ctx = FormCtx::form::<WorkerForm>()
        .value(WorkerFormField::Name, name)
        .value(WorkerFormField::Duration, duration)
        .m2m(WorkerFormField::SourceIds, sources);
    form(FormOpts {
        attrs: form_hx_post_url::<MainContentKey>(post_path).set("hx-swap", "outerHTML"),
        title: "Worker",
        form_error: Some(error).filter(|e| !e.is_empty()),
        inputs: WorkerForm::render_inputs(&ctx),
        actions: html! {
            (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
        },
        ..Default::default()
    })
}

#[derive(Generic)]
pub struct WebsiteWorkerCreatePage {
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
    pub name: String,
    pub duration: String,
    pub sources: Vec<lariv_rs::components::ManyToManyItem>,
    pub error: String,
}

impl RenderTemplate for WebsiteWorkerCreatePage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            WebsiteWorkerCreateModalKey::FORM_NAME
        } else {
            self.form_name.as_str()
        };
        let ctx = FormCtx::form::<WorkerForm>()
            .value(WorkerFormField::Name, &self.name)
            .value(WorkerFormField::Duration, &self.duration)
            .m2m(WorkerFormField::SourceIds, &self.sources);
        modal_keyed::<WebsiteWorkerCreateModalKey>(
            "",
            form(FormOpts {
                title: "Create Worker",
                attrs: form_hx_post_url::<WebsiteWorkerCreateModalKey>(&modal_create_post_query(
                    WebsiteWorkerCreatePostRouteTag,
                    form_name,
                    &self.refresh_table,
                    &self.target_input,
                )),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                inputs: WorkerForm::render_inputs(&ctx),
                actions: html! {
                    (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
                },
                ..Default::default()
            }),
        )
    }
}

#[derive(Generic)]
pub struct WebsiteWorkerEditPage {
    pub id: i64,
    pub name: String,
    pub duration: String,
    pub sources: Vec<lariv_rs::components::ManyToManyItem>,
    pub error: String,
}

impl WebsiteWorkerEditPage {
    fn body(&self) -> Markup {
        worker_form_body(
            &self.name,
            &self.duration,
            &self.sources,
            &self.error,
            &WebsiteWorkerEditPostRouteTag::new(self.id).path(),
        )
    }
}

impl RenderAppPane for WebsiteWorkerEditPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            websites_menu("workers"),
            detail_crumbs("Workers", &WebsiteWorkersListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            detail_crumbs("Workers", &WebsiteWorkersListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
}

impl RenderTemplate for WebsiteWorkerEditPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Edit Worker — Seer",
            chrome,
            websites_menu("workers"),
            detail_crumbs("Workers", &WebsiteWorkersListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
}

fn source_form_body(
    runner_id: Option<i64>,
    runner_display: &str,
    url: &str,
    depth: i64,
    filter: &str,
    is_filter_whitelist: bool,
    error: &str,
    post_path: &str,
) -> Markup {
    let runner_s = runner_id
        .filter(|&i| i > 0)
        .map(|i| i.to_string())
        .unwrap_or_default();
    let depth_s = depth.to_string();
    let mut ctx = FormCtx::form::<SourceForm>()
        .value(SourceFormField::WebsiteRunnerId, runner_s.as_str())
        .display(SourceFormField::WebsiteRunnerId, runner_display)
        .value(SourceFormField::Url, url)
        .value(SourceFormField::Depth, depth_s.as_str())
        .value(SourceFormField::Filter, filter);
    if is_filter_whitelist {
        ctx = ctx.value(SourceFormField::IsFilterWhitelist, "true");
    }
    form(FormOpts {
        attrs: form_hx_post_url::<MainContentKey>(post_path).set("hx-swap", "outerHTML"),
        title: "Source",
        form_error: Some(error).filter(|e| !e.is_empty()),
        inputs: SourceForm::render_inputs(&ctx),
        actions: html! {
            (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
        },
        ..Default::default()
    })
}

#[derive(Generic)]
pub struct WebsiteSourceCreatePage {
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
    pub website_runner_id: Option<i64>,
    pub runner_display: String,
    pub url: String,
    pub depth: i64,
    pub filter: String,
    pub is_filter_whitelist: bool,
    pub error: String,
}

impl RenderTemplate for WebsiteSourceCreatePage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            WebsiteSourceCreateModalKey::FORM_NAME
        } else {
            self.form_name.as_str()
        };
        let runner_s = self
            .website_runner_id
            .filter(|&i| i > 0)
            .map(|i| i.to_string())
            .unwrap_or_default();
        let depth_s = self.depth.to_string();
        let mut ctx = FormCtx::form::<SourceForm>()
            .value(SourceFormField::WebsiteRunnerId, runner_s.as_str())
            .display(SourceFormField::WebsiteRunnerId, &self.runner_display)
            .value(SourceFormField::Url, &self.url)
            .value(SourceFormField::Depth, depth_s.as_str())
            .value(SourceFormField::Filter, &self.filter);
        if self.is_filter_whitelist {
            ctx = ctx.value(SourceFormField::IsFilterWhitelist, "true");
        }
        modal_keyed::<WebsiteSourceCreateModalKey>(
            "",
            form(FormOpts {
                title: "Create Source",
                attrs: form_hx_post_url::<WebsiteSourceCreateModalKey>(&modal_create_post_query(
                    WebsiteSourceCreatePostRouteTag,
                    form_name,
                    &self.refresh_table,
                    &self.target_input,
                )),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                inputs: SourceForm::render_inputs(&ctx),
                actions: html! {
                    (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
                },
                ..Default::default()
            }),
        )
    }
}

#[derive(Generic)]
pub struct WebsiteSourceEditPage {
    pub id: i64,
    pub website_runner_id: Option<i64>,
    pub runner_display: String,
    pub url: String,
    pub depth: i64,
    pub filter: String,
    pub is_filter_whitelist: bool,
    pub error: String,
}

impl WebsiteSourceEditPage {
    fn body(&self) -> Markup {
        source_form_body(
            self.website_runner_id,
            &self.runner_display,
            &self.url,
            self.depth,
            &self.filter,
            self.is_filter_whitelist,
            &self.error,
            &WebsiteSourceEditPostRouteTag::new(self.id).path(),
        )
    }
}

impl RenderAppPane for WebsiteSourceEditPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            websites_menu("sources"),
            detail_crumbs("Sources", &WebsiteSourcesListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            detail_crumbs("Sources", &WebsiteSourcesListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
}

impl RenderTemplate for WebsiteSourceEditPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Edit Source — Seer",
            chrome,
            websites_menu("sources"),
            detail_crumbs("Sources", &WebsiteSourcesListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
}

#[derive(Generic)]
pub struct WebsiteWorkerSelectPage {
    pub runners: ObjectList<WorkerSelectRow>,
    pub filter_name: String,
    pub target_input: String,
    pub path_and_query: String,
}

impl RenderPickerSelect<WebsiteWorkerSelectTableKey, WebsiteWorkerSelectModalKey>
    for WebsiteWorkerSelectPage
{
    fn render_table(&self) -> Markup {
        let target = if self.target_input.is_empty() {
            "website_runner_id"
        } else {
            self.target_input.as_str()
        };
        let headers = [TableColumnHeader {
            key: "Name",
            label: "Name",
            sort_url: None,
            push_url: false,
        }];
        let rows: Vec<TableRow> = self
            .runners
            .items
            .iter()
            .map(|r| TableRow {
                attrs: row_attr_select(target, &r.id.to_string(), &r.name),
                cells: vec![field_text(FieldText {
                    value: &r.name,
                    classes: "",
                })],
            })
            .collect();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        WebsiteWorkerSelectTableKey,
                        WebsiteWorkerSelectModalKey,
                        WebsiteWorkerSelectRouteTag,
                    >(WebsiteWorkerSelectRouteTag),
                    inputs: html! {
                        (NameFilterForm::render_inputs(
                            &FormCtx::form::<NameFilterForm>()
                                .value(NameFilterFormField::Name, &self.filter_name),
                        ))
                        input type="hidden" name="target_input" value=(self.target_input) {}
                    },
                    actions: html! {
                        (container_row("flex gap-2", html! {
                            (button_submit(ButtonSubmit { label: "Apply", ..Default::default() }))
                            (button_clear(ButtonClear { label: "Clear", ..Default::default() }))
                        }))
                    },
                    ..Default::default()
                }),
                ..Default::default()
            }))
            (picker_create_button::<WebsiteWorkerCreateModalKey>(
                &self.target_input,
                Some("plus"),
                "btn-square btn-outline btn-sm",
            ))
        };
        data_table_list_refresh::<WebsiteWorkerSelectTableKey>(
            "Select Worker",
            actions,
            &headers,
            &rows,
            html! {},
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for WebsiteWorkerSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

#[derive(Generic)]
pub struct WebsiteSourceUnsetSelectPage {
    pub sources: ObjectList<SourceSelectRow>,
    pub filter_name: String,
    pub target_input: String,
    pub path_and_query: String,
}

impl RenderPickerSelect<WebsiteSourceUnsetSelectTableKey, WebsiteSourceUnsetSelectModalKey>
    for WebsiteSourceUnsetSelectPage
{
    fn render_table(&self) -> Markup {
        let target = if self.target_input.is_empty() {
            "source_ids"
        } else {
            self.target_input.as_str()
        };
        let headers = [TableColumnHeader {
            key: "Source",
            label: "Source",
            sort_url: None,
            push_url: false,
        }];
        let rows: Vec<TableRow> = self
            .sources
            .items
            .iter()
            .map(|s| TableRow {
                attrs: row_attr_select_multi(target, &s.id.to_string(), &s.label),
                cells: vec![field_text(FieldText {
                    value: &s.label,
                    classes: "",
                })],
            })
            .collect();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        WebsiteSourceUnsetSelectTableKey,
                        WebsiteSourceUnsetSelectModalKey,
                        WebsiteSourceUnsetSelectRouteTag,
                    >(WebsiteSourceUnsetSelectRouteTag),
                    inputs: html! {
                        (NameFilterForm::render_inputs(
                            &FormCtx::form::<NameFilterForm>()
                                .value(NameFilterFormField::Name, &self.filter_name),
                        ))
                        input type="hidden" name="target_input" value=(self.target_input) {}
                    },
                    actions: html! {
                        (container_row("flex gap-2", html! {
                            (button_submit(ButtonSubmit { label: "Apply", ..Default::default() }))
                            (button_clear(ButtonClear { label: "Clear", ..Default::default() }))
                        }))
                    },
                    ..Default::default()
                }),
                ..Default::default()
            }))
            (picker_create_button::<WebsiteSourceCreateModalKey>(
                &self.target_input,
                Some("plus"),
                "btn-square btn-outline btn-sm",
            ))
        };
        data_table_list_refresh::<WebsiteSourceUnsetSelectTableKey>(
            "Select Sources",
            actions,
            &headers,
            &rows,
            html! {},
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for WebsiteSourceUnsetSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

lariv_rs::define_register_items! {
    plugin: SeerWebsitesTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        ListIdx: WebsitesListPageTag => WebsitesListPage,
        DetailIdx: WebsiteDetailPageTag => WebsiteDetailPage,
        WorkersListIdx: WebsiteWorkersListPageTag => WebsiteWorkersListPage,
        WorkerDetailIdx: WebsiteWorkerDetailPageTag => WebsiteWorkerDetailPage,
        WorkerCreateIdx: WebsiteWorkerCreatePageTag => WebsiteWorkerCreatePage,
        WorkerEditIdx: WebsiteWorkerEditPageTag => WebsiteWorkerEditPage,
        SourcesListIdx: WebsiteSourcesListPageTag => WebsiteSourcesListPage,
        SourceDetailIdx: WebsiteSourceDetailPageTag => WebsiteSourceDetailPage,
        SourceCreateIdx: WebsiteSourceCreatePageTag => WebsiteSourceCreatePage,
        SourceEditIdx: WebsiteSourceEditPageTag => WebsiteSourceEditPage,
        WorkerSelectIdx: WebsiteWorkerSelectPageTag => WebsiteWorkerSelectPage,
        SourceUnsetSelectIdx: WebsiteSourceUnsetSelectPageTag => WebsiteSourceUnsetSelectPage,
    ]
}
