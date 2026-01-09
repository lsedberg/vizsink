use tokio::{
    io::{self, AsyncBufReadExt},
    sync::broadcast,
};

use crate::{
    app::server::ServerState,
    dsl::{
        commands::Op,
        parser::{ParserContext, parse_line},
    },
};

pub async fn feed_stdin(
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
