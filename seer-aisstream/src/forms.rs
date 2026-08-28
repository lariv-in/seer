//! Request form structs for AIS stream preferences.

use lariv_rs::html_form::{
    html_form,
    widgets::{Checkbox, Text},
};

#[html_form]
pub struct PreferencesForm {
    #[form(label = "Enabled", widget = Checkbox)]
    pub enabled: bool,

    #[form(label = "API key", widget = Text)]
    pub api_key: String,
}
