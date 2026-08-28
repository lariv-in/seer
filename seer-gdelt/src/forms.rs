//! Request form structs for GDELT preferences, workers and sources.

use lariv_rs::html_form::{
    html_form,
    widgets::{Checkbox, Date, Duration, ForeignKey, ManyToMany, Number, Text, Textarea},
};

#[html_form]
pub struct PreferencesForm {
    #[form(label = "GCP project ID", widget = Text)]
    pub project_id: String,
}

#[html_form]
pub struct WorkerForm {
    #[form(label = "Name", required, widget = Text)]
    pub name: String,

    #[form(label = "Duration", required, widget = Duration)]
    pub duration: String,

    #[form(
        label = "Sources without worker",
        widget = ManyToMany,
        url = "/seer-gdelt/sources/unset/select/",
        swap_key = "fk-gdelt-worker-sources",
        placeholder = "Select unassigned sources..."
    )]
    pub source_ids: Vec<i64>,
}

#[html_form]
pub struct SourceForm {
    #[form(
        label = "Worker",
        widget = ForeignKey,
        url = "/seer-gdelt/workers/select/",
        swap_key = "fk-gdelt-source-worker",
        display = "worker",
        placeholder = "Select a worker..."
    )]
    pub gdelt_worker_id: Option<i64>,

    #[form(
        label = "Query",
        required,
        widget = Text,
        placeholder = "GDELT full-text query"
    )]
    pub query: String,

    #[form(label = "Domain", widget = Text, placeholder = "example.com")]
    pub domain: String,

    #[form(label = "Action country", widget = Text, placeholder = "ISO country code")]
    pub action_country: String,

    #[form(label = "Start date", widget = Date)]
    pub start_date: String,

    #[form(label = "End date", widget = Date)]
    pub end_date: String,

    #[form(label = "Min mentions", widget = Number)]
    pub min_mentions: i64,

    #[form(label = "Max records", widget = Number)]
    pub max_records: i64,

    #[form(label = "Sort", widget = Text, placeholder = "e.g. DateDesc")]
    pub sort: String,

    #[form(label = "Natural language filter", widget = Textarea, rows = 4)]
    pub natural_language_filter: String,

    #[form(label = "Blacklist filter", widget = Checkbox)]
    pub is_blacklist: bool,
}

#[html_form]
pub struct NameFilterForm {
    #[form(label = "Name", widget = Text)]
    pub name: String,
}
