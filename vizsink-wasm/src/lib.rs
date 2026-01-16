use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::Event;
use web_sys::{MessageEvent, WebSocket, console};

#[wasm_bindgen(start)]
pub fn start() {
    let window = web_sys::window().expect("should have a window in this context");

    let location = window.location();
    let ws_url = format!(
        "ws://{}/ws",
        &location.host().expect("window should have a host location")
    );
    let ws = WebSocket::new(&ws_url).unwrap();

    console::log_1(&"Listening!".into());

    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        if let Some(txt) = e.data().as_string() {
            console::log_1(&txt.into());
        }
    });

    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

    let onopen = Closure::<dyn FnMut(Event)>::new(move |_| {
        console::log_1(&"Connected.".into());
    });

    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));

    onmessage.forget(); // keep alive
    onopen.forget(); // keep alive
}
