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
use crate::SeerTwitterTag;
use crate::{
    forms::{
        NameFilterForm, NameFilterFormField, PreferencesForm, PreferencesFormField, SourceForm,
        SourceFormField, WorkerForm, WorkerFormField,
    },
    handlers::{sources::SourceSelectRow, workers::WorkerSelectRow},
    keys::{
        TwitterSourceCreateModalKey, TwitterSourceUnsetSelectModalKey,
        TwitterSourceUnsetSelectTableKey, TwitterWorkerCreateModalKey, TwitterWorkerSelectModalKey,
        TwitterWorkerSelectTableKey,
    },
    routes::{
        PrefsGetRouteTag, PrefsPostRouteTag, TwitterListRouteTag, TwitterSourceCreatePostRouteTag,
        TwitterSourceEditPostRouteTag, TwitterSourceUnsetSelectRouteTag, TwitterSourcesListRouteTag,
        TwitterWorkerCreatePostRouteTag, TwitterWorkerEditPostRouteTag, TwitterWorkerSelectRouteTag,
        TwitterWorkersListRouteTag,
    },
};

fn twitter_menu(active: &str) -> Markup {
    let list_url = TwitterListRouteTag.url();
    let workers_url = TwitterWorkersListRouteTag.url();
    let sources_url = TwitterSourcesListRouteTag.url();
    let prefs_url = PrefsGetRouteTag.url();
    seer_menu(
        "Twitter",
        html! {
            (menu_item("Posts", &list_url, active == "posts"))
            (menu_item("Workers", &workers_url, active == "workers"))
            (menu_item("Sources", &sources_url, active == "sources"))
            (menu_item("Preferences", &prefs_url, active == "preferences"))
        },
    )
}

fn prefs_crumbs() -> Markup {
    let list_url = TwitterListRouteTag.url();
    detail_crumbs("Twitter", &list_url, "Preferences")
}

seer_common::seer_scaffold_page!(
    TwitterListPage,
    "Twitter — Seer",
    twitter_menu("posts"),
    list_crumbs("Posts")
);
seer_common::seer_scaffold_page!(
    TwitterPostDetailPage,
    "Twitter — Seer",
    twitter_menu("posts"),
    list_crumbs("Posts")
);
seer_common::seer_scaffold_page!(
    TwitterWorkersListPage,
    "Twitter — Seer",
    twitter_menu("workers"),
    list_crumbs("Workers")
);
seer_common::seer_scaffold_page!(
    TwitterWorkerDetailPage,
    "Twitter — Seer",
    twitter_menu("workers"),
    list_crumbs("Workers")
);
seer_common::seer_scaffold_page!(
    TwitterSourcesListPage,
    "Twitter — Seer",
    twitter_menu("sources"),
    list_crumbs("Sources")
);
seer_common::seer_scaffold_page!(
    TwitterSourceDetailPage,
    "Twitter — Seer",
    twitter_menu("sources"),
    list_crumbs("Sources")
);

#[derive(Generic)]
pub struct TwitterPreferencesPage {
    pub nitter_instance_url: String,
    pub error: String,
}

impl TwitterPreferencesPage {
    fn body(&self) -> Markup {
        form(FormOpts {
            attrs: form_hx_post_url::<MainContentKey>(&PrefsPostRouteTag.path())
                .set("hx-swap", "outerHTML"),
            title: "Twitter Preferences",
            subtitle: "Nitter instance used for RSS ingest",
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: PreferencesForm::render_inputs(
                &FormCtx::form::<PreferencesForm>().value(
                    PreferencesFormField::NitterInstanceUrl,
                    self.nitter_instance_url.as_str(),
                ),
            ),
            actions: html! {
                (button_submit(ButtonSubmit {
                    label: "Save Preferences",
                    ..Default::default()
                }))
            },
            ..Default::default()
        })
    }
}

impl RenderAppPane for TwitterPreferencesPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(twitter_menu("preferences"), prefs_crumbs(), self.body())
    }

    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(prefs_crumbs(), self.body())
    }
}

impl RenderTemplate for TwitterPreferencesPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Twitter Preferences — Seer",
            chrome,
            twitter_menu("preferences"),
            prefs_crumbs(),
            self.body(),
        )
    }
}

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
pub struct TwitterWorkerCreatePage {
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
    pub name: String,
    pub duration: String,
    pub sources: Vec<lariv_rs::components::ManyToManyItem>,
    pub error: String,
}

impl RenderTemplate for TwitterWorkerCreatePage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            TwitterWorkerCreateModalKey::FORM_NAME
        } else {
            self.form_name.as_str()
        };
        let ctx = FormCtx::form::<WorkerForm>()
            .value(WorkerFormField::Name, &self.name)
            .value(WorkerFormField::Duration, &self.duration)
            .m2m(WorkerFormField::SourceIds, &self.sources);
        modal_keyed::<TwitterWorkerCreateModalKey>(
            "",
            form(FormOpts {
                title: "Create Worker",
                attrs: form_hx_post_url::<TwitterWorkerCreateModalKey>(&modal_create_post_query(
                    TwitterWorkerCreatePostRouteTag,
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
pub struct TwitterWorkerEditPage {
    pub id: i64,
    pub name: String,
    pub duration: String,
    pub sources: Vec<lariv_rs::components::ManyToManyItem>,
    pub error: String,
}

impl TwitterWorkerEditPage {
    fn body(&self) -> Markup {
        worker_form_body(
            &self.name,
            &self.duration,
            &self.sources,
            &self.error,
            &TwitterWorkerEditPostRouteTag::new(self.id).path(),
        )
    }
}

impl RenderAppPane for TwitterWorkerEditPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            twitter_menu("workers"),
            detail_crumbs("Workers", &TwitterWorkersListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            detail_crumbs("Workers", &TwitterWorkersListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
}

impl RenderTemplate for TwitterWorkerEditPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Edit Worker — Seer",
            chrome,
            twitter_menu("workers"),
            detail_crumbs("Workers", &TwitterWorkersListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn source_form_body(
    runner_id: Option<i64>,
    runner_display: &str,
    usernames: &[String],
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
        .value(SourceFormField::TwitterRunnerId, runner_s.as_str())
        .display(SourceFormField::TwitterRunnerId, runner_display)
        .list(SourceFormField::Usernames, usernames)
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
pub struct TwitterSourceCreatePage {
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
    pub twitter_runner_id: Option<i64>,
    pub runner_display: String,
    pub usernames: Vec<String>,
    pub search_query: String,
    pub is_filter_whitelist: bool,
    pub filter: String,
    pub max_fresh_posts: i64,
    pub load_websites: bool,
    pub error: String,
}

impl RenderTemplate for TwitterSourceCreatePage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            TwitterSourceCreateModalKey::FORM_NAME
        } else {
            self.form_name.as_str()
        };
        let runner_s = self
            .twitter_runner_id
            .filter(|&i| i > 0)
            .map(|i| i.to_string())
            .unwrap_or_default();
        let max_s = self.max_fresh_posts.to_string();
        let mut ctx = FormCtx::form::<SourceForm>()
            .value(SourceFormField::TwitterRunnerId, runner_s.as_str())
            .display(SourceFormField::TwitterRunnerId, &self.runner_display)
            .list(SourceFormField::Usernames, &self.usernames)
            .value(SourceFormField::SearchQuery, &self.search_query)
            .value(SourceFormField::Filter, &self.filter)
            .value(SourceFormField::MaxFreshPosts, max_s.as_str());
        if self.is_filter_whitelist {
            ctx = ctx.value(SourceFormField::IsFilterWhitelist, "true");
        }
        if self.load_websites {
            ctx = ctx.value(SourceFormField::LoadWebsites, "true");
        }
        modal_keyed::<TwitterSourceCreateModalKey>(
            "",
            form(FormOpts {
                title: "Create Source",
                attrs: form_hx_post_url::<TwitterSourceCreateModalKey>(&modal_create_post_query(
                    TwitterSourceCreatePostRouteTag,
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
pub struct TwitterSourceEditPage {
    pub id: i64,
    pub twitter_runner_id: Option<i64>,
    pub runner_display: String,
    pub usernames: Vec<String>,
    pub search_query: String,
    pub is_filter_whitelist: bool,
    pub filter: String,
    pub max_fresh_posts: i64,
    pub load_websites: bool,
    pub error: String,
}

impl TwitterSourceEditPage {
    fn body(&self) -> Markup {
        source_form_body(
            self.twitter_runner_id,
            &self.runner_display,
            &self.usernames,
            &self.search_query,
            self.is_filter_whitelist,
            &self.filter,
            self.max_fresh_posts,
            self.load_websites,
            &self.error,
            &TwitterSourceEditPostRouteTag::new(self.id).path(),
        )
    }
}

impl RenderAppPane for TwitterSourceEditPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            twitter_menu("sources"),
            detail_crumbs("Sources", &TwitterSourcesListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            detail_crumbs("Sources", &TwitterSourcesListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
}

impl RenderTemplate for TwitterSourceEditPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Edit Source — Seer",
            chrome,
            twitter_menu("sources"),
            detail_crumbs("Sources", &TwitterSourcesListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
}

#[derive(Generic)]
pub struct TwitterWorkerSelectPage {
    pub runners: ObjectList<WorkerSelectRow>,
    pub filter_name: String,
    pub target_input: String,
    pub path_and_query: String,
}

impl RenderPickerSelect<TwitterWorkerSelectTableKey, TwitterWorkerSelectModalKey>
    for TwitterWorkerSelectPage
{
    fn render_table(&self) -> Markup {
        let target = if self.target_input.is_empty() {
            "twitter_runner_id"
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
                        TwitterWorkerSelectTableKey,
                        TwitterWorkerSelectModalKey,
                        TwitterWorkerSelectRouteTag,
                    >(TwitterWorkerSelectRouteTag),
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
            (picker_create_button::<TwitterWorkerCreateModalKey>(
                &self.target_input,
                Some("plus"),
                "btn-square btn-outline btn-sm",
            ))
        };
        data_table_list_refresh::<TwitterWorkerSelectTableKey>(
            "Select Worker",
            actions,
            &headers,
            &rows,
            html! {},
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for TwitterWorkerSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

#[derive(Generic)]
pub struct TwitterSourceUnsetSelectPage {
    pub sources: ObjectList<SourceSelectRow>,
    pub filter_name: String,
    pub target_input: String,
    pub path_and_query: String,
}

impl RenderPickerSelect<TwitterSourceUnsetSelectTableKey, TwitterSourceUnsetSelectModalKey>
    for TwitterSourceUnsetSelectPage
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
                        TwitterSourceUnsetSelectTableKey,
                        TwitterSourceUnsetSelectModalKey,
                        TwitterSourceUnsetSelectRouteTag,
                    >(TwitterSourceUnsetSelectRouteTag),
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
            (picker_create_button::<TwitterSourceCreateModalKey>(
                &self.target_input,
                Some("plus"),
                "btn-square btn-outline btn-sm",
            ))
        };
        data_table_list_refresh::<TwitterSourceUnsetSelectTableKey>(
            "Select Sources",
            actions,
            &headers,
            &rows,
            html! {},
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for TwitterSourceUnsetSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

lariv_rs::define_register_items! {
    plugin: SeerTwitterTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        ListIdx: TwitterListPageTag => TwitterListPage,
        DetailIdx: TwitterPostDetailPageTag => TwitterPostDetailPage,
        PrefsIdx: TwitterPreferencesPageTag => TwitterPreferencesPage,
        WorkersListIdx: TwitterWorkersListPageTag => TwitterWorkersListPage,
        WorkerDetailIdx: TwitterWorkerDetailPageTag => TwitterWorkerDetailPage,
        WorkerCreateIdx: TwitterWorkerCreatePageTag => TwitterWorkerCreatePage,
        WorkerEditIdx: TwitterWorkerEditPageTag => TwitterWorkerEditPage,
        SourcesListIdx: TwitterSourcesListPageTag => TwitterSourcesListPage,
        SourceDetailIdx: TwitterSourceDetailPageTag => TwitterSourceDetailPage,
        SourceCreateIdx: TwitterSourceCreatePageTag => TwitterSourceCreatePage,
        SourceEditIdx: TwitterSourceEditPageTag => TwitterSourceEditPage,
        WorkerSelectIdx: TwitterWorkerSelectPageTag => TwitterWorkerSelectPage,
        SourceUnsetSelectIdx: TwitterSourceUnsetSelectPageTag => TwitterSourceUnsetSelectPage,
    ]
}
