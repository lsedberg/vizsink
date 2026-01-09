use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: Option<u8>,
}

impl Color {
    pub fn from_hex(mut hex: String) -> Option<Self> {
        // Remove #
        if hex.starts_with("#") {
            hex.remove(0);
        }

        if hex.len() != 6 && hex.len() != 8 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

        let mut a = None;

        if hex.len() == 8 {
            a = Some(u8::from_str_radix(&hex[6..8], 16).unwrap_or(0));
        }

        Some(Color { r, g, b, a })
    }
}

#[test]
fn color_from_str_test() {
    // Test hex -> rgb
    assert!(
        Color::from_hex("#44ba39".to_string()).unwrap()
            == Color {
                r: 68,
                g: 186,
                b: 57,
                a: None
            }
    );

    // Test hex -> rgb (no `#`)
    assert!(
        Color::from_hex("44ba39".to_string()).unwrap()
            == Color {
                r: 68,
                g: 186,
                b: 57,
                a: None
            }
    );

    // Test hex -> rgba
    assert!(
        Color::from_hex("44ba395a".to_string()).unwrap()
            == Color {
                r: 68,
                g: 186,
                b: 57,
                a: Some(90)
            }
    );

    // Test non-valid hex
    assert!(Color::from_hex("egf".to_string()) == None);

    // Test non-valid hex component
    assert!(
        Color::from_hex("44bx39".to_string()).unwrap()
            == Color {
                r: 68,
                g: 0,
                b: 57,
                a: None
            }
    );
}
