//! Tokens

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Equal,
}

/// Takes a raw string and splits it into a vector of [`Token`]s.
///
/// These tokens are an intermediate step, before being further passed into the parser.
pub fn tokenizer(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut characters = line.chars().peekable();

    while let Some(c) = characters.next() {
        match c {
            '=' => tokens.push(Token::Equal),
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            '[' => tokens.push(Token::LBracket),
            ']' => tokens.push(Token::RBracket),
            ',' => tokens.push(Token::Comma),
            _ => {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    let mut word = String::new();
                    word.push(c);
                    while let Some(c) = characters.next_if(|c| c.is_alphanumeric() || c == &'_') {
                        word.push(c);
                    }
                    tokens.push(Token::Word(word))
                }
            }
        }
    }

    tokens
}

#[test]
fn test_tokenizer() {
    let line = "entity uav shape=my_drone x =0 y= 0 angle = 0deg";
    let tokens = tokenizer(line);
    assert_eq!(
        tokens,
        vec![
            Token::Word("entity".to_string()),
            Token::Word("uav".to_string()),
            Token::Word("shape".to_string()),
            Token::Equal,
            Token::Word("my_drone".to_string()),
            Token::Word("x".to_string()),
            Token::Equal,
            Token::Word("0".to_string()),
            Token::Word("y".to_string()),
            Token::Equal,
            Token::Word("0".to_string()),
            Token::Word("angle".to_string()),
            Token::Equal,
            Token::Word("0deg".to_string()),
        ]
    );

    let line = "draw poly points=[(45, 2), (23,4), ( 4   ,   5)]";
    let tokens = tokenizer(line);
    assert_eq!(
        tokens,
        vec![
            Token::Word("draw".to_string()),
            Token::Word("poly".to_string()),
            Token::Word("points".to_string()),
            Token::Equal,
            Token::LBracket,
            Token::LParen,
            Token::Word("45".to_string()),
            Token::Comma,
            Token::Word("2".to_string()),
            Token::RParen,
            Token::Comma,
            Token::LParen,
            Token::Word("23".to_string()),
            Token::Comma,
            Token::Word("4".to_string()),
            Token::RParen,
            Token::Comma,
            Token::LParen,
            Token::Word("4".to_string()),
            Token::Comma,
            Token::Word("5".to_string()),
            Token::RParen,
            Token::RBracket,
        ]
    );
}
