#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    Frame(FrameNode),
    Canvas(CanvasNode),
    Layer(LayerNode),
    Shape(ShapeNode),
    Entity(EntityNode),
    Draw(DrawNode),
    Error(ErrorNode),
    Nop,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FrameNode {
    Set {
        name: String,
        x: f32,
        y: f32,
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
        x: f32,
        y: f32,
        angle: Option<f32>,
    },
    Move {
        name: String,
        x: f32,
        y: f32,
        angle: Option<f32>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrawNode {
    Primitive(Primitive),
    Shape(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: Option<String>,
        thickness: Option<f32>,
    },
    Circle {
        x: f32,
        y: f32,
        r: f32,
        color: Option<String>,
        thickness: Option<f32>,
    },
    Rectangle {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Option<String>,
        thickness: Option<f32>,
    },
    Polygon {
        points: Vec<(f32, f32)>,
        color: Option<String>,
        thickness: Option<f32>,
    },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Angle {
    radians: f32,
}

impl Angle {
    pub fn radians(&self) -> f32 {
        self.radians
    }
    pub fn degrees(&self) -> f32 {
        self.radians.to_degrees()
    }
}

impl std::str::FromStr for Angle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Some(num) = s.strip_suffix("deg") {
            let v: f32 = num
                .trim()
                .parse()
                .map_err(|e: std::num::ParseFloatError| e.to_string())?;
            Ok(Angle {
                radians: v.to_radians(),
            })
        } else if let Some(num) = s.strip_suffix("rad") {
            let v: f32 = num
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
