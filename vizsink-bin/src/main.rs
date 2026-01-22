use std::sync::Arc;

use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service},
};
use clap::Parser;
use color_eyre::eyre::Result;
use tokio::sync::broadcast;
use tokio::{
    io::{self, AsyncBufReadExt},
    sync::RwLock,
};
use tower_http::services::ServeDir;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, default_value_t = 8080)]
    port: u16,
}

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
    cached_lines: Arc<RwLock<Vec<String>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let (tx, _) = broadcast::channel::<String>(1024);

    let app_state = AppState {
        tx,
        cached_lines: Arc::new(RwLock::new(Vec::new())),
    };

    // stdin reader task
    {
        let app_state = app_state.clone();
        tokio::spawn(async move {
            let mut lines = io::BufReader::new(io::stdin()).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let mut cached_lines = app_state.cached_lines.write().await;
                cached_lines.push(line.clone());
                let _ = app_state.tx.send(line);
            }
        });
    }

    let static_service =
        get_service(ServeDir::new("./vizsink-bin/static")).handle_error(|err| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Static file error: {}", err),
            )
        });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .fallback(static_service)
        .with_state(app_state);

    let addr = format!("127.0.0.1:{}", cli.port);
    let listener = tokio::net::TcpListener::bind(addr.clone())
        .await
        .expect("could not create listener");

    println!("Serving VizSink at: `{}`", addr);
    println!("Open a browser to visualize.");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    // 1) send snapshot on connection
    {
        let cached_lines = state.cached_lines.read().await;
        if !cached_lines.is_empty() {
            let lines = cached_lines.join("\n");
            if socket.send(Message::Text(lines.into())).await.is_err() {
                // client disconnected
                return;
            }
        }
        // Close cached_lines read
    }
    let mut rx = state.tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        if socket.send(Message::Text(msg.into())).await.is_err() {
            break;
        }
    }
}
