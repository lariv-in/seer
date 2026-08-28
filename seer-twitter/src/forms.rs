//! Request form structs for Twitter preferences, workers and sources.

use lariv_rs::html_form::{
    html_form,
    widgets::{Checkbox, Duration, ForeignKey, List, ManyToMany, Number, Text, Textarea},
};

#[html_form]
pub struct PreferencesForm {
    #[form(label = "Nitter instance URL", widget = Text)]
    pub nitter_instance_url: String,
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
        url = "/seer-twitter/sources/unset/select/",
        swap_key = "fk-twitter-worker-sources",
        placeholder = "Select unassigned sources..."
    )]
    pub source_ids: Vec<i64>,
}

#[html_form]
pub struct SourceForm {
    #[form(
        label = "Worker",
        widget = ForeignKey,
        url = "/seer-twitter/workers/select/",
        swap_key = "fk-twitter-source-runner",
        display = "runner",
        placeholder = "Select a worker..."
    )]
    pub twitter_runner_id: Option<i64>,

    #[form(
        label = "Usernames",
        required,
        widget = List,
        placeholder = "username (no @)"
    )]
    pub usernames: Vec<String>,

    #[form(label = "Search query", widget = Text)]
    pub search_query: String,

    #[form(label = "Whitelist filter", widget = Checkbox)]
    pub is_filter_whitelist: bool,

    #[form(label = "Filter patterns", widget = Textarea, rows = 4)]
    pub filter: String,

    #[form(label = "Max fresh posts", widget = Number)]
    pub max_fresh_posts: i64,

    #[form(label = "Load websites", widget = Checkbox)]
    pub load_websites: bool,
}

#[html_form]
pub struct NameFilterForm {
    #[form(label = "Name", widget = Text)]
    pub name: String,
}
