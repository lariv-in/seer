use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use maud::Markup;
use prost::Message as ProstMessage;
use tokio::sync::mpsc;
use tracing::{info, warn};

use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout},
};
use seer_common::{cell, col, row, seer_table_with_subtitle};

use crate::{
    dispatch::{connected_nodes, register_node, unregister_node},
    keys::FleetNodesTableKey,
    messages::{
        command, response, response_ok, Command, GetId, GetVersion, Response, VersionId,
    },
    templates::FleetHomePage,
};

struct NodeRow {
    id: String,
    version: String,
}

pub async fn home(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Markup {
    let nodes = connected_nodes();
    let data: Vec<NodeRow> = nodes
        .into_iter()
        .map(|n| NodeRow {
            id: n.id.to_string(),
            version: n
                .version
                .as_ref()
                .map(|v| format!("{}.{}.{}", v.major, v.minor, v.patch))
                .unwrap_or_else(|| "unknown".into()),
        })
        .collect();

    let headers = [col("NodeId", "Node ID"), col("Version", "Version")];
    let rows: Vec<_> = data
        .iter()
        .map(|r| row(vec![cell(&r.id), cell(&r.version)]))
        .collect();

    let body = seer_table_with_subtitle::<FleetNodesTableKey>(
        "Node Fleet",
        "WebSocket: /fleet/websocket/",
        &headers,
        &rows,
    );
    html_built_page_or_app_layout(
        &FleetHomePage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}

pub async fn fleet_ws(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_fleet_socket)
}

async fn handle_fleet_socket(socket: WebSocket) {
    let (mut sink, mut stream) = socket.split();

    // GetID
    let get_id = Command {
        id: 1,
        command_type: Some(command::CommandType::GetId(GetId {})),
    };
    if sink
        .send(Message::Binary(get_id.encode_to_vec().into()))
        .await
        .is_err()
    {
        return;
    }
    let Some(Ok(Message::Binary(data))) = stream.next().await else {
        return;
    };
    let Ok(resp) = Response::decode(data.as_ref()) else {
        return;
    };
    let Some(response::ResponseType::Ok(ok)) = resp.response_type else {
        warn!("fleet: get id error");
        return;
    };
    let Some(response_ok::Response::Id(VersionId { id: node_id })) = ok.response else {
        return;
    };
    if node_id == 0 {
        return;
    }

    // GetVersion
    let mut version = None;
    let get_ver = Command {
        id: 2,
        command_type: Some(command::CommandType::GetVersion(GetVersion {})),
    };
    if sink
        .send(Message::Binary(get_ver.encode_to_vec().into()))
        .await
        .is_ok()
    {
        if let Some(Ok(Message::Binary(data))) = stream.next().await {
            if let Ok(resp) = Response::decode(data.as_ref()) {
                if let Some(response::ResponseType::Ok(ok)) = resp.response_type {
                    if let Some(response_ok::Response::Version(v)) = ok.response {
                        version = Some(v);
                    }
                }
            }
        }
    }

    let (cmd_tx, mut cmd_rx) = mpsc::channel::<Command>(32);
    let (resp_tx, resp_rx) = mpsc::channel::<Response>(32);
    let conn = register_node(node_id, cmd_tx, resp_rx, version.clone());
    info!(
        "seer-node-fleet: node connected id={node_id} version={:?}",
        version.as_ref().map(|v| format!("{}.{}.{}", v.major, v.minor, v.patch))
    );

    loop {
        tokio::select! {
            cmd = cmd_rx.recv() => {
                let Some(cmd) = cmd else { break; };
                if sink.send(Message::Binary(cmd.encode_to_vec().into())).await.is_err() {
                    break;
                }
                let Some(Ok(msg)) = stream.next().await else { break; };
                let Message::Binary(data) = msg else { continue; };
                if let Ok(resp) = Response::decode(data.as_ref()) {
                    if let Err(e) = resp_tx.send(resp).await {
                        warn!(error = %e, "seer-node-fleet: failed to forward response");
                    }
                }
            }
            msg = stream.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(p))) => {
                        if let Err(e) = sink.send(Message::Pong(p)).await {
                            warn!(error = %e, "seer-node-fleet: failed to send pong");
                        }
                    }
                    Some(Ok(Message::Binary(data))) => {
                        // Unsolicited response — try forward
                        if let Ok(resp) = Response::decode(data.as_ref()) {
                            if let Err(e) = resp_tx.send(resp).await {
                                warn!(error = %e, "seer-node-fleet: failed to forward unsolicited response");
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    unregister_node(node_id, &conn);
    info!("seer-node-fleet: node disconnected id={node_id}");
}
