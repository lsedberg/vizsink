use std::net::SocketAddr;

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
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::broadcast;
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let (tx, _) = broadcast::channel::<String>(1024);

    // stdin reader task
    {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut lines = io::BufReader::new(io::stdin()).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = tx.send(line);
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
        .with_state(AppState { tx });

    let addr = format!("127.0.0.1:{}", cli.port);
    let listener = tokio::net::TcpListener::bind(addr.clone())
        .await
        .expect(format!("Could not create a listener at: {}", addr).as_str());

    println!("Serving VizSink at: `{}`", addr);
    println!("Open a browser to visualize.");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        if socket.send(Message::Text(msg.into())).await.is_err() {
            break;
        }
    }
}
