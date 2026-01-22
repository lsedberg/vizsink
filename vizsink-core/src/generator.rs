//! Canvas Command Generator
//! Transforms the AST into simple commands close to the native canvas commands.

use crate::ast::{self, ASTNode};

pub type Float = f64;

const DEFAULT_STROKE_COLOR: &str = "black";
const DEFAULT_STROKE_WIDTH: Float = 0.10; // in natural units

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    /// Render commands are rerun in order every time the scene rerenders.
    Render(RenderCommand),

    /// Effects are triggered once upon receiving the command.
    Effect(EffectCommand),
}

impl From<RenderCommand> for Command {
    fn from(render_command: RenderCommand) -> Self {
        Command::Render(render_command)
    }
}

impl From<EffectCommand> for Command {
    fn from(effect_command: EffectCommand) -> Self {
        Command::Effect(effect_command)
    }
}

/// Commands that rerun on redraw.
#[derive(Debug, PartialEq, Clone)]
pub enum RenderCommand {
    /// Draws and arc centered at (x, y) with a radius. The path starts at start_angle, ends at end_angle.
    /// Travels counter clockwise if counter_clockwise is true, else clockwise.
    ///
    /// Calls `ctx.arc(x, y, radius, startAngle, endAngle, counterclockwise)`
    ///
    /// [MDN reference](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/arc)
    Arc {
        center: Point2D,
        radius: f64,
        angle_start: f64,
        angle_end: f64,
        ccw: bool,
    },

    /// Initiates a stroke.
    ///
    /// Calls `ctx.stroke()`
    ///
    /// [MDN reference](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/stroke)
    Stroke,

    /// Sets the width of the stroke in model units.
    ///
    /// First transforms the model units into pixels, then uses `ctx.lineWidth`.
    ///
    /// [MDN reference](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/lineWidth)
    LineWidth(f64),

    /// Sets the color of the stroke.
    ///
    /// Uses `ctx.strokeStyle`
    ///
    /// [MDN reference](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/strokeStyle)
    StrokeStyle(String),

    /// Starts a path
    ///
    /// Calls `ctx.beginPath()`
    ///
    /// [MDN reference](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/beginPath)
    BeginPath,

    /// Closes a path
    ///
    /// Calls `ctx.closePath()`
    ///
    /// [MDN reference](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/closePath)
    ClosePath,

    /// Sets the color of fill.
    ///
    /// Uses `ctx.fillStyle`
    ///
    /// [MDN reference](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/fillStyle)
    FillColor(String),

    /// Fills the current path.
    ///
    /// Uses `ctx.fill`
    ///
    /// [MDN reference](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/fill)
    Fill,

    /// Move the cursor to model coordinates.
    ///
    /// Transforms the coordinates to pixel coordinates,
    /// then calls `ctx.moveTo()`.
    ///
    /// [MDN reference](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/moveTo)
    MoveTo(Point2D),

    /// Draws a line from the cursor (or the previous line end) to the new model coordinates.
    ///
    /// First transforms the coordinates to pixel coordinates,
    /// then calls `ctx.lineTo()`.
    ///
    /// [MDN reference](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/lineTo)
    LineTo(Point2D),

    /// Saves the state of the canvas by pushing the current state onto a stack.
    /// Used together with [`RenderCommand::Restore`]
    ///
    /// Calls `ctx.save()`
    ///
    /// [MDN reference](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/save)
    Save,

    /// Restores the most recently saved canvas state by popping the top of the state stack.
    /// Used together with [`RenderCommand::Save`]
    ///
    /// Calls `ctx.restore()`
    ///
    /// [MDN Reference](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/restore)
    Restore,
}

/// Permanent commands that are only run when received, and *not* on every redraw of the canvas.
#[derive(Debug, PartialEq, Clone)]
pub enum EffectCommand {
    /// Console logs to the browser console.
    ///
    /// Calls `console.log()`
    ConsoleLog(String),

    /// Console logs an error to the browser console.
    ///
    /// Calls `console.error()`
    ConsoleError(String),

    /// Console logs a warning to the browser console.
    ///
    /// Calls `console.warn()`
    ConsoleWarn(String),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Point2D {
    pub x: Float,
    pub y: Float,
}

pub struct Frame {
    name: String,
    x: Float,
    y: Float,
    yaw: Float,
}

impl Default for Frame {
    fn default() -> Self {
        Frame {
            name: "main".into(),
            x: 0.0,
            y: 0.0,
            yaw: 0.0,
        }
    }
}

impl Frame {
    fn frame_to_world(&self, frame_point: Point2D) -> Point2D {
        let (s, c) = self.yaw.sin_cos();
        Point2D {
            x: c * frame_point.x - s * frame_point.y + self.x,
            y: s * frame_point.x + c * frame_point.y + self.y,
        }
    }

    fn world_to_frame(&self, world_point: Point2D) -> Point2D {
        // Undo translation, then rotate by -yaw.
        let dx = world_point.x - self.x;
        let dy = world_point.y - self.y;
        let (s, c) = self.yaw.sin_cos();
        Point2D {
            x: c * dx + s * dy,
            y: -s * dx + c * dy,
        }
    }
}

/// Append a mixed list of RenderCommand and EffectCommand values.
/// Usage:
/// ```rs
/// append_cmds!(commands, [
///     RenderCommand::Save,
///     RenderCommand::BeginPath,
///     EffectCommand:ConsoleLog("hello".into()),
///     RenderCommand::Stroke,
/// ]);
/// ```
macro_rules! append_cmds {
    ($vec:expr, [ $($cmd:expr),* $(,)? ]) => {
        $(
            $vec.push(::std::convert::Into::<Command>::into($cmd));
        )*
    };
}

fn draw_poly_commands(
    points: Vec<Point2D>,
    stroke_color: Option<String>,
    stroke_width: Option<Float>,
    fill_color: Option<String>,
    closed: bool,
) -> Vec<Command> {
    let mut commands = Vec::new();

    let mut iter = points.into_iter();

    let start = match iter.next() {
        Some(p) => p,
        None => return Vec::new(),
    };

    let stroke_color = match stroke_color {
        Some(value) => value,
        None => DEFAULT_STROKE_COLOR.to_string(),
    };
    let stroke_width = match stroke_width {
        Some(value) => value,
        None => DEFAULT_STROKE_WIDTH,
    };

    append_cmds!(
        commands,
        [
            RenderCommand::Save,
            RenderCommand::BeginPath,
            RenderCommand::StrokeStyle(stroke_color),
            RenderCommand::LineWidth(stroke_width),
            RenderCommand::MoveTo(start),
        ]
    );

    for point in iter {
        commands.push(RenderCommand::LineTo(point).into());
    }

    if closed {
        commands.push(RenderCommand::ClosePath.into());
    }

    if let Some(fill_color) = fill_color {
        append_cmds!(
            commands,
            [RenderCommand::FillColor(fill_color), RenderCommand::Fill]
        );
    }

    if stroke_width != 0.0 {
        commands.push(RenderCommand::Stroke.into());
    }

    append_cmds!(commands, [RenderCommand::Restore]);

    commands
}

pub fn generate_commands(ast: Vec<ASTNode>) -> Vec<Command> {
    let mut commands = vec![];

    let frame_main: Frame = Frame {
        name: "main".to_string(),
        x: 0.0,
        y: 0.0,
        yaw: 0.0,
    };

    let current_frame = frame_main;

    // TODO: Add multiple frames.
    // let frames = HashMap::<String, Frame>::new();

    for node in ast {
        match node {
            ASTNode::Frame(frame_node) => todo!(),
            ASTNode::Canvas(canvas_node) => todo!(),
            ASTNode::Layer(layer_node) => todo!(),
            ASTNode::Shape(shape_node) => todo!(),
            ASTNode::Entity(entity_node) => todo!(),
            ASTNode::Draw(draw_node) => match draw_node {
                ast::DrawNode::Primitive(primitive) => match primitive {
                    ast::Primitive::Line {
                        x1,
                        y1,
                        x2,
                        y2,
                        stroke_color,
                        stroke_width,
                    } => {
                        let mut line = draw_poly_commands(
                            vec![Point2D { x: x1, y: y1 }, Point2D { x: x2, y: y2 }],
                            stroke_color,
                            stroke_width,
                            None,
                            false,
                        );
                        commands.append(&mut line);
                    }
                    ast::Primitive::Circle {
                        x,
                        y,
                        r,
                        stroke_color,
                        stroke_width,
                        fill_color,
                    } => {
                        let circle_center = current_frame.frame_to_world(Point2D { x, y });
                        let stroke_color = match stroke_color {
                            Some(value) => value,
                            None => DEFAULT_STROKE_COLOR.to_string(),
                        };
                        let stroke_width = match stroke_width {
                            Some(value) => value,
                            None => DEFAULT_STROKE_WIDTH,
                        };

                        append_cmds!(
                            commands,
                            [
                                RenderCommand::Save,
                                RenderCommand::BeginPath,
                                RenderCommand::StrokeStyle(stroke_color),
                                RenderCommand::LineWidth(stroke_width),
                                RenderCommand::Arc {
                                    center: circle_center,
                                    radius: r,
                                    angle_start: 0.0,
                                    angle_end: std::f64::consts::TAU,
                                    ccw: false,
                                },
                            ]
                        );
                        if let Some(fill_color) = fill_color {
                            append_cmds!(
                                commands,
                                [RenderCommand::FillColor(fill_color), RenderCommand::Fill]
                            );
                        }

                        if stroke_width > 0.0 {
                            commands.push(RenderCommand::Stroke.into());
                        }

                        commands.push(RenderCommand::Restore.into());
                    }
                    ast::Primitive::Rectangle {
                        x,
                        y,
                        w,
                        h,
                        stroke_color,
                        stroke_width,
                        fill_color,
                    } => {
                        let p1 = current_frame.frame_to_world(Point2D { x, y });
                        let p2 = current_frame.frame_to_world(Point2D { x: x + w, y });
                        let p3 = current_frame.frame_to_world(Point2D { x: x + w, y: y + h });
                        let p4 = current_frame.frame_to_world(Point2D { x, y: y + h });

                        let mut line = draw_poly_commands(
                            vec![p1, p2, p3, p4, p1],
                            stroke_color,
                            stroke_width,
                            fill_color,
                            true,
                        );
                        commands.append(&mut line);
                    }
                    ast::Primitive::Polygon {
                        points,
                        stroke_color,
                        stroke_width,
                    } => todo!(),
                },
                ast::DrawNode::Shape(_) => todo!(),
            },
            ASTNode::Error(error_node) => {
                commands.push(EffectCommand::ConsoleError(format!("{:?}", error_node)).into())
            }
            ASTNode::Nop => {}
        }
    }

    commands
}

#[test]
fn test_generation() {
    let ast = vec![ASTNode::Draw(ast::DrawNode::Primitive(
        ast::Primitive::Line {
            x1: 1.0,
            y1: 2.0,
            x2: 3.0,
            y2: 4.0,
            stroke_color: Some("red".to_string()),
            stroke_width: Some(0.4),
        },
    ))];

    let commands = generate_commands(ast);

    assert_eq!(
        commands,
        vec![
            Command::Render(RenderCommand::Save),
            Command::Render(RenderCommand::BeginPath),
            Command::Render(RenderCommand::StrokeStyle("red".to_string())),
            Command::Render(RenderCommand::LineWidth(0.4)),
            Command::Render(RenderCommand::MoveTo(Point2D { x: 1.0, y: 2.0 })),
            Command::Render(RenderCommand::LineTo(Point2D { x: 3.0, y: 4.0 })),
            Command::Render(RenderCommand::Stroke),
            Command::Render(RenderCommand::Restore),
        ]
    );
}

#[test]
fn draw_poly_test() {
    let commands = draw_poly_commands(
        vec![Point2D { x: 1.0, y: 2.0 }, Point2D { x: 3.0, y: 4.0 }],
        Some("orange".into()),
        None,
        None,
        false,
    );
    dbg!(commands);

    // assert_eq!(
    //     commands,
    //     [Command::Render(RenderCommand::Save)]
    // )
}
