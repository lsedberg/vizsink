use core::f64;
use std::cell::RefCell;
use std::rc::Rc;

use vizsink_core::generator;
use vizsink_core::generator::Command;
use vizsink_core::generator::EffectCommand;
use vizsink_core::generator::Point2D;
use vizsink_core::generator::RenderCommand;
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
    static APP_STATE: RefCell<Option<AppHandle>> = const { RefCell::new(None) }; //RefCell::new(None);
}

struct AppState {
    document: Document,
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    camera: Camera,

    // Movement

    // Camera Movement
    is_panning: bool,
    last_pointer: Option<Point2D>,

    // scene / command buffer
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

    // draw background
    draw_grid(app);

    // replay all draw commands
    for command in &app.commands {
        match command {
            Command::Render(draw_command) => match draw_command {
                RenderCommand::Stroke => app.ctx.stroke(),
                RenderCommand::LineWidth(width) => {
                    let px = app.camera.world_length_to_pixels(*width);
                    app.ctx.set_line_width(px);
                }
                RenderCommand::StrokeStyle(style) => app.ctx.set_stroke_style_str(style.as_str()),
                RenderCommand::BeginPath => app.ctx.begin_path(),
                RenderCommand::ClosePath => app.ctx.close_path(),
                RenderCommand::MoveTo(point2d) => {
                    let point_px = app.camera.world_to_pixel(*point2d, cw as u32, ch as u32);
                    app.ctx.move_to(point_px.x, point_px.y);
                }
                RenderCommand::LineTo(point2d) => {
                    let point_px = app.camera.world_to_pixel(*point2d, cw as u32, ch as u32);
                    app.ctx.line_to(point_px.x, point_px.y);
                }
                RenderCommand::Save => app.ctx.save(),
                RenderCommand::Restore => app.ctx.restore(),
                RenderCommand::Arc {
                    center,
                    radius,
                    angle_start,
                    angle_end,
                    ccw,
                } => {
                    let center_px = app.camera.world_to_pixel(*center, cw as u32, ch as u32);
                    let radius_px = app.camera.world_length_to_pixels(*radius);
                    let _ = app.ctx.arc_with_anticlockwise(
                        center_px.x,
                        center_px.y,
                        radius_px,
                        *angle_start,
                        *angle_end,
                        *ccw,
                    );
                }
                RenderCommand::FillColor(fill_color) => app.ctx.set_fill_style_str(fill_color),
                RenderCommand::Fill => app.ctx.fill(),
            },
            Command::Effect(_) => {}
        }
    }

    app.dirty = false;
}

fn execute_commands(commands: Vec<Command>) {
    for command in commands {
        match command {
            Command::Render(_) => {}
            Command::Effect(other_command) => match other_command {
                EffectCommand::ConsoleLog(msg) => console_log!(msg),
                EffectCommand::ConsoleError(msg) => console_error!(msg),
                EffectCommand::ConsoleWarn(msg) => console_warn!(msg),
            },
        }
    }
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
            execute_commands(commands.clone());
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
            app.last_pointer = Some(Point2D {
                x: ev.client_x() as f64,
                y: ev.client_y() as f64,
            });
        });
        canvas
            .add_event_listener_with_callback("pointerdown", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();
    }

    // pointer move
    {
        let h = app_handle.clone();
        let cb = Closure::<dyn FnMut(PointerEvent)>::new(move |ev: PointerEvent| {
            let mut app = h.borrow_mut();
            if !app.is_panning {
                return;
            }
            if let Some(last) = app.last_pointer.take() {
                let dx = ev.client_x() as f64 - last.x;
                let dy = ev.client_y() as f64 - last.y;
                app.camera.x -= dx / app.camera.scale;
                app.camera.y += dy / app.camera.scale;
                app.last_pointer = Some(Point2D {
                    x: ev.client_x() as f64,
                    y: ev.client_y() as f64,
                });
                app.dirty = true;
                // redraw immediately
                draw_scene_locked(&mut app);
            }
        });
        canvas
            .add_event_listener_with_callback("pointermove", cb.as_ref().unchecked_ref())
            .unwrap();
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
        canvas
            .add_event_listener_with_callback("pointerup", cb.as_ref().unchecked_ref())
            .unwrap();
        canvas
            .add_event_listener_with_callback("pointercancel", cb.as_ref().unchecked_ref())
            .unwrap();
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
    APP_STATE.with(|s| s.borrow().as_ref().map(f))
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

/// Picks a world-space grid spacing and how many minor steps per major.
fn grid_spacing_for_scale(scale: f64) -> (f64, usize) {
    let target_px = 50.0_f64;
    let mut desired_world = target_px / scale;
    if desired_world <= 0.0 {
        desired_world = 1.0;
    }

    // candidate bases [1,2,5] * 10^exp
    let exp = desired_world.abs().log10().floor() as i32;
    let mut best = (10f64.powi(exp), 5usize);
    let bases = [1.0];
    let mut best_diff = f64::INFINITY;

    for b in &bases {
        let cand = b * 10f64.powi(exp);
        let diff = (cand - desired_world).abs();
        if diff < best_diff {
            best_diff = diff;
            best = (cand, 10);
        }
    }

    for delta in [-1, 1] {
        let e = exp + delta;
        for b in &bases {
            let cand = b * 10f64.powi(e);
            let diff = (cand - desired_world).abs();
            if diff < best_diff {
                best_diff = diff;
                best = (cand, 10);
            }
        }
    }

    // minor -> return spacing, and number of minors per major (use 5)
    (best.0, 10)
}

/// Draw an "infinite" grid by drawing lines across the visible world bounds.
/// Minor and major lines are drawn; axis (x=0, y=0
fn draw_grid(app: &mut AppState) {
    let cw = app.canvas.width() as f64;
    let ch = app.canvas.height() as f64;

    let top_left = app
        .camera
        .world_to_point(Point2D { x: 0.0, y: 0.0 }, cw as u32, ch as u32);
    let bottom_right = app
        .camera
        .world_to_point(Point2D { x: cw, y: ch }, cw as u32, ch as u32);

    let min_x = top_left.x.min(bottom_right.x);
    let max_x = top_left.x.max(bottom_right.x);
    let min_y = top_left.y.min(bottom_right.y);
    let max_y = top_left.y.max(bottom_right.y);

    let (spacing, major_every) = grid_spacing_for_scale(app.camera.scale);
    if spacing <= 0.0 {
        return;
    }

    // draw minor lines
    app.ctx.save();
    app.ctx.begin_path();
    app.ctx.set_stroke_style_str("#e6e6e6");
    app.ctx.set_line_width(1.0);

    let start_ix = (min_x / spacing).floor() as i64;
    let end_ix = (max_x / spacing).ceil() as i64;

    for i in start_ix..=end_ix {
        let x = i as f64 * spacing;
        let p1 = app
            .camera
            .world_to_pixel(Point2D { x, y: min_y }, cw as u32, ch as u32);
        let p2 = app
            .camera
            .world_to_pixel(Point2D { x, y: max_y }, cw as u32, ch as u32);
        app.ctx.move_to(p1.x, p1.y);
        app.ctx.line_to(p2.x, p2.y);
    }

    let start_jy = (min_y / spacing).floor() as i64;
    let end_jy = (max_y / spacing).ceil() as i64;
    for j in start_jy..=end_jy {
        let y = j as f64 * spacing;
        let p1 = app
            .camera
            .world_to_pixel(Point2D { x: min_x, y }, cw as u32, ch as u32);
        let p2 = app
            .camera
            .world_to_pixel(Point2D { x: max_x, y }, cw as u32, ch as u32);
        app.ctx.move_to(p1.x, p1.y);
        app.ctx.line_to(p2.x, p2.y);
    }

    app.ctx.stroke();
    app.ctx.close_path();
    app.ctx.restore();

    // draw major lines
    app.ctx.save();
    app.ctx.begin_path();
    app.ctx.set_stroke_style_str("#cfcfcf");
    app.ctx.set_line_width(1.5);

    for i in start_ix..=end_ix {
        if (i % (major_every as i64)) == 0 {
            let x = i as f64 * spacing;
            let p1 = app
                .camera
                .world_to_pixel(Point2D { x, y: min_y }, cw as u32, ch as u32);
            let p2 = app
                .camera
                .world_to_pixel(Point2D { x, y: max_y }, cw as u32, ch as u32);
            app.ctx.move_to(p1.x, p1.y);
            app.ctx.line_to(p2.x, p2.y);
        }
    }

    for j in start_jy..=end_jy {
        if (j % (major_every as i64)) == 0 {
            let y = j as f64 * spacing;
            let p1 = app
                .camera
                .world_to_pixel(Point2D { x: min_x, y }, cw as u32, ch as u32);
            let p2 = app
                .camera
                .world_to_pixel(Point2D { x: max_x, y }, cw as u32, ch as u32);
            app.ctx.move_to(p1.x, p1.y);
            app.ctx.line_to(p2.x, p2.y);
        }
    }

    app.ctx.stroke();
    app.ctx.close_path();
    app.ctx.restore();

    // draw x and y axes
    app.ctx.save();
    app.ctx.begin_path();
    app.ctx.set_line_width(1.0);

    // x axis
    app.ctx.set_stroke_style_str("#ff0000");
    if min_y <= 0.0 && max_y >= 0.0 {
        let p1 = app
            .camera
            .world_to_pixel(Point2D { x: min_x, y: 0.0 }, cw as u32, ch as u32);
        let p2 = app
            .camera
            .world_to_pixel(Point2D { x: max_x, y: 0.0 }, cw as u32, ch as u32);
        app.ctx.move_to(p1.x, p1.y);
        app.ctx.line_to(p2.x, p2.y);
    }

    app.ctx.stroke();
    app.ctx.close_path();

    app.ctx.begin_path();

    // y axis
    app.ctx.set_stroke_style_str("#00ff00");
    if min_x <= 0.0 && max_x >= 0.0 {
        let p1 = app
            .camera
            .world_to_pixel(Point2D { x: 0.0, y: min_y }, cw as u32, ch as u32);
        let p2 = app
            .camera
            .world_to_pixel(Point2D { x: 0.0, y: max_y }, cw as u32, ch as u32);
        app.ctx.move_to(p1.x, p1.y);
        app.ctx.line_to(p2.x, p2.y);
    }

    app.ctx.stroke();
    app.ctx.close_path();

    app.ctx.restore();
}

#[wasm_bindgen(start)]
pub fn start() {
    let window = web_sys::window().expect("should have a window in this context");
    let document = window
        .document()
        .expect("should have a document in this window");

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
