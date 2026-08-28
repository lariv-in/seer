use parking_lot::RwLock;
use rand::Rng;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tokio::sync::mpsc;
use tracing::warn;

use crate::messages::{self, Command, Response};

#[derive(Clone, Debug)]
pub struct ConnectedNode {
    pub id: u64,
    pub version: Option<messages::VersionResponse>,
}

pub(crate) struct NodeConn {
    cmd_tx: mpsc::Sender<Command>,
    resp_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<Response>>>,
    version: Option<messages::VersionResponse>,
    closed: Arc<std::sync::atomic::AtomicBool>,
}

fn nodes() -> &'static RwLock<HashMap<u64, Arc<NodeConn>>> {
    static N: OnceLock<RwLock<HashMap<u64, Arc<NodeConn>>>> = OnceLock::new();
    N.get_or_init(|| RwLock::new(HashMap::new()))
}

pub(crate) fn register_node(
    id: u64,
    cmd_tx: mpsc::Sender<Command>,
    resp_rx: mpsc::Receiver<Response>,
    version: Option<messages::VersionResponse>,
) -> Arc<NodeConn> {
    let conn = Arc::new(NodeConn {
        cmd_tx,
        resp_rx: Arc::new(tokio::sync::Mutex::new(resp_rx)),
        version,
        closed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    });
    if let Some(old) = nodes().write().insert(id, conn.clone()) {
        old.closed
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
    conn
}

pub(crate) fn unregister_node(id: u64, conn: &Arc<NodeConn>) {
    let mut map = nodes().write();
    if let Some(cur) = map.get(&id) {
        if Arc::ptr_eq(cur, conn) {
            map.remove(&id);
        }
    }
    conn.closed
        .store(true, std::sync::atomic::Ordering::SeqCst);
}

pub fn connected_nodes() -> Vec<ConnectedNode> {
    let map = nodes().read();
    let mut out: Vec<_> = map
        .iter()
        .map(|(id, c)| ConnectedNode {
            id: *id,
            version: c.version.clone(),
        })
        .collect();
    out.sort_by_key(|n| n.id);
    out
}

/// Send command to a random connected node; waits until a node is available.
pub async fn dispatch_command(cmd: Command) -> anyhow::Result<Response> {
    loop {
        let ids: Vec<u64> = nodes().read().keys().copied().collect();
        if ids.is_empty() {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        }
        let node_id = {
            let mut rng = rand::rng();
            ids[rng.random_range(0..ids.len())]
        };
        let Some(conn) = nodes().read().get(&node_id).cloned() else {
            continue;
        };
        if conn
            .closed
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            continue;
        }
        if conn.cmd_tx.send(cmd.clone()).await.is_err() {
            warn!("seer-node-fleet: send failed node={node_id}");
            unregister_node(node_id, &conn);
            continue;
        }
        let mut rx = conn.resp_rx.lock().await;
        match rx.recv().await {
            Some(resp) => return Ok(resp),
            None => {
                warn!("seer-node-fleet: response channel closed node={node_id}");
                drop(rx);
                unregister_node(node_id, &conn);
                continue;
            }
        }
    }
}

/// Go-parity name for [`dispatch_command`].
#[allow(non_snake_case)]
pub async fn DispatchCommand(cmd: Command) -> anyhow::Result<Response> {
    dispatch_command(cmd).await
}
