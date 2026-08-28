//! Request form structs for OpenSky preferences.

use lariv_rs::html_form::{html_form, widgets::Text};

#[html_form]
pub struct PreferencesForm {
    #[form(label = "Client ID", widget = Text)]
    pub client_id: String,

    #[form(label = "Client secret", widget = Text)]
    pub client_secret: String,
}
