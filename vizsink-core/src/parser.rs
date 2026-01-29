use std::{collections::HashMap, str::FromStr};

use crate::{
    ast::{self, ASTNode, Angle, CanvasNode, ErrorNode, Float, FrameNode, GridNode, LayerNode},
    generator::Point2D,
    tokens::{Token, tokenizer},
};

type Params = HashMap<String, String>;

pub struct ParserContext {
    grids: HashMap<String, Grid>,
}

impl ParserContext {
    pub fn new() -> Self {
        ParserContext {
            grids: HashMap::new(),
        }
    }
}

pub struct Grid {
    name: String,
    position: Point2D,
    cell_size: Float,
}

pub fn parse_lines(raw: &str, ctx: &mut ParserContext) -> Vec<ASTNode> {
    let lines = raw.lines();
    let mut tokens = vec![];

    // while let Some(line) = lines.next() {
    for line in lines {
        let mut line_tokens = parse_line(line, ctx);
        tokens.append(&mut line_tokens)
    }

    tokens
}

pub fn parse_line(line: &str, ctx: &mut ParserContext) -> Vec<ASTNode> {
    let mut tokens = tokenizer(line).into_iter();
    let command = tokens.next();
    let tokens = tokens.collect();
    let mut parsed_line = match command {
        Some(Token::Word(s)) => match s.as_str() {
            "frame" => parse_frame(tokens),
            "canvas" => parse_canvas(tokens),
            "layer" => parse_layer(tokens),
            "shape" => parse_shape(tokens),
            "entity" => parse_entity(tokens),
            "grid" => parse_grid(tokens, ctx),
            "draw" => parse_draw(tokens, ctx),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };

    if parsed_line.is_empty() {
        parsed_line.push(ASTNode::Error(ErrorNode::ParseError(format!(
            "Could not parse: {} (empty)",
            line
        ))));
    }

    parsed_line
}

/// Parse frame commands.
///
/// # Example
///
/// Frame Set
/// ```bash
/// frame set my_frame x=1 y=2 yaw=90deg
/// ```
///
/// Frame Select
/// ```bash
/// frame select my_frame
/// ```
fn parse_frame(tokens: Vec<Token>) -> Vec<ASTNode> {
    let mut nodes = vec![];
    let mut it = tokens.into_iter();

    let (subcommand, name) = match (it.next(), it.next()) {
        (Some(Token::Word(s)), Some(Token::Word(n))) => (s.to_lowercase(), n),
        _ => {
            ast::push_err(&mut nodes, "frame: expected subcommand and name");
            return nodes;
        }
    };

    let params = parse_params(it.collect());

    match subcommand.as_str() {
        "set" => {
            let x = match parse_required::<Float>(&params, "x") {
                Ok(v) => v,
                Err(e) => {
                    ast::push_err(&mut nodes, format!("frame set {}: {}", name, e));
                    return nodes;
                }
            };
            let y = match parse_required::<Float>(&params, "y") {
                Ok(v) => v,
                Err(e) => {
                    ast::push_err(&mut nodes, format!("frame set {}: {}", name, e));
                    return nodes;
                }
            };
            let yaw = match parse_required::<Angle>(&params, "yaw") {
                Ok(v) => v,
                Err(e) => {
                    ast::push_err(&mut nodes, format!("frame set {}: {}", name, e));
                    return nodes;
                }
            };

            let parent = params.get("parent").cloned();
            nodes.push(ASTNode::Frame(FrameNode::Set {
                name,
                x,
                y,
                yaw,
                parent,
            }));
        }
        "select" => {
            nodes.push(ASTNode::Frame(FrameNode::Select { name }));
        }
        other => ast::push_err(&mut nodes, format!("unknown frame subcommand '{}'", other)),
    }

    nodes
}

fn parse_canvas(tokens: Vec<Token>) -> Vec<ASTNode> {
    let mut nodes = vec![];
    let mut it = tokens.into_iter();

    if let Some(Token::Word(name)) = it.next() {
        nodes.push(ASTNode::Canvas(CanvasNode { name }))
    } else {
        ast::push_err(&mut nodes, "canvas: no name")
    }

    nodes
}

fn parse_layer(tokens: Vec<Token>) -> Vec<ASTNode> {
    let mut nodes = vec![];
    let mut it = tokens.into_iter();

    if let Some(Token::Word(name)) = it.next() {
        nodes.push(ASTNode::Layer(LayerNode { name }))
    } else {
        ast::push_err(&mut nodes, "layer: no name");
    }

    nodes
}

fn parse_shape(tokens: Vec<Token>) -> Vec<ASTNode> {
    vec![]
}

fn parse_entity(tokens: Vec<Token>) -> Vec<ASTNode> {
    vec![]
}

fn parse_grid(tokens: Vec<Token>, ctx: &mut ParserContext) -> Vec<ASTNode> {
    let mut nodes = vec![];
    let mut it = tokens.into_iter();

    if let Some(Token::Word(name)) = it.next() {
        let params = parse_params(it.collect());

        let x = match parse_required::<Float>(&params, "x") {
            Ok(v) => v,
            Err(e) => {
                ast::push_err(&mut nodes, format!("grid {}", e));
                return nodes;
            }
        };

        let y = match parse_required::<Float>(&params, "y") {
            Ok(v) => v,
            Err(e) => {
                ast::push_err(&mut nodes, format!("grid {}", e));
                return nodes;
            }
        };

        let w = match parse_required::<u32>(&params, "w") {
            Ok(v) => v,
            Err(e) => {
                ast::push_err(&mut nodes, format!("grid {e}"));
                return nodes;
            }
        };

        let h = match parse_required::<u32>(&params, "h") {
            Ok(v) => v,
            Err(e) => {
                ast::push_err(&mut nodes, format!("grid {e}"));
                return nodes;
            }
        };

        let cell = match parse_required::<Float>(&params, "cell") {
            Ok(v) => v,
            Err(e) => {
                ast::push_err(&mut nodes, format!("grid {e}"));
                return nodes;
            }
        };

        nodes.push(ASTNode::Grid(GridNode {
            name: name.clone(),
            x,
            y,
            w,
            h,
            cell,
        }));

        // Push grid to context

        ctx.grids.insert(
            name.clone(),
            Grid {
                name,
                position: Point2D { x, y },
                cell_size: cell,
            },
        );
    } else {
        ast::push_err(&mut nodes, "grid: no name")
    }

    nodes
}

fn parse_draw(tokens: Vec<Token>, ctx: &mut ParserContext) -> Vec<ASTNode> {
    let mut nodes = vec![];
    let mut it = tokens.into_iter();

    if let Some(Token::Word(id)) = it.next() {
        // Send off the remaining tokens.
        let tokens: Vec<Token> = it.collect();
        let mut draw_commands = match id.as_str() {
            "line" => parse_primitive_line(tokens),
            "circle" => parse_primitive_circle(tokens),
            "rect" => parse_primitive_rectangle(tokens),
            "cell" => parse_primitive_cell(tokens, ctx),
            // "rectangle" => parse_primitive_rectangle(),
            // "polygon" => parse_primitive_polygon(),
            s => parse_draw_shape(s, tokens),
        };

        nodes.append(&mut draw_commands);
    } else {
        ast::push_err(&mut nodes, "draw: missing name of shape or primitive");
    }

    nodes
}

fn parse_primitive_line(tokens: Vec<Token>) -> Vec<ASTNode> {
    let mut nodes = vec![];

    let params = parse_params(tokens);

    let x1 = match parse_required::<Float>(&params, "x1") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("line: {}", e));
            return nodes;
        }
    };

    let y1 = match parse_required::<Float>(&params, "y1") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("line: {}", e));
            return nodes;
        }
    };

    let x2 = match parse_required::<Float>(&params, "x2") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("line: {}", e));
            return nodes;
        }
    };

    let y2 = match parse_required::<Float>(&params, "y2") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("line: {}", e));
            return nodes;
        }
    };

    let stroke_color = match parse_optional::<String>(&params, "stroke_color") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("line: {}", e));
            return nodes;
        }
    };

    let stroke_width = match parse_optional::<Float>(&params, "stroke_width") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("line: {}", e));
            return nodes;
        }
    };

    nodes.push(ASTNode::Draw(ast::DrawNode::Primitive(
        ast::Primitive::Line {
            x1,
            y1,
            x2,
            y2,
            stroke_color,
            stroke_width,
        },
    )));

    nodes
}

fn parse_primitive_circle(tokens: Vec<Token>) -> Vec<ASTNode> {
    let mut nodes = vec![];

    let params = parse_params(tokens);

    let x = match parse_required::<Float>(&params, "x") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("circle: {}", e));
            return nodes;
        }
    };

    let y = match parse_required::<Float>(&params, "y") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("circle: {}", e));
            return nodes;
        }
    };

    let r = match parse_required::<Float>(&params, "r") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("circle: {}", e));
            return nodes;
        }
    };

    let stroke_color = match parse_optional::<String>(&params, "stroke_color") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("circle: {}", e));
            return nodes;
        }
    };

    let stroke_width = match parse_optional::<Float>(&params, "stroke_width") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("circle: {}", e));
            return nodes;
        }
    };

    let fill_color = match parse_optional::<String>(&params, "fill_color") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("circle: {}", e));
            return nodes;
        }
    };

    nodes.push(ASTNode::Draw(ast::DrawNode::Primitive(
        ast::Primitive::Circle {
            x,
            y,
            r,
            stroke_color,
            stroke_width,
            fill_color,
        },
    )));

    nodes
}

fn parse_primitive_rectangle(tokens: Vec<Token>) -> Vec<ASTNode> {
    let mut nodes = vec![];

    let params = parse_params(tokens);

    let x = match parse_required::<Float>(&params, "x") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("rect: {}", e));
            return nodes;
        }
    };

    let y = match parse_required::<Float>(&params, "y") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("rect: {}", e));
            return nodes;
        }
    };

    let w = match parse_required::<Float>(&params, "w") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("rect: {}", e));
            return nodes;
        }
    };

    let h = match parse_required::<Float>(&params, "h") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("rect: {}", e));
            return nodes;
        }
    };

    let stroke_color = match parse_optional::<String>(&params, "stroke_color") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("rect: {}", e));
            return nodes;
        }
    };

    let stroke_width = match parse_optional::<Float>(&params, "stroke_width") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("rect: {}", e));
            return nodes;
        }
    };

    let fill_color = match parse_optional::<String>(&params, "fill_color") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("rect: {}", e));
            return nodes;
        }
    };

    nodes.push(ASTNode::Draw(ast::DrawNode::Primitive(
        ast::Primitive::Rectangle {
            x,
            y,
            w,
            h,
            stroke_color,
            stroke_width,
            fill_color,
        },
    )));

    nodes
}

fn parse_primitive_cell(tokens: Vec<Token>, ctx: &mut ParserContext) -> Vec<ASTNode> {
    let mut nodes = vec![];

    let params = parse_params(tokens);

    let x = match parse_required::<Float>(&params, "x") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("cell: {e}"));
            return nodes;
        }
    };

    let y = match parse_required::<Float>(&params, "y") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("cell: {e}"));
            return nodes;
        }
    };

    let grid_name = match parse_required::<String>(&params, "grid") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("cell: {e}"));
            return nodes;
        }
    };

    let stroke_color = match parse_optional::<String>(&params, "stroke_color") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("cell: {}", e));
            return nodes;
        }
    };

    let stroke_width = match parse_optional::<Float>(&params, "stroke_width") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("cell: {}", e));
            return nodes;
        }
    };

    let fill_color = match parse_optional::<String>(&params, "fill_color") {
        Ok(v) => v,
        Err(e) => {
            ast::push_err(&mut nodes, format!("cell: {}", e));
            return nodes;
        }
    };

    let grid = match ctx.grids.get(&grid_name) {
        Some(v) => v,
        None => {
            ast::push_err(&mut nodes, format!("cell: no grid with name: {grid_name}"));
            return nodes;
        }
    };

    // Overwrite default as we'd usually like fill, but no strokes (unless specified)
    let fill_color = match fill_color {
        Some(color) => color,
        None => "black".to_string(),
    };

    let stroke_width = match stroke_width {
        Some(color) => color,
        None => 0.0,
    };

    let w = grid.cell_size;
    let x_world = grid.position.x + x * w;
    let y_world = grid.position.y + y * w;

    nodes.push(ASTNode::Draw(ast::DrawNode::Primitive(
        ast::Primitive::Rectangle {
            x: x_world,
            y: y_world,
            w,
            h: w,
            stroke_color,
            stroke_width: Some(stroke_width),
            fill_color: Some(fill_color),
        },
    )));

    nodes
}

fn parse_draw_shape(name: &str, tokens: Vec<Token>) -> Vec<ASTNode> {
    // todo!()
    vec![]
}

fn parse_params(tokens: Vec<Token>) -> Params {
    let mut params: Params = HashMap::new();
    let mut tokens = tokens.into_iter().peekable();
    while let Some(Token::Word(token)) = tokens.next() {
        if let (Some(Token::Equal), Some(Token::Word(value))) = (tokens.next(), tokens.next()) {
            params.insert(token, value);
        }
    }

    params
}

// Helpers

/// Fetches an item from the params list and converts it to the given type.
/// Returns error if the item is cannot be converted or does not exist.
fn parse_required<T: FromStr>(params: &Params, key: &str) -> Result<T, String>
where
    <T as FromStr>::Err: ToString,
{
    match params.get(key) {
        Some(s) => s
            .parse::<T>()
            .map_err(|e| format!("invalid value for '{}': {}", key, e.to_string())),
        None => Err(format!("missing parameter '{}'", key)),
    }
}

/// Fetches an item from the params list and converts it to the given type.
/// Returns error if the item is cannot be converted or does not exist.
/// The result is returned as an option.
fn parse_optional<T: FromStr>(params: &Params, key: &str) -> Result<Option<T>, String>
where
    <T as FromStr>::Err: ToString,
{
    match params.get(key) {
        Some(s) => s
            .parse::<T>()
            .map(|v| Some(v))
            .map_err(|e| format!("invalid value for '{}': {}", key, e.to_string())),
        None => Ok(None),
    }
}

// Tests

#[test]
fn test_parse_line() {
    let line = "frame set my_frame x=1 y=2 yaw=90deg";
    let parsed = parse_line(line, &mut ParserContext::new());

    dbg!(parsed);
}

#[test]
fn test_parse_lines() {
    let lines = r#"frame select my_frame
    canvas my_canvas
    grid world x=1 y=2 w=3 h=4 cell=0.1"#;
    let parsed = parse_lines(lines, &mut ParserContext::new());

    assert_eq!(
        parsed,
        vec![
            ASTNode::Frame(FrameNode::Select {
                name: "my_frame".to_string()
            }),
            ASTNode::Canvas(CanvasNode {
                name: "my_canvas".to_string()
            }),
            ASTNode::Grid(GridNode {
                name: "world".to_string(),
                x: 1.0,
                y: 2.0,
                w: 3,
                h: 4,
                cell: 0.1
            })
        ]
    );

    dbg!(parsed);
}

#[test]
fn test_params() {
    let params_raw = "x=4 y= 6.5 z   = -4";
    let tokens = tokenizer(params_raw);
    dbg!(&tokens);
    let params = parse_params(tokens);

    assert_eq!(parse_required::<Float>(&params, "x"), Ok(4.0));
    assert_eq!(parse_required::<Float>(&params, "y"), Ok(6.5));
    assert_eq!(parse_required::<Float>(&params, "z"), Ok(-4.0));
}
