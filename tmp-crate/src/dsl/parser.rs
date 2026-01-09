use std::collections::HashMap;

use crate::dsl::commands::Op;
use crate::utils::color;

#[derive(Clone, Default)]
pub struct ParserContext {
    colors: HashMap<String, color::Color>,
}

pub fn parse_line(line: String, context: &mut ParserContext) -> Vec<Op> {
    let mut ops = Vec::<Op>::new();

    let mut args = line.split(" ").peekable();

    match args.next() {
        Some("color") => {
            match (args.next(), args.next()) {
                (Some(name), Some(color_string)) => {
                    if let Some(color) = color::Color::from_hex(color_string.to_string()) {
                        context.colors.insert(name.to_string(), color);
                    } else {
                        return ops; // Early Return
                    }
                }
                _ => return ops,
            }
        }
        Some("line") => {
            match (args.next(), args.next(), args.next(), args.next()) {
                (Some(x1s), Some(y1s), Some(x2s), Some(y2s)) => {
                    if let (Ok(x1), Ok(y1), Ok(x2), Ok(y2)) = (
                        x1s.parse::<f32>(),
                        y1s.parse::<f32>(),
                        x2s.parse::<f32>(),
                        y2s.parse::<f32>(),
                    ) {
                        ops.push(Op::Line { x1, y1, x2, y2 })
                    } else {
                        return ops; // Parsing fail
                    }
                }
                _ => return ops, // Missing arguments
            }
        }
        _ => {} // Noop
    }

    ops
}
