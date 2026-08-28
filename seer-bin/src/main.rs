#![recursion_limit = "512"]

use lariv_rs::app::App;
use lariv_rs::plugins::{dashboard, filesystem, llm_assistant, otp, pwa, users};
use tracing_subscriber::EnvFilter;

#[lariv_rs::main(
    stack_size = 64 * 1024 * 1024,
    flavor = "multi_thread",
    thread_name = "seer-server"
)]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("warn".parse().expect("directive")),
        )
        .init();

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
    // let app = seer_twitter::install(app);
    // let app = seer_gdelt::install(app);
    let app = seer_opensky::install(app);
    let app = seer_aisstream::install(app);
    let app = seer_assistant::install(app);

    let app = dashboard::install(app);

    let app = app.load_config("config.toml").await?;
    let app = app.mount();
    app.run_migrations().await?;
    app.run().await?;
    Ok(())
}
