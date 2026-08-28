use futures_util::{SinkExt, StreamExt};
use nodescraper_rs::messages;
use nodescraper_rs::scrapers::Scraper;
use nodescraper_rs::scrapers::website::WebsiteScraper;
use prost::Message;
use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};
use tokio::{pin, sync::Mutex};

static DEVICE_ID: OnceLock<u64> = OnceLock::new();

const VERSION: messages::VersionResponse = messages::VersionResponse {
    major: 0,
    minor: 0,
    patch: 1,
};

fn get_remote_url() -> Result<url::Url, url::ParseError> {
    let remote_url = match option_env!("REMOTE_URL") {
        Some(s) => s,
        None => "ws://localhost:42069/fleet/websocket",
    };
    let remote_url = url::Url::parse(remote_url)?;
    Ok(remote_url)
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let remote_url = get_remote_url().expect("Error while getting the remote url");
    let scheme = remote_url.scheme();
    if scheme != "ws" && scheme != "wss" {
        panic!("Scheme for remote url should be ws or wss")
    }

    let website_scraper =
        Arc::new(WebsiteScraper::init().expect("Error while initializing the website scraper"));

    loop {
        tokio::time::sleep(Duration::new(1, 0)).await;
        let (ws_stream, _) = match tokio_tungstenite::connect_async(remote_url.clone()).await {
            Ok(conn) => conn,
            Err(e) => {
                log::error!("Error while connecting to remote {remote_url} {e}");
                continue;
            }
        };
        let (sink, stream) = ws_stream.split();

        let sink = Arc::new(Mutex::new(sink));
        let command_stream =
            stream.then(async |item| item.map(|item| messages::Command::decode(item.into_data())));
        pin!(command_stream);
        let command_stream = command_stream.for_each_concurrent(None, |command| {
            let website_scraper = Arc::clone(&website_scraper);
            let sink = Arc::clone(&sink);
            async move {
                let command = match command {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!("Error while getting the data from websocket: {e}");
                        return;
                    }
                };
                let command = match command {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!("Error while parsing data to Command format: {e}");
                        return;
                    }
                };
                let response = handle_command(command, async |id, trigger| match trigger {
                    messages::trigger_scraper::ScraperArgs::WebsiteScraper(v) => {
                        website_scraper.scrape(id, v).await
                    }
                })
                .await;
                println!("Response: {:#?}", response);
                let response = response.encode_to_vec();
                let mut sink = sink.lock_owned().await;
                match sink
                    .send(tokio_tungstenite::tungstenite::Message::binary(response))
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Error while sending back message to the server: {e}");
                    }
                };
            }
        });
        command_stream.await;
    }
}

async fn handle_command<
    O: Future<Output = messages::Response>,
    S: Fn(u64, messages::trigger_scraper::ScraperArgs) -> O,
>(
    command: messages::Command,
    scraper_handler: S,
) -> messages::Response {
    println!("Command: {:#?}", command);
    match command.command_type {
        Some(command_type) => match command_type {
            messages::command::CommandType::GetVersion(_) => messages::Response {
                command_id: command.id,
                response_type: Some(messages::response::ResponseType::Ok(messages::ResponseOk {
                    response: Some(messages::response_ok::Response::Version(VERSION)),
                })),
            },
            messages::command::CommandType::TriggerScraper(t) => {
                if let Some(t) = t.scraper_args {
                    scraper_handler(command.id, t).await
                } else {
                    messages::Response {
                        command_id: command.id,
                        response_type: Some(messages::response::ResponseType::Error(
                            messages::ResponseError {
                                response_error: Some(
                                    messages::response_error::ResponseError::MissingCommandFieldError(
                                        messages::MissingCommandFieldError {},
                                    ),
                                ),
                            },
                        )),
                    }
                }
            }
            messages::command::CommandType::GetId(_) => messages::Response {
                command_id: command.id,
                response_type: Some(messages::response::ResponseType::Ok(messages::ResponseOk {
                    response: Some(messages::response_ok::Response::Id(messages::VersionId {
                        id: *DEVICE_ID.get_or_init(|| rand::random()),
                    })),
                })),
            },
        },
        None => messages::Response {
            command_id: command.id,
            response_type: Some(messages::response::ResponseType::Error(
                messages::ResponseError {
                    response_error: Some(
                        messages::response_error::ResponseError::MissingCommandFieldError(
                            messages::MissingCommandFieldError {},
                        ),
                    ),
                },
            )),
        },
    }
}
