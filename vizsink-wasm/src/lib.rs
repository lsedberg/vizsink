use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::Event;
use web_sys::{MessageEvent, WebSocket};

use vizsink_core::{
    ast::{self, ASTNode, ErrorNode},
    parser,
};

mod utils;

#[wasm_bindgen(start)]
pub fn start() {
    let window = web_sys::window().expect("should have a window in this context");

    let location = window.location();
    let ws_url = format!(
        "ws://{}/ws",
        &location.host().expect("window should have a host location")
    );
    let ws = WebSocket::new(&ws_url).unwrap();

    console_log!("Listening!");

    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        if let Some(txt) = e.data().as_string() {
            let parsed = parser::parse_line(&txt);

            console_log!("Received (", parsed.len(), " commands): ", &txt);
            for node in parsed {
                execute_command(node);
            }
        }
    });

    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

    let onopen = Closure::<dyn FnMut(Event)>::new(move |_| {
        console_info!("Connected");
    });

    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));

    onmessage.forget(); // keep alive
    onopen.forget(); // keep alive
}

fn execute_command(node: ASTNode) {
    match node {
        ASTNode::Frame(frame) => execute_frame(frame),
        ASTNode::Error(error) => execute_error(error),
        // _ => {},
        _ => todo!("Unhandled"),
    }
}

fn execute_error(error: ast::ErrorNode) {
    match error {
        ErrorNode::ParseError(e) => console_error!("ParserError:", e),
    }
}

fn execute_frame(frame: ast::FrameNode) {
    match frame {
        ast::FrameNode::Select { name } => {
            console_warn!("FrameNode::Select: not yet implemented")
        }
        ast::FrameNode::Set {
            name,
            x,
            y,
            yaw,
            parent,
        } => {
            console_warn!("FrameNode::Set: not yet implemented")
        }
    }
}
