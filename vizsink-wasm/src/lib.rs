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
    let document = window
        .document()
        .expect("should have a document in this window");
    let status_el = document
        .get_element_by_id("connection-status")
        .expect("no status element");

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

    let onopen = {
        let status_el = status_el.clone();
        Closure::<dyn FnMut(Event)>::new(move |_| {
            status_el.set_inner_html("Connected");
            console_info!("Connected");
        })
    };

    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));

    let onclose = {
        let status_el = status_el.clone();

        Closure::<dyn FnMut(Event)>::new(move |_| {
            status_el.set_inner_html("No Connection");
            console_error!("Connection lost");
        })
    };

    ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));

    // keep alive
    onmessage.forget();
    onopen.forget();
    onclose.forget();
}

fn execute_command(node: ASTNode) {
    match node {
        ASTNode::Frame(frame) => execute_frame(frame),
        ASTNode::Error(error) => execute_error(error),
        _ => console_error!("Unhandled Command"),
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
