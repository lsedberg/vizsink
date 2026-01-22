use std::cell::RefCell;
use std::rc::Rc;

use vizsink_core::generator;
use vizsink_core::generator::Command;
use vizsink_core::generator::Point2D;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::Document;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, MessageEvent, PointerEvent, WebSocket, WheelEvent,
};

use vizsink_core::parser;

mod utils;

type AppHandle = Rc<RefCell<AppState>>;

thread_local! {
    // global app state accessible from anywhere in this module
    static APP_STATE: RefCell<Option<AppHandle>> = RefCell::new(None);
}

struct AppState {
    document: Document,
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    camera: Camera,
    // Movement
    // TODO:
    // TODO:
    // TODO:
    // TODO:
    // TODO:
    // TODO:
    // TODO:
    // TODO:, should this be in camera?

    // Camera Movement
    is_panning: bool,
    last_pointer: Option<Point2D>,

    // scene / command buffer
    // TODO: separate between draw commands that must be redrawn, and other more persistent commands such as console logs.
    commands: Vec<Command>,
    /// dirty indicates we need to redraw
    dirty: bool,
}

fn init_app_state(
    document: Document,
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
) -> AppHandle {
    let handle = Rc::new(RefCell::new(AppState {
        canvas,
        ctx,
        document,
        camera: Camera {
            x: 0.0,
            y: 0.0,
            scale: 10.0,
        },
        is_panning: false,
        last_pointer: None,
        commands: Vec::new(),
        dirty: true,
    }));
    APP_STATE.with(|s| *s.borrow_mut() = Some(handle.clone()));
    handle
}

fn mark_dirty_and_draw() {
    with_app_state(|h| {
        let mut app = h.borrow_mut();
        app.dirty = true;
        draw_scene_locked(&mut app);
    });
}

fn draw_scene_locked(app: &mut AppState) {
    // clear canvas
    let cw = app.canvas.width() as f64;
    let ch = app.canvas.height() as f64;
    app.ctx.clear_rect(0.0, 0.0, cw, ch);

    // replay all commands
    for command in &app.commands {
        match command {
            Command::Stroke => app.ctx.stroke(),
            Command::LineWidth(width) => {
                let px = app.camera.world_length_to_pixels(*width);
                app.ctx.set_line_width(px);
            }
            Command::StrokeStyle(style) => app.ctx.set_stroke_style_str(style.as_str()),
            Command::BeginPath => app.ctx.begin_path(),
            Command::ClosePath => app.ctx.close_path(),
            Command::MoveTo(point2d) => {
                let point_px = app.camera.world_to_pixel(*point2d, cw as u32, ch as u32);
                app.ctx.move_to(point_px.x, point_px.y);
            }
            Command::LineTo(point2d) => {
                let point_px = app.camera.world_to_pixel(*point2d, cw as u32, ch as u32);
                app.ctx.line_to(point_px.x, point_px.y);
            }
            Command::ConsoleLog(msg) => console_log!(msg),
            Command::ConsoleError(msg) => console_error!(msg),
            Command::ConsoleWarn(msg) => console_warn!(msg),
        }
    }

    app.dirty = false;
}

fn draw_scene() {
    with_app_state(|h| {
        let mut app = h.borrow_mut();
        draw_scene_locked(&mut app);
    });
}

fn append_commands_and_draw(mut new_cmds: Vec<Command>) {
    with_app_state(|h| {
        let mut app = h.borrow_mut();
        app.commands.append(&mut new_cmds);
        app.dirty = true;
        draw_scene_locked(&mut app);
    });
}

fn setup_ws(app_handle: AppHandle, ws_url: String) {
    let ws = WebSocket::new(&ws_url).expect("ws new failed");

    // keep closure alive by forgetting
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        if let Some(txt) = e.data().as_string() {
            let parsed = parser::parse_lines(&txt);
            let commands = generator::generate_commands(parsed);
            append_commands_and_draw(commands);
            console_log!("Received commands: ", txt);
        }
    });

    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    {
        let app_handle = app_handle.clone();
        let onopen = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            let status_el = app_handle
                .borrow_mut()
                .document
                .get_element_by_id("connection-status")
                .expect("status");
            status_el.set_inner_html("Connected");
            console_log!("Connected")
        });
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();
    }

    {
        let app_handle = app_handle.clone();
        let onclose = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            let status_el = app_handle
                .borrow_mut()
                .document
                .get_element_by_id("connection-status")
                .expect("status");
            status_el.set_inner_html("Disconnected");
            console_error!("Connection Lost")
        });
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();
    }
}

fn attach_canvas_handlers(app_handle: AppHandle) {
    let canvas = app_handle.borrow().canvas.clone();

    // pointer down
    {
        let h = app_handle.clone();
        let c = canvas.clone();
        let cb = Closure::<dyn FnMut(PointerEvent)>::new(move |ev: PointerEvent| {
            let _ = c.set_pointer_capture(ev.pointer_id());
            let mut app = h.borrow_mut();
            app.is_panning = true;
            app.last_pointer = Some(Point2D { x: ev.client_x() as f64, y: ev.client_y() as f64});
        });
        canvas.add_event_listener_with_callback("pointerdown", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // pointer move
    {
        let h = app_handle.clone();
        let cb = Closure::<dyn FnMut(PointerEvent)>::new(move |ev: PointerEvent| {
            let mut app = h.borrow_mut();
            if !app.is_panning { return; }
            if let Some(last) = app.last_pointer.take() {
                let dx = ev.client_x() as f64 - last.x;
                let dy = ev.client_y() as f64 - last.y;
                app.camera.x -= dx / app.camera.scale;
                app.camera.y += dy / app.camera.scale;
                app.last_pointer = Some(Point2D { x: ev.client_x() as f64, y: ev.client_y() as f64});
                app.dirty = true;
                // redraw immediately
                draw_scene_locked(&mut app);
            }
        });
        canvas.add_event_listener_with_callback("pointermove", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // pointer up / cancel
    {
        let h = app_handle.clone();
        let c = canvas.clone();
        let cb = Closure::<dyn FnMut(PointerEvent)>::new(move |ev: PointerEvent| {
            let _ = c.release_pointer_capture(ev.pointer_id());
            let mut app = h.borrow_mut();
            app.is_panning = false;
            app.last_pointer = None;
        });
        canvas.add_event_listener_with_callback("pointerup", cb.as_ref().unchecked_ref()).unwrap();
        canvas.add_event_listener_with_callback("pointercancel", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // wheel, zoom on cursor
    {
        let handle = app_handle.clone();
        let c = canvas.clone();
        let cb = Closure::<dyn FnMut(WheelEvent)>::new(move |ev: WheelEvent| {
            ev.prevent_default();
            let rect = c.get_bounding_client_rect();
            let px = ev.client_x() as f64 - rect.left();
            let py = ev.client_y() as f64 - rect.top();
            let cw = c.width() as f64;
            let ch = c.height() as f64;

            let mut app = handle.borrow_mut();
            let scale_old = app.camera.scale;
            let zoom_in = ev.delta_y() < 0.0;
            let factor = if zoom_in { 1.1 } else { 0.9 };

            // Todo move constants to global const
            let scale_new = (scale_old * factor).clamp(0.01, 1e6);

            // keep world point under cursor stationary
            let inv_old = 1.0 / scale_old;
            let inv_new = 1.0 / scale_new;
            app.camera.x += (px - cw / 2.0) * (inv_old - inv_new);
            app.camera.y += (ch / 2.0 - py) * (inv_old - inv_new);
            app.camera.scale = scale_new;

            app.dirty = true;
            draw_scene_locked(&mut app);
        });
        canvas
            .add_event_listener_with_callback("wheel", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();
    }
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

    /// Reset camera position and zoom/scale.
    fn reset(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.scale = 10.0;
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

    let app = init_app_state(document.clone(), canvas.clone(), ctx);
    attach_canvas_handlers(app.clone());

    let location = window.location();
    let ws_url = format!(
        "ws://{}/ws",
        &location.host().expect("window should have a host location")
    );
    // let ws = WebSocket::new(&ws_url).unwrap();
    setup_ws(app.clone(), ws_url);

    // initial draw
    draw_scene();

    console_log!("VizSink started!");
}
