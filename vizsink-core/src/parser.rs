use std::{collections::HashMap, str::FromStr};

use crate::{
    ast::{self, ASTNode, Angle, CanvasNode, ErrorNode, FrameNode, LayerNode},
    tokens::{Token, tokenizer},
};

type Params = HashMap<String, String>;

pub fn parse_lines(raw: &str) -> Vec<ASTNode> {
    let mut lines = raw.lines();
    let mut tokens = vec![];

    while let Some(line) = lines.next() {
        let mut line_tokens = parse_line(line);
        tokens.append(&mut line_tokens)
    }

    tokens
}

pub fn parse_line(line: &str) -> Vec<ASTNode> {
    let mut tokens = tokenizer(line).into_iter();
    let command = tokens.next();
    let tokens = tokens.collect();
    let mut parsed_line = match command {
        Some(Token::Word(s)) => {
            if s == "frame" {
                parse_frame(tokens)
            } else if s == "canvas" {
                parse_canvas(tokens)
            } else if s == "layer" {
                parse_layer(tokens)
            } else if s == "shape" {
                parse_shape(tokens)
            } else if s == "entity" {
                parse_entity(tokens)
            } else if s == "draw" {
                parse_draw(tokens)
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };

    if parsed_line.is_empty() {
        parsed_line.push(ASTNode::Error(ErrorNode::ParseError(format!(
            "Could not parse: {}",
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
            let x = match parse_required::<f32>(&params, "x") {
                Ok(v) => v,
                Err(e) => {
                    ast::push_err(&mut nodes, format!("frame set {}: {}", name, e));
                    return nodes;
                }
            };
            let y = match parse_required::<f32>(&params, "y") {
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
    todo!();
}

fn parse_entity(tokens: Vec<Token>) -> Vec<ASTNode> {
    todo!();
}

fn parse_draw(tokens: Vec<Token>) -> Vec<ASTNode> {
    todo!();
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
    let parsed = parse_line(line);

    dbg!(parsed);
}

#[test]
fn test_parse_lines() {
    let lines = r#"frame select my_frame
    canvas my_canvas"#;
    let parsed = parse_lines(lines);

    assert_eq!(
        parsed,
        vec![
            ASTNode::Frame(FrameNode::Select {
                name: "my_frame".to_string()
            }),
            ASTNode::Canvas(CanvasNode {
                name: "my_canvas".to_string()
            }),
        ]
    );

    dbg!(parsed);
}

#[test]
fn test_params() {
    let params_raw = "x=4 y= 6 z   = -4";
    let tokens = tokenizer(params_raw);
    let params = parse_params(tokens);

    assert_eq!(parse_required(&params, "x"), Ok(4));
    assert_eq!(parse_required(&params, "y"), Ok(6));
    assert_eq!(parse_required(&params, "z"), Ok(-4));
}
