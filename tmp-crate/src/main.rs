use std::sync::Arc;

use axum::{
    Router,
    http::StatusCode,
    routing::{get, get_service},
};
use clap::Parser;
use color_eyre::Result;
use tokio::{
    self,
    sync::{RwLock, broadcast},
};
use tower_http::services::ServeDir;

mod app;
mod dsl;
mod utils;

use crate::{
    app::server::{ServerState, handle_ws},
    dsl::{commands::Op, parser::ParserContext},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, default_value_t = 8080)]
    port: u16,
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
        app::cli::feed_stdin(tx.clone(), &mut parser_context, feed_state).await;
    });

    let static_service =
        get_service(ServeDir::new("./src/frontend")).handle_error(|err| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Static file error: {}", err),
            )
        });
    let app = Router::new()
        .route("/ws", get(handle_ws))
        .fallback(static_service)
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
