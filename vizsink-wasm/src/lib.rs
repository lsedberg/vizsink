use std::cell::RefCell;
use std::rc::Rc;

use vizsink_core::ast::Primitive;
use vizsink_core::generator;
use vizsink_core::generator::Command;
use vizsink_core::generator::Point2D;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::Event;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MessageEvent, WebSocket};

use vizsink_core::parser;

mod utils;

type AppHandle = Rc<RefCell<AppState>>;

thread_local! {
    // global app state accessible from anywhere in this module
    static APP_STATE: RefCell<Option<AppHandle>> = RefCell::new(None);
}

struct AppState {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    camera: Camera,
}

fn set_app_state(handle: AppHandle) {
    APP_STATE.with(|s| *s.borrow_mut() = Some(handle));
}

fn with_app_state<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&AppHandle) -> R,
{
    APP_STATE.with(|s| s.borrow().as_ref().map(|h| f(h)))
}

struct Camera {
    /// Camera x-coordinate in world units
    x: f64,
    /// Camera y-coordinate in world units
    y: f64,
    /// Scale is in pixels per natural unit.
    scale: f64,
}

impl Camera {
    /// Convert a point in world/model coordinates to canvas pixel coordinates
    ///
    /// World +Y is mapped to screen up, and therefor flipped as Y+ in screen coordinates are downwards.
    fn world_to_pixel(&self, world_point: Point2D, canvas_w: u32, canvas_h: u32) -> Point2D {
        let px = (world_point.x - self.x) * self.scale + canvas_w as f64 / 2.0;
        let py = canvas_h as f64 / 2.0 - (world_point.y - self.y) * self.scale;
        Point2D { x: px, y: py }
    }

    /// Convert a canvas pixel coordinate into world/model coordinates.
    ///
    /// Inverse of world_to_pixel.
    fn world_to_point(&self, pixel_point: Point2D, canvas_w: u32, canvas_h: u32) -> Point2D {
        let wx = (pixel_point.x - canvas_w as f64 / 2.0) / self.scale + self.x;
        let wy = (canvas_h as f64 / 2.0 - pixel_point.y) / self.scale + self.y;
        Point2D { x: wx, y: wy }
    }

    /// Convert a length in world/model units into screen pixels.
    fn world_length_to_pixels(&self, world_length: f64) -> f64 {
        world_length * self.scale
    }

    /// Convert a length in screen pixels back into world/model units.
    fn pixels_to_world_length(&self, pixel_length: f64) -> f64 {
        pixel_length / self.scale
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    let window = web_sys::window().expect("should have a window in this context");
    let document = window
        .document()
        .expect("should have a document in this window");
    let status_el = document
        .get_element_by_id("connection-status")
        .expect("no status element");

    // Canvas initialization
    let canvas = document
        .get_element_by_id("canvas-main")
        .expect("no canvas element")
        .dyn_into::<HtmlCanvasElement>()
        .expect("canvas should be HtmlCanvasElement");
    let ctx = canvas
        .get_context("2d")
        .expect("2d context request should not fail")
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .expect("context should be CanvasRenderingContext2d");

    let app_state = Rc::new(RefCell::new(AppState {
        canvas,
        ctx,
        camera: Camera {
            x: 0.0,
            y: 0.0,
            scale: 10.0,
        },
    }));
    APP_STATE.with(|s| *s.borrow_mut() = Some(app_state.clone()));

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
            let commands = generator::generate_commands(parsed);

            console_log!("Received (", commands.len(), " commands): ", &txt);
            execute_commands(commands);
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

fn execute_commands(commands: Vec<Command>) {
    with_app_state(|app_rc| {
        let app = app_rc.borrow();
        let canvas_w = app.canvas.width();
        let canvas_h = app.canvas.height();
        for command in commands {
            match command {
                Command::Stroke => app.ctx.stroke(),
                Command::LineWidth(width) => {
                    let px = app.camera.world_length_to_pixels(width);
                    app.ctx.set_line_width(px);
                }
                Command::StrokeStyle(style) => app.ctx.set_stroke_style_str(style.as_str()),
                Command::BeginPath => app.ctx.begin_path(),
                Command::ClosePath => app.ctx.close_path(),
                Command::MoveTo(point2d) => {
                    let point_px = app.camera.world_to_pixel(point2d, canvas_w, canvas_h);
                    app.ctx.move_to(point_px.x, point_px.y);
                }
                Command::LineTo(point2d) => {
                    let point_px = app.camera.world_to_pixel(point2d, canvas_w, canvas_h);
                    console_log!(point_px.x, point_px.y);
                    app.ctx.line_to(point_px.x, point_px.y);
                }
                Command::ConsoleLog(msg) => console_log!(msg),
                Command::ConsoleError(msg) => console_error!(msg),
                Command::ConsoleWarn(msg) => console_warn!(msg),
            }
        }
    });
}
