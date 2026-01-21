//! Canvas Command Generator
//! Transforms the AST into simple commands close to the native canvas commands.

use crate::ast::{self, ASTNode};

pub type Number = f64;

const DEFAULT_STROKE_COLOR: &str = "black";
const DEFAULT_STROKE_WIDTH: Number = 0.10; // in natural units

#[derive(Debug, PartialEq)]
pub enum Command {
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

#[derive(Debug, PartialEq)]
pub struct Point2D {
    pub x: Number,
    pub y: Number,
}

pub struct Frame {
    name: String,
    x: Number,
    y: Number,
    yaw: Number,
}

impl Frame {
    fn to_world(&self, frame_point: Point2D) -> Point2D {
        let (s, c) = self.yaw.sin_cos();
        Point2D {
            x: c * frame_point.x - s * frame_point.y + self.x,
            y: s * frame_point.x + c * frame_point.y + self.y,
        }
    }
    fn from_world(&self, world_point: Point2D) -> Point2D {
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

pub fn generate_commands(ast: Vec<ASTNode>) -> Vec<Command> {
    let mut commands = vec![];

    let frame_main: Frame = Frame {
        name: "main".to_string(),
        x: 0.0,
        y: 0.0,
        yaw: 0.0,
    };

    let current_frame = frame_main;

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
                        color,
                        thickness,
                    } => {
                        let line_start_world = current_frame.to_world(Point2D { x: x1, y: y1 });
                        let line_end_world = current_frame.to_world(Point2D { x: x2, y: y2 });
                        let color = match color {
                            Some(value) => value,
                            None => DEFAULT_STROKE_COLOR.to_string(),
                        };
                        let thickness = match thickness {
                            Some(value) => value,
                            None => DEFAULT_STROKE_WIDTH,
                        };

                        commands.append(&mut vec![
                            Command::BeginPath,
                            Command::StrokeStyle(color),
                            Command::LineWidth(thickness),
                            Command::MoveTo(line_start_world),
                            Command::LineTo(line_end_world),
                            Command::Stroke,
                        ]);
                    }
                    ast::Primitive::Circle {
                        x,
                        y,
                        r,
                        color,
                        thickness,
                    } => todo!(),
                    ast::Primitive::Rectangle {
                        x,
                        y,
                        w,
                        h,
                        color,
                        thickness,
                    } => todo!(),
                    ast::Primitive::Polygon {
                        points,
                        color,
                        thickness,
                    } => todo!(),
                },
                ast::DrawNode::Shape(_) => todo!(),
            },
            ASTNode::Error(error_node) => {
                commands.push(Command::ConsoleError(format!("{:?}", error_node)))
            }
            ASTNode::Nop => todo!(),
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
            color: Some("red".to_string()),
            thickness: Some(4.0),
        },
    ))];

    let commands = generate_commands(ast);

    assert_eq!(
        commands,
        vec![
            Command::BeginPath,
            Command::StrokeStyle("red".to_string()),
            Command::LineWidth(4.0),
            Command::MoveTo(Point2D { x: 1.0, y: 2.0 }),
            Command::LineTo(Point2D { x: 3.0, y: 4.0 }),
            Command::Stroke,
        ]
    );
}
