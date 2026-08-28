//! Request form structs for Intel preferences.

use lariv_rs::html_form::{
    html_form,
    widgets::{Select, Text},
};

#[html_form]
pub struct PreferencesForm {
    #[form(label = "Geocoding API key", widget = Text)]
    pub geocoding_api_key: String,

    #[form(label = "Gemini API key", widget = Text)]
    pub api_key: String,

    #[form(label = "Title model", widget = Select, required, choices = "title_model")]
    pub title_model: String,

    #[form(label = "Summary model", widget = Select, required, choices = "summary_model")]
    pub summary_model: String,

    #[form(
        label = "Embedding model",
        widget = Select,
        required,
        choices = "embedding_model"
    )]
    pub embedding_model: String,
}
