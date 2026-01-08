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
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::{
    self,
    io::{self, AsyncBufReadExt},
    sync::broadcast,
};

mod dsl;
mod utils;

use crate::dsl::{commands, parser::ParserContext};

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
    ops: Vec<commands::Op>,
    stdin_tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let (tx, _) = broadcast::channel::<String>(1024);

    let server_state: ServerState = ServerState {
        ops: vec![],
        stdin_tx: tx.clone(),
    };

    // spawn stdin feeder
    tokio::spawn(async move {
        let mut parser_context = ParserContext::default();
        feed_stdin(tx.clone(), &mut parser_context).await;
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

async fn handle_socket(socket: WebSocket, server_state: ServerState) {
    // split socket to read and write concurrently
    let (mut sender, mut receiver) = socket.split();
    let mut rx = server_state.stdin_tx.subscribe();

    let sender_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(text) => {
                    println!("RX Recv, sending: {}", text);
                    if sender.send(Message::Text(text.into())).await.is_err() {
                        // client disconnected
                        break;
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

async fn feed_stdin(tx: broadcast::Sender<String>, parse_context: &mut ParserContext) {
    let mut lines = io::BufReader::new(io::stdin()).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        // Parse line:
        // let parsed = parse_line(line.clone(), parse_context);

        // pass to some global server state...

        // send raw stdin line to all websocket subscribers
        let _ = tx.send(line);
    }
}
