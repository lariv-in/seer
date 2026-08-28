use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonSubmit, FieldText, FormOpts, MainContentKey, ObjectList, ShellChrome,
        TableButtonFilter, TableColumnHeader, TableRow, button_clear, button_submit, container_row,
        data_table_list_refresh, field_text, form, form_hx_get_picker_route, form_hx_post_url,
        modal_keyed, row_attr_select, row_attr_select_multi, table_button_filter,
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
use crate::SeerRedditTag;
use crate::{
    forms::{
        NameFilterForm, NameFilterFormField, SourceForm, SourceFormField, WorkerForm,
        WorkerFormField,
    },
    handlers::{sources::SourceSelectRow, workers::WorkerSelectRow},
    keys::{
        RedditSourceCreateModalKey, RedditSourceUnsetSelectModalKey,
        RedditSourceUnsetSelectTableKey, RedditWorkerCreateModalKey, RedditWorkerSelectModalKey,
        RedditWorkerSelectTableKey,
    },
    routes::{
        RedditListRouteTag, RedditSourceCreatePostRouteTag, RedditSourceEditPostRouteTag,
        RedditSourceUnsetSelectRouteTag, RedditSourcesListRouteTag, RedditWorkerCreatePostRouteTag,
        RedditWorkerEditPostRouteTag, RedditWorkerSelectRouteTag, RedditWorkersListRouteTag,
    },
};

fn reddit_menu(active: &str) -> Markup {
    let list_url = RedditListRouteTag.url();
    let workers_url = RedditWorkersListRouteTag.url();
    let sources_url = RedditSourcesListRouteTag.url();
    seer_menu(
        "Reddit",
        html! {
            (menu_item("Posts", &list_url, active == "posts"))
            (menu_item("Workers", &workers_url, active == "workers"))
            (menu_item("Sources", &sources_url, active == "sources"))
        },
    )
}

seer_common::seer_scaffold_page!(
    RedditListPage,
    "Reddit — Seer",
    reddit_menu("posts"),
    list_crumbs("Posts")
);
seer_common::seer_scaffold_page!(
    RedditPostDetailPage,
    "Reddit — Seer",
    reddit_menu("posts"),
    list_crumbs("Posts")
);
seer_common::seer_scaffold_page!(
    RedditWorkersListPage,
    "Reddit — Seer",
    reddit_menu("workers"),
    list_crumbs("Workers")
);
seer_common::seer_scaffold_page!(
    RedditWorkerDetailPage,
    "Reddit — Seer",
    reddit_menu("workers"),
    list_crumbs("Workers")
);
seer_common::seer_scaffold_page!(
    RedditSourcesListPage,
    "Reddit — Seer",
    reddit_menu("sources"),
    list_crumbs("Sources")
);
seer_common::seer_scaffold_page!(
    RedditSourceDetailPage,
    "Reddit — Seer",
    reddit_menu("sources"),
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
pub struct RedditWorkerCreatePage {
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
    pub name: String,
    pub duration: String,
    pub sources: Vec<lariv_rs::components::ManyToManyItem>,
    pub error: String,
}

impl RenderTemplate for RedditWorkerCreatePage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            RedditWorkerCreateModalKey::FORM_NAME
        } else {
            self.form_name.as_str()
        };
        let ctx = FormCtx::form::<WorkerForm>()
            .value(WorkerFormField::Name, &self.name)
            .value(WorkerFormField::Duration, &self.duration)
            .m2m(WorkerFormField::SourceIds, &self.sources);
        modal_keyed::<RedditWorkerCreateModalKey>(
            "",
            form(FormOpts {
                title: "Create Worker",
                attrs: form_hx_post_url::<RedditWorkerCreateModalKey>(&modal_create_post_query(
                    RedditWorkerCreatePostRouteTag,
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
pub struct RedditWorkerEditPage {
    pub id: i64,
    pub name: String,
    pub duration: String,
    pub sources: Vec<lariv_rs::components::ManyToManyItem>,
    pub error: String,
}

impl RedditWorkerEditPage {
    fn body(&self) -> Markup {
        worker_form_body(
            &self.name,
            &self.duration,
            &self.sources,
            &self.error,
            &RedditWorkerEditPostRouteTag::new(self.id).path(),
        )
    }
}

impl RenderAppPane for RedditWorkerEditPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            reddit_menu("workers"),
            detail_crumbs("Workers", &RedditWorkersListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            detail_crumbs("Workers", &RedditWorkersListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
}

impl RenderTemplate for RedditWorkerEditPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Edit Worker — Seer",
            chrome,
            reddit_menu("workers"),
            detail_crumbs("Workers", &RedditWorkersListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
}

fn source_form_body(
    runner_id: Option<i64>,
    runner_display: &str,
    subreddits: &[String],
    search_query: &str,
    is_filter_whitelist: bool,
    filter: &str,
    max_fresh_posts: i64,
    load_websites: bool,
    error: &str,
    post_path: &str,
) -> Markup {
    let runner_s = runner_id
        .filter(|&i| i > 0)
        .map(|i| i.to_string())
        .unwrap_or_default();
    let max_s = max_fresh_posts.to_string();
    let mut ctx = FormCtx::form::<SourceForm>()
        .value(SourceFormField::RedditRunnerId, runner_s.as_str())
        .display(SourceFormField::RedditRunnerId, runner_display)
        .list(SourceFormField::Subreddits, subreddits)
        .value(SourceFormField::SearchQuery, search_query)
        .value(SourceFormField::Filter, filter)
        .value(SourceFormField::MaxFreshPosts, max_s.as_str());
    if is_filter_whitelist {
        ctx = ctx.value(SourceFormField::IsFilterWhitelist, "true");
    }
    if load_websites {
        ctx = ctx.value(SourceFormField::LoadWebsites, "true");
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
pub struct RedditSourceCreatePage {
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
    pub reddit_runner_id: Option<i64>,
    pub runner_display: String,
    pub subreddits: Vec<String>,
    pub search_query: String,
    pub is_filter_whitelist: bool,
    pub filter: String,
    pub max_fresh_posts: i64,
    pub load_websites: bool,
    pub error: String,
}

impl RenderTemplate for RedditSourceCreatePage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            RedditSourceCreateModalKey::FORM_NAME
        } else {
            self.form_name.as_str()
        };
        let runner_s = self
            .reddit_runner_id
            .filter(|&i| i > 0)
            .map(|i| i.to_string())
            .unwrap_or_default();
        let max_s = self.max_fresh_posts.to_string();
        let mut ctx = FormCtx::form::<SourceForm>()
            .value(SourceFormField::RedditRunnerId, runner_s.as_str())
            .display(SourceFormField::RedditRunnerId, &self.runner_display)
            .list(SourceFormField::Subreddits, &self.subreddits)
            .value(SourceFormField::SearchQuery, &self.search_query)
            .value(SourceFormField::Filter, &self.filter)
            .value(SourceFormField::MaxFreshPosts, max_s.as_str());
        if self.is_filter_whitelist {
            ctx = ctx.value(SourceFormField::IsFilterWhitelist, "true");
        }
        if self.load_websites {
            ctx = ctx.value(SourceFormField::LoadWebsites, "true");
        }
        modal_keyed::<RedditSourceCreateModalKey>(
            "",
            form(FormOpts {
                title: "Create Source",
                attrs: form_hx_post_url::<RedditSourceCreateModalKey>(&modal_create_post_query(
                    RedditSourceCreatePostRouteTag,
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
pub struct RedditSourceEditPage {
    pub id: i64,
    pub reddit_runner_id: Option<i64>,
    pub runner_display: String,
    pub subreddits: Vec<String>,
    pub search_query: String,
    pub is_filter_whitelist: bool,
    pub filter: String,
    pub max_fresh_posts: i64,
    pub load_websites: bool,
    pub error: String,
}

impl RedditSourceEditPage {
    fn body(&self) -> Markup {
        source_form_body(
            self.reddit_runner_id,
            &self.runner_display,
            &self.subreddits,
            &self.search_query,
            self.is_filter_whitelist,
            &self.filter,
            self.max_fresh_posts,
            self.load_websites,
            &self.error,
            &RedditSourceEditPostRouteTag::new(self.id).path(),
        )
    }
}

impl RenderAppPane for RedditSourceEditPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            reddit_menu("sources"),
            detail_crumbs("Sources", &RedditSourcesListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            detail_crumbs("Sources", &RedditSourcesListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
}

impl RenderTemplate for RedditSourceEditPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Edit Source — Seer",
            chrome,
            reddit_menu("sources"),
            detail_crumbs("Sources", &RedditSourcesListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
}

#[derive(Generic)]
pub struct RedditWorkerSelectPage {
    pub runners: ObjectList<WorkerSelectRow>,
    pub filter_name: String,
    pub target_input: String,
    pub path_and_query: String,
}

impl RenderPickerSelect<RedditWorkerSelectTableKey, RedditWorkerSelectModalKey>
    for RedditWorkerSelectPage
{
    fn render_table(&self) -> Markup {
        let target = if self.target_input.is_empty() {
            "reddit_runner_id"
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
                        RedditWorkerSelectTableKey,
                        RedditWorkerSelectModalKey,
                        RedditWorkerSelectRouteTag,
                    >(RedditWorkerSelectRouteTag),
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
            (picker_create_button::<RedditWorkerCreateModalKey>(
                &self.target_input,
                Some("plus"),
                "btn-square btn-outline btn-sm",
            ))
        };
        data_table_list_refresh::<RedditWorkerSelectTableKey>(
            "Select Worker",
            actions,
            &headers,
            &rows,
            html! {},
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for RedditWorkerSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

#[derive(Generic)]
pub struct RedditSourceUnsetSelectPage {
    pub sources: ObjectList<SourceSelectRow>,
    pub filter_name: String,
    pub target_input: String,
    pub path_and_query: String,
}

impl RenderPickerSelect<RedditSourceUnsetSelectTableKey, RedditSourceUnsetSelectModalKey>
    for RedditSourceUnsetSelectPage
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
                        RedditSourceUnsetSelectTableKey,
                        RedditSourceUnsetSelectModalKey,
                        RedditSourceUnsetSelectRouteTag,
                    >(RedditSourceUnsetSelectRouteTag),
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
            (picker_create_button::<RedditSourceCreateModalKey>(
                &self.target_input,
                Some("plus"),
                "btn-square btn-outline btn-sm",
            ))
        };
        data_table_list_refresh::<RedditSourceUnsetSelectTableKey>(
            "Select Sources",
            actions,
            &headers,
            &rows,
            html! {},
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for RedditSourceUnsetSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

lariv_rs::define_register_items! {
    plugin: SeerRedditTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        ListIdx: RedditListPageTag => RedditListPage,
        DetailIdx: RedditPostDetailPageTag => RedditPostDetailPage,
        WorkersListIdx: RedditWorkersListPageTag => RedditWorkersListPage,
        WorkerDetailIdx: RedditWorkerDetailPageTag => RedditWorkerDetailPage,
        WorkerCreateIdx: RedditWorkerCreatePageTag => RedditWorkerCreatePage,
        WorkerEditIdx: RedditWorkerEditPageTag => RedditWorkerEditPage,
        SourcesListIdx: RedditSourcesListPageTag => RedditSourcesListPage,
        SourceDetailIdx: RedditSourceDetailPageTag => RedditSourceDetailPage,
        SourceCreateIdx: RedditSourceCreatePageTag => RedditSourceCreatePage,
        SourceEditIdx: RedditSourceEditPageTag => RedditSourceEditPage,
        WorkerSelectIdx: RedditWorkerSelectPageTag => RedditWorkerSelectPage,
        SourceUnsetSelectIdx: RedditSourceUnsetSelectPageTag => RedditSourceUnsetSelectPage,
    ]
}
