use serde::{Deserialize, Serialize};

use crate::utils::color;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Op {
    Grid { width: u32, height: u32 },
    Cell { x: u32, y: u32 },
    Rect { x: f32, y: f32, w: f32, h: f32 },
    Line { x1: f32, y1: f32, x2: f32, y2: f32 },
    Color { color: color::Color },
    Clear,
}
