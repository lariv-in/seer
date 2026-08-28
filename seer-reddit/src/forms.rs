//! Request form structs for Reddit workers and sources.

use lariv_rs::html_form::{
    html_form,
    widgets::{Checkbox, Duration, ForeignKey, List, ManyToMany, Number, Text, Textarea},
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
        url = "/seer-reddit/sources/unset/select/",
        swap_key = "fk-reddit-worker-sources",
        placeholder = "Select unassigned sources..."
    )]
    pub source_ids: Vec<i64>,
}

#[html_form]
pub struct SourceForm {
    #[form(
        label = "Worker",
        widget = ForeignKey,
        url = "/seer-reddit/workers/select/",
        swap_key = "fk-reddit-source-runner",
        display = "runner",
        placeholder = "Select a worker..."
    )]
    pub reddit_runner_id: Option<i64>,

    #[form(
        label = "Subreddits",
        required,
        widget = List,
        placeholder = "subreddit name"
    )]
    pub subreddits: Vec<String>,

    #[form(label = "Search query", widget = Text)]
    pub search_query: String,

    #[form(label = "Whitelist filter", widget = Checkbox)]
    pub is_filter_whitelist: bool,

    #[form(label = "Filter criteria", widget = Textarea, rows = 4)]
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
