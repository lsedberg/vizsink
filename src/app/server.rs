use std::sync::Arc;

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::sync::{RwLock, broadcast};

use crate::dsl::commands::Op;

/// Global Server State
///
/// Will keep track of sent operations, such that new connections can synchronize.
#[derive(Clone)]
pub struct ServerState {
    pub ops: Arc<RwLock<Vec<Op>>>,
    pub stdin_tx: broadcast::Sender<Vec<Op>>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Data {
    Snapshot { state: Vec<Op> },
    Live { new_ops: Vec<Op> },
}

/// Upgrades and handles socket.
pub async fn handle_ws(
    ws: WebSocketUpgrade,
    State(server_state): State<ServerState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, server_state))
}

/// Handles the socket connection.
/// Both reads and writes.
async fn handle_socket(socket: WebSocket, server_state: ServerState) {
    // split socket to read and write concurrently
    let (mut sender, mut receiver) = socket.split();

    // 1) send snapshot on connection
    let snapshot = {
        let ops = server_state.ops.read().await;
        Data::Snapshot { state: ops.clone() }
    };

    if let Ok(snapshot_json) = serde_json::to_string(&snapshot) {
        let _ = sender.send(Message::Text(snapshot_json.into())).await;
    }

    // 2) subscribe to live ops
    let mut rx = server_state.stdin_tx.subscribe();

    let sender_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(new_ops) => {
                    let live_ops = Data::Live { new_ops };

                    if let Ok(live_ops_json) = serde_json::to_string(&live_ops) {
                        if sender
                            .send(Message::Text(live_ops_json.into()))
                            .await
                            .is_err()
                        {
                            // client disconnected
                            break;
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    // optionally handle lagging
                    continue;
                }
                Err(_) => break,
            }
        }
    });

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Close(_)) | Err(_) => break,
            _ => { /* ignore or handle incoming messages */ }
        }
    }

    let _ = sender_task.abort();
}
