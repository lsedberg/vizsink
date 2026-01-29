#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    Frame(FrameNode),
    Canvas(CanvasNode),
    Layer(LayerNode),
    Shape(ShapeNode),
    Entity(EntityNode),
    Draw(DrawNode),
    Grid(GridNode),
    Error(ErrorNode),
    Nop,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FrameNode {
    Set {
        name: String,
        x: Float,
        y: Float,
        yaw: Angle,
        parent: Option<String>,
    },
    Select {
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasNode {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayerNode {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapeNode {
    name: String,
    primitive: Primitive,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntityNode {
    Create {
        name: String,
        shape: String,
        x: Float,
        y: Float,
        angle: Option<Float>,
    },
    Move {
        name: String,
        x: Float,
        y: Float,
        angle: Option<Float>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrawNode {
    Primitive(Primitive),
    Shape(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridNode {
    pub name: String,
    pub x: Float,
    pub y: Float,
    pub w: u32,
    pub h: u32,
    pub cell: Float,
}

pub type Float = f64;

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Line {
        x1: Float,
        y1: Float,
        x2: Float,
        y2: Float,
        stroke_color: Option<String>,
        stroke_width: Option<Float>,
    },
    Circle {
        x: Float,
        y: Float,
        r: Float,
        stroke_color: Option<String>,
        stroke_width: Option<Float>,
        fill_color: Option<String>,
    },
    Rectangle {
        x: Float,
        y: Float,
        w: Float,
        h: Float,
        stroke_color: Option<String>,
        stroke_width: Option<Float>,
        fill_color: Option<String>,
    },
    Polygon {
        points: Vec<(Float, Float)>,
        stroke_color: Option<String>,
        stroke_width: Option<Float>,
    },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Angle {
    radians: Float,
}

impl Angle {
    pub fn radians(&self) -> Float {
        self.radians
    }
    pub fn degrees(&self) -> Float {
        self.radians.to_degrees()
    }
}

impl std::str::FromStr for Angle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Some(num) = s.strip_suffix("deg") {
            let v: Float = num
                .trim()
                .parse()
                .map_err(|e: std::num::ParseFloatError| e.to_string())?;
            Ok(Angle {
                radians: v.to_radians(),
            })
        } else if let Some(num) = s.strip_suffix("rad") {
            let v: Float = num
                .trim()
                .parse()
                .map_err(|e: std::num::ParseFloatError| e.to_string())?;
            Ok(Angle { radians: v })
        } else {
            Err("angle must end with 'deg' or 'rad'".into())
        }
    }
}

impl TryFrom<&str> for Angle {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

// Errors
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorNode {
    ParseError(String),
}

// Error Helper Functions
pub fn err_node<S: Into<String>>(msg: S) -> ASTNode {
    ASTNode::Error(ErrorNode::ParseError(msg.into()))
}
pub fn push_err<S: Into<String>>(nodes: &mut Vec<ASTNode>, msg: S) {
    nodes.push(err_node(msg));
}
