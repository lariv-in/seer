pub mod event;
pub mod gdelt_preferences;
pub mod gdelt_source;
pub mod gdelt_worker;

pub use event::{Entity as GdeltEventEntity, Model as GdeltEvent};
pub use gdelt_preferences::{Entity as GdeltPreferencesEntity, Model as GdeltPreferences};
pub use gdelt_source::{Entity as GdeltSourceEntity, Model as GdeltSource};
pub use gdelt_worker::{Entity as GdeltWorkerEntity, Model as GdeltWorker};
