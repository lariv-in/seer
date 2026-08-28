pub mod intel;
pub mod intel_event;
pub mod intel_preferences;

pub use intel::{Entity as IntelEntity, Model as Intel};
pub use intel_event::{Entity as IntelEventEntity, Model as IntelEvent};
pub use intel_preferences::{Entity as IntelPreferencesEntity, Model as IntelPreferences};
