//! Request form structs for Websites workers and sources.

use lariv_rs::html_form::{
    html_form,
    widgets::{Checkbox, Duration, ForeignKey, ManyToMany, Number, Text, Textarea},
};

#[html_form]
pub struct WorkerForm {
    #[form(label = "Name", required, widget = Text)]
    pub name: String,

    #[form(label = "Duration", required, widget = Duration)]
    pub duration: String,

    #[form(
        label = "Sources without worker",
        widget = ManyToMany,
        url = "/seer-websites/sources/unset/select/",
        swap_key = "fk-website-worker-sources",
        placeholder = "Select unassigned sources..."
    )]
    pub source_ids: Vec<i64>,
}

#[html_form]
pub struct SourceForm {
    #[form(
        label = "Worker",
        widget = ForeignKey,
        url = "/seer-websites/workers/select/",
        swap_key = "fk-website-source-runner",
        display = "runner",
        placeholder = "Select a worker..."
    )]
    pub website_runner_id: Option<i64>,

    #[form(
        label = "URL",
        required,
        widget = Text,
        placeholder = "https://example.com"
    )]
    pub url: String,

    #[form(label = "Depth", widget = Number)]
    pub depth: i64,

    #[form(label = "Whitelist filter", widget = Checkbox)]
    pub is_filter_whitelist: bool,

    #[form(label = "Filter criteria", widget = Textarea, rows = 4)]
    pub filter: String,
}

#[html_form]
pub struct NameFilterForm {
    #[form(label = "Name", widget = Text)]
    pub name: String,
}
