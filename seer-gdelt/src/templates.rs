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
use crate::SeerGdeltTag;
use crate::{
    forms::{
        NameFilterForm, NameFilterFormField, PreferencesForm, PreferencesFormField, SourceForm,
        SourceFormField, WorkerForm, WorkerFormField,
    },
    handlers::{sources::SourceSelectRow, workers::WorkerSelectRow},
    keys::{
        GdeltSourceCreateModalKey, GdeltSourceUnsetSelectModalKey, GdeltSourceUnsetSelectTableKey,
        GdeltWorkerCreateModalKey, GdeltWorkerSelectModalKey, GdeltWorkerSelectTableKey,
    },
    routes::{
        GdeltListRouteTag, GdeltSourceCreatePostRouteTag, GdeltSourceEditPostRouteTag,
        GdeltSourceUnsetSelectRouteTag, GdeltSourcesListRouteTag, GdeltWorkerCreatePostRouteTag,
        GdeltWorkerEditPostRouteTag, GdeltWorkerSelectRouteTag, GdeltWorkersListRouteTag,
        PrefsGetRouteTag, PrefsPostRouteTag,
    },
};

fn gdelt_menu(active: &str) -> Markup {
    let list_url = GdeltListRouteTag.url();
    let workers_url = GdeltWorkersListRouteTag.url();
    let sources_url = GdeltSourcesListRouteTag.url();
    let prefs_url = PrefsGetRouteTag.url();
    seer_menu(
        "GDELT",
        html! {
            (menu_item("Events", &list_url, active == "events"))
            (menu_item("Workers", &workers_url, active == "workers"))
            (menu_item("Sources", &sources_url, active == "sources"))
            (menu_item("Preferences", &prefs_url, active == "preferences"))
        },
    )
}

fn prefs_crumbs() -> Markup {
    let list_url = GdeltListRouteTag.url();
    detail_crumbs("GDELT", &list_url, "Preferences")
}

seer_common::seer_scaffold_page!(
    GdeltListPage,
    "GDELT — Seer",
    gdelt_menu("events"),
    list_crumbs("Events")
);
seer_common::seer_scaffold_page!(
    GdeltWorkersListPage,
    "GDELT — Seer",
    gdelt_menu("workers"),
    list_crumbs("Workers")
);
seer_common::seer_scaffold_page!(
    GdeltWorkerDetailPage,
    "GDELT — Seer",
    gdelt_menu("workers"),
    list_crumbs("Workers")
);
seer_common::seer_scaffold_page!(
    GdeltSourcesListPage,
    "GDELT — Seer",
    gdelt_menu("sources"),
    list_crumbs("Sources")
);
seer_common::seer_scaffold_page!(
    GdeltSourceDetailPage,
    "GDELT — Seer",
    gdelt_menu("sources"),
    list_crumbs("Sources")
);

#[derive(Generic)]
pub struct GdeltPreferencesPage {
    pub project_id: String,
    pub error: String,
}

impl GdeltPreferencesPage {
    fn body(&self) -> Markup {
        form(FormOpts {
            attrs: form_hx_post_url::<MainContentKey>(&PrefsPostRouteTag.path())
                .set("hx-swap", "outerHTML"),
            title: "GDELT Preferences",
            subtitle: "BigQuery project ID (ADC / GOOGLE_APPLICATION_CREDENTIALS stay env-based)",
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: PreferencesForm::render_inputs(
                &FormCtx::form::<PreferencesForm>()
                    .value(PreferencesFormField::ProjectId, self.project_id.as_str()),
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

impl RenderAppPane for GdeltPreferencesPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(gdelt_menu("preferences"), prefs_crumbs(), self.body())
    }

    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(prefs_crumbs(), self.body())
    }
}

impl RenderTemplate for GdeltPreferencesPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "GDELT Preferences — Seer",
            chrome,
            gdelt_menu("preferences"),
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
pub struct GdeltWorkerCreatePage {
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
    pub name: String,
    pub duration: String,
    pub sources: Vec<lariv_rs::components::ManyToManyItem>,
    pub error: String,
}

impl RenderTemplate for GdeltWorkerCreatePage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            GdeltWorkerCreateModalKey::FORM_NAME
        } else {
            self.form_name.as_str()
        };
        let ctx = FormCtx::form::<WorkerForm>()
            .value(WorkerFormField::Name, &self.name)
            .value(WorkerFormField::Duration, &self.duration)
            .m2m(WorkerFormField::SourceIds, &self.sources);
        modal_keyed::<GdeltWorkerCreateModalKey>(
            "",
            form(FormOpts {
                title: "Create Worker",
                attrs: form_hx_post_url::<GdeltWorkerCreateModalKey>(&modal_create_post_query(
                    GdeltWorkerCreatePostRouteTag,
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
pub struct GdeltWorkerEditPage {
    pub id: i64,
    pub name: String,
    pub duration: String,
    pub sources: Vec<lariv_rs::components::ManyToManyItem>,
    pub error: String,
}

impl GdeltWorkerEditPage {
    fn body(&self) -> Markup {
        worker_form_body(
            &self.name,
            &self.duration,
            &self.sources,
            &self.error,
            &GdeltWorkerEditPostRouteTag::new(self.id).path(),
        )
    }
}

impl RenderAppPane for GdeltWorkerEditPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            gdelt_menu("workers"),
            detail_crumbs("Workers", &GdeltWorkersListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            detail_crumbs("Workers", &GdeltWorkersListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
}

impl RenderTemplate for GdeltWorkerEditPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Edit Worker — Seer",
            chrome,
            gdelt_menu("workers"),
            detail_crumbs("Workers", &GdeltWorkersListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn source_form_body(
    worker_id: Option<i64>,
    worker_display: &str,
    query: &str,
    domain: &str,
    action_country: &str,
    start_date: &str,
    end_date: &str,
    min_mentions: i64,
    max_records: i64,
    sort: &str,
    natural_language_filter: &str,
    is_blacklist: bool,
    error: &str,
    post_path: &str,
) -> Markup {
    let worker_s = worker_id
        .filter(|&i| i > 0)
        .map(|i| i.to_string())
        .unwrap_or_default();
    let min_s = min_mentions.to_string();
    let max_s = max_records.to_string();
    let mut ctx = FormCtx::form::<SourceForm>()
        .value(SourceFormField::GdeltWorkerId, worker_s.as_str())
        .display(SourceFormField::GdeltWorkerId, worker_display)
        .value(SourceFormField::Query, query)
        .value(SourceFormField::Domain, domain)
        .value(SourceFormField::ActionCountry, action_country)
        .value(SourceFormField::StartDate, start_date)
        .value(SourceFormField::EndDate, end_date)
        .value(SourceFormField::MinMentions, min_s.as_str())
        .value(SourceFormField::MaxRecords, max_s.as_str())
        .value(SourceFormField::Sort, sort)
        .value(
            SourceFormField::NaturalLanguageFilter,
            natural_language_filter,
        );
    if is_blacklist {
        ctx = ctx.value(SourceFormField::IsBlacklist, "true");
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
pub struct GdeltSourceCreatePage {
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
    pub gdelt_worker_id: Option<i64>,
    pub worker_display: String,
    pub query: String,
    pub domain: String,
    pub action_country: String,
    pub start_date: String,
    pub end_date: String,
    pub min_mentions: i64,
    pub max_records: i64,
    pub sort: String,
    pub natural_language_filter: String,
    pub is_blacklist: bool,
    pub error: String,
}

impl RenderTemplate for GdeltSourceCreatePage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            GdeltSourceCreateModalKey::FORM_NAME
        } else {
            self.form_name.as_str()
        };
        let mut ctx = FormCtx::form::<SourceForm>()
            .value(
                SourceFormField::GdeltWorkerId,
                self.gdelt_worker_id
                    .filter(|&i| i > 0)
                    .map(|i| i.to_string())
                    .unwrap_or_default(),
            )
            .display(SourceFormField::GdeltWorkerId, &self.worker_display)
            .value(SourceFormField::Query, &self.query)
            .value(SourceFormField::Domain, &self.domain)
            .value(SourceFormField::ActionCountry, &self.action_country)
            .value(SourceFormField::StartDate, &self.start_date)
            .value(SourceFormField::EndDate, &self.end_date)
            .value(SourceFormField::MinMentions, self.min_mentions.to_string())
            .value(SourceFormField::MaxRecords, self.max_records.to_string())
            .value(SourceFormField::Sort, &self.sort)
            .value(
                SourceFormField::NaturalLanguageFilter,
                &self.natural_language_filter,
            );
        if self.is_blacklist {
            ctx = ctx.value(SourceFormField::IsBlacklist, "true");
        }
        modal_keyed::<GdeltSourceCreateModalKey>(
            "",
            form(FormOpts {
                title: "Create Source",
                attrs: form_hx_post_url::<GdeltSourceCreateModalKey>(&modal_create_post_query(
                    GdeltSourceCreatePostRouteTag,
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
pub struct GdeltSourceEditPage {
    pub id: i64,
    pub gdelt_worker_id: Option<i64>,
    pub worker_display: String,
    pub query: String,
    pub domain: String,
    pub action_country: String,
    pub start_date: String,
    pub end_date: String,
    pub min_mentions: i64,
    pub max_records: i64,
    pub sort: String,
    pub natural_language_filter: String,
    pub is_blacklist: bool,
    pub error: String,
}

impl GdeltSourceEditPage {
    fn body(&self) -> Markup {
        source_form_body(
            self.gdelt_worker_id,
            &self.worker_display,
            &self.query,
            &self.domain,
            &self.action_country,
            &self.start_date,
            &self.end_date,
            self.min_mentions,
            self.max_records,
            &self.sort,
            &self.natural_language_filter,
            self.is_blacklist,
            &self.error,
            &GdeltSourceEditPostRouteTag::new(self.id).path(),
        )
    }
}

impl RenderAppPane for GdeltSourceEditPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            gdelt_menu("sources"),
            detail_crumbs("Sources", &GdeltSourcesListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            detail_crumbs("Sources", &GdeltSourcesListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
}

impl RenderTemplate for GdeltSourceEditPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Edit Source — Seer",
            chrome,
            gdelt_menu("sources"),
            detail_crumbs("Sources", &GdeltSourcesListRouteTag.url(), "Edit"),
            self.body(),
        )
    }
}

#[derive(Generic)]
pub struct GdeltWorkerSelectPage {
    pub workers: ObjectList<WorkerSelectRow>,
    pub filter_name: String,
    pub target_input: String,
    pub path_and_query: String,
}

impl RenderPickerSelect<GdeltWorkerSelectTableKey, GdeltWorkerSelectModalKey>
    for GdeltWorkerSelectPage
{
    fn render_table(&self) -> Markup {
        let target = if self.target_input.is_empty() {
            "gdelt_worker_id"
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
            .workers
            .items
            .iter()
            .map(|w| TableRow {
                attrs: row_attr_select(target, &w.id.to_string(), &w.name),
                cells: vec![field_text(FieldText {
                    value: &w.name,
                    classes: "",
                })],
            })
            .collect();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        GdeltWorkerSelectTableKey,
                        GdeltWorkerSelectModalKey,
                        GdeltWorkerSelectRouteTag,
                    >(GdeltWorkerSelectRouteTag),
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
            (picker_create_button::<GdeltWorkerCreateModalKey>(
                &self.target_input,
                Some("plus"),
                "btn-square btn-outline btn-sm",
            ))
        };
        data_table_list_refresh::<GdeltWorkerSelectTableKey>(
            "Select Worker",
            actions,
            &headers,
            &rows,
            html! {},
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for GdeltWorkerSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

#[derive(Generic)]
pub struct GdeltSourceUnsetSelectPage {
    pub sources: ObjectList<SourceSelectRow>,
    pub filter_name: String,
    pub target_input: String,
    pub path_and_query: String,
}

impl RenderPickerSelect<GdeltSourceUnsetSelectTableKey, GdeltSourceUnsetSelectModalKey>
    for GdeltSourceUnsetSelectPage
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
                        GdeltSourceUnsetSelectTableKey,
                        GdeltSourceUnsetSelectModalKey,
                        GdeltSourceUnsetSelectRouteTag,
                    >(GdeltSourceUnsetSelectRouteTag),
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
            (picker_create_button::<GdeltSourceCreateModalKey>(
                &self.target_input,
                Some("plus"),
                "btn-square btn-outline btn-sm",
            ))
        };
        data_table_list_refresh::<GdeltSourceUnsetSelectTableKey>(
            "Select Sources",
            actions,
            &headers,
            &rows,
            html! {},
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for GdeltSourceUnsetSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

lariv_rs::define_register_items! {
    plugin: SeerGdeltTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        ListIdx: GdeltListPageTag => GdeltListPage,
        PrefsIdx: GdeltPreferencesPageTag => GdeltPreferencesPage,
        WorkersListIdx: GdeltWorkersListPageTag => GdeltWorkersListPage,
        WorkerDetailIdx: GdeltWorkerDetailPageTag => GdeltWorkerDetailPage,
        WorkerCreateIdx: GdeltWorkerCreatePageTag => GdeltWorkerCreatePage,
        WorkerEditIdx: GdeltWorkerEditPageTag => GdeltWorkerEditPage,
        SourcesListIdx: GdeltSourcesListPageTag => GdeltSourcesListPage,
        SourceDetailIdx: GdeltSourceDetailPageTag => GdeltSourceDetailPage,
        SourceCreateIdx: GdeltSourceCreatePageTag => GdeltSourceCreatePage,
        SourceEditIdx: GdeltSourceEditPageTag => GdeltSourceEditPage,
        WorkerSelectIdx: GdeltWorkerSelectPageTag => GdeltWorkerSelectPage,
        SourceUnsetSelectIdx: GdeltSourceUnsetSelectPageTag => GdeltSourceUnsetSelectPage,
    ]
}
