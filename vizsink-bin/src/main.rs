use include_dir::{Dir, include_dir};
use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use clap::Parser;
use color_eyre::eyre::Result;
use tokio::sync::broadcast;
use tokio::{
    io::{self, AsyncBufReadExt},
    sync::RwLock,
};

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

static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");

async fn serve_embedded(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');
    let mime_type = mime_guess::from_path(path).first_or_text_plain();

    match STATIC_DIR.get_file(path) {
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime_type.as_ref()).unwrap(),
            )
            .body(Body::from(file.contents()))
            .unwrap(),
    }
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

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/{*path}", get(serve_embedded))
        .with_state(app_state);

    let addr = format!("0.0.0.0:{}", cli.port);
    let listener = match tokio::net::TcpListener::bind(addr.clone()).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            println!("Tip: You can change the default port by using the `--port <port>` flag.");
            std::process::exit(1);
        }
    };
    // .expect("could not create listener");

    println!("Serving VizSink at: `{}/index.html`", addr);
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
