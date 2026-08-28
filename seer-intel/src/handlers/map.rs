use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use maud::{html, Markup, PreEscaped};
use tracing::warn;

use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{html_built_page_or_app_layout, Htmx},
};

use crate::{
    map::{points_in_bounds, MapBounds, LIVE_MAP_JS},
    state::IntelState,
    templates::IntelMapPage,
};

fn map_style_url() -> String {
    std::env::var("MAP_STYLE_URL").unwrap_or_default()
}

pub async fn page(
    Cap(_state): Cap<IntelState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Markup {
    let style = map_style_url();
    let body = html! {
        div class="container mx-auto p-4 space-y-4" {
            h1 class="text-2xl font-semibold" { "Intel Map" }
            p class="opacity-70" {
                "Geocoded intel events. Pan and zoom to explore; click a point to open the intel detail."
            }
            div
                data-seer-live-map
                data-ws-path="/seer-intel/map/data"
                data-map-style=(style.as_str())
                class="space-y-2"
            {
                div
                    data-seer-map-canvas
                    class="bg-base-200 rounded w-full h-[70vh] min-h-96 overflow-hidden"
                {}
                p data-seer-map-status class="text-sm opacity-70" { "Initializing…" }
            }
            (PreEscaped(format!("<script>\n{LIVE_MAP_JS}\n</script>")))
        }
    };
    html_built_page_or_app_layout(
        &IntelMapPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}

pub async fn data_ws(Cap(state): Cap<IntelState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    let db = state.db.clone();
    ws.on_upgrade(move |socket| handle_map_ws(socket, db))
}

enum BoundsCodec {
    Json,
    Cbor,
}

async fn handle_map_ws(socket: WebSocket, db: sea_orm::DatabaseConnection) {
    let (mut sink, mut stream) = socket.split();
    while let Some(Ok(msg)) = stream.next().await {
        let (bounds, codec) = match msg {
            Message::Text(t) => (
                serde_json::from_str::<MapBounds>(&t).unwrap_or_default(),
                BoundsCodec::Json,
            ),
            Message::Binary(b) => {
                let decoded = ciborium::from_reader::<MapBounds, _>(std::io::Cursor::new(b.as_ref()))
                    .map(|bounds| (bounds, BoundsCodec::Cbor))
                    .or_else(|_| {
                        serde_json::from_slice::<MapBounds>(&b)
                            .map(|bounds| (bounds, BoundsCodec::Json))
                    })
                    .unwrap_or_else(|_| (MapBounds::default(), BoundsCodec::Cbor));
                decoded
            }
            Message::Close(_) => break,
            Message::Ping(p) => {
                if let Err(e) = sink.send(Message::Pong(p)).await {
                    warn!(error = %e, "seer-intel: failed to send pong");
                }
                continue;
            }
            _ => continue,
        };
        let points = points_in_bounds(&db, &bounds).await;
        let send_ok = match codec {
            BoundsCodec::Json => match serde_json::to_string(&points) {
                Ok(j) => sink.send(Message::Text(j.into())).await.is_ok(),
                Err(e) => {
                    warn!(error = %e, "seer-intel: failed to encode map points as JSON");
                    false
                }
            },
            BoundsCodec::Cbor => {
                let mut buf = Vec::new();
                if ciborium::into_writer(&points, &mut buf).is_ok() {
                    sink.send(Message::Binary(buf.into())).await.is_ok()
                } else if let Ok(j) = serde_json::to_vec(&points) {
                    sink.send(Message::Binary(j.into())).await.is_ok()
                } else {
                    false
                }
            }
        };
        if !send_ok {
            break;
        }
    }
}
