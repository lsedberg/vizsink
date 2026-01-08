use std::sync::Arc;

use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::{Html, IntoResponse},
    routing::get,
};
use clap::Parser;
use color_eyre::Result;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::{
    self,
    io::{self, AsyncBufReadExt},
    sync::{RwLock, broadcast},
};

mod dsl;
mod utils;

use crate::dsl::{
    commands::{self, Op},
    parser::{ParserContext, parse_line},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, default_value_t = 8080)]
    port: u16,
}

/// Global Server State
///
/// Will keep track of sent operations, such that new connections can synchronize.
#[derive(Clone)]
struct ServerState {
    ops: Arc<RwLock<Vec<commands::Op>>>,
    stdin_tx: broadcast::Sender<Vec<Op>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let (tx, _) = broadcast::channel::<Vec<Op>>(1024);

    let server_state: ServerState = ServerState {
        ops: Arc::new(RwLock::new(Vec::new())),
        stdin_tx: tx.clone(),
    };

    // spawn stdin feeder
    let feed_state = server_state.clone();
    tokio::spawn(async move {
        let mut parser_context = ParserContext::default();
        feed_stdin(tx.clone(), &mut parser_context, feed_state).await;
    });

    let app = Router::new()
        .route("/", get(index))
        .route(
            "/ws",
            get(handle_ws), // get(move |ws: WebSocketUpgrade| handle_ws(ws, tx.clone())),
        )
        .with_state(server_state);

    let addr = format!("127.0.0.1:{}", cli.port);
    let listener = tokio::net::TcpListener::bind(addr.clone())
        .await
        .expect(format!("Could not create a listener at: {}", addr).as_str());

    println!("Serving VizSink at: `{}`", addr);
    println!("Open a browser to visualize.");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(include_str!("frontend/index.html"))
}

// async fn handle_ws(mut ws: WebSocketUpgrade, tx: broadcast::Sender<String>, State(server_state): State<ServerState>) -> impl IntoResponse {
async fn handle_ws(
    ws: WebSocketUpgrade,
    State(server_state): State<ServerState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, server_state))
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Data {
    Snapshot { state: Vec<Op> },
    Live { new_ops: Vec<Op> },
}

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

async fn feed_stdin(
    tx: broadcast::Sender<Vec<Op>>,
    parse_context: &mut ParserContext,
    state: ServerState,
) {
    let mut lines = io::BufReader::new(io::stdin()).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let line_ops = parse_line(line.clone(), parse_context);

        if line_ops.len() > 0 {
            // persist to shared state (?)
            {
                let mut ops = state.ops.write().await;
                ops.append(&mut line_ops.clone())
            }
            // broadcast to subscribers
            let _ = tx.send(line_ops);
        }
    }
}
