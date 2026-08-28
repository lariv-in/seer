//! Compile smoke test for the Seer deployment plugin stack.

#![recursion_limit = "512"]

use std::path::PathBuf;

use lariv_rs::app::App;
use lariv_rs::plugins::{dashboard, filesystem, llm_assistant, otp, pwa, users};

const STACK_SIZE: usize = 64 * 1024 * 1024;

const MINIMAL_DB_TOML: &str = r##"database_url = "sqlite::memory:"
[users]
adminEmail = "admin@test.local"
adminPassword = "adminadmin"
signingKey = "dGVzdC1zaWduaW5nLWtleS1wYWRkZWQtdG8tNjQtYnl0ZXMhISEhISEhISEhISE="
jwtIssuer = "c2Vlci10ZXN0LWlzc3Vlci1wYWRkZWQtdG8tNjQtYnl0ZXMhISEhISEhISEhISE="

[filesystem]
storageBackend = "local"
localDir = "/tmp/seer-test-fs"

[pwa]
PWA_APP_NAME = "Seer"
PWA_APP_DESCRIPTION = "Seer test"
PWA_THEME_COLOR = "#0A0302"
PWA_BACKGROUND_COLOR = "#ffffff"
PWA_APP_DISPLAY = "standalone"
PWA_APP_SCOPE = "/"
PWA_APP_ORIENTATION = "any"
PWA_APP_START_URL = "/"
staticDir = "./pwa_static"
"##;

fn temp_config(body: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "seer-mount-{}-{}.toml",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    std::fs::write(&path, body).expect("write temp config");
    path
}

#[test]
fn seer_stack_mounts() {
    std::thread::Builder::new()
        .name("seer-mount".into())
        .stack_size(STACK_SIZE)
        .spawn(|| {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            rt.block_on(async {
                let app = App::new_web_app();
                let app = users::install(app);
                let app = otp::install(app);
                let app = pwa::install(app);
                let app = filesystem::install(app);
                let app = llm_assistant::install(app);

                let app = seer_workerregistry::install(app);
                let app = seer_intel::install(app);
                let app = seer_node_fleet::install(app);
                let app = seer_websites::install(app);
                let app = seer_reddit::install(app);
                let app = seer_twitter::install(app);
                // let app = seer_gdelt::install(app);
                let app = seer_opensky::install(app);
                let app = seer_aisstream::install(app);
                let app = seer_assistant::install(app);
                let app = dashboard::install(app);

                let path = temp_config(MINIMAL_DB_TOML);
                let app = app.load_config(&path).await.expect("load_config");
                std::fs::remove_file(&path).ok();
                let _mounted = app.mount();
            });
        })
        .expect("spawn seer-mount thread")
        .join()
        .expect("seer-mount thread");
}
