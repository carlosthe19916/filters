use std::{char, default, io::Read};

pub const COLON: char = ':';
pub const COMMA: char = ',';
pub const AND: char = COMMA;
pub const OR: char = '|';
pub const EQ: char = '=';
pub const LIKE: char = '~';
pub const NOT: char = '!';
pub const LT: char = '<';
pub const GT: char = '>';
pub const QUOTE: char = '"';
pub const SQUOTE: char = '\'';
pub const ESCAPE: char = '\\';
pub const SPACE: char = ' ';
pub const LPAREN: char = '(';
pub const RPAREN: char = ')';

pub struct Lexer {
    pub tokens: Vec<Token>,
}

impl Lexer {
    pub fn with(filter: String) -> Result<Vec<Token>, String> {
        let mut tokens: Vec<Token> = vec![];

        let mut reader = Reader::from(&filter);
        let mut bfr: Vec<char> = vec![];

        let push = |bfr: &mut Vec<char>, tokens: &mut Vec<Token>, kind: Kind| {
            if !bfr.is_empty() {
                let val: String = bfr.iter().collect();
                tokens.push(Token { kind, value: val });
                bfr.clear();
            }
        };

        while let Some(ch) = reader.next() {
            match ch {
                QUOTE | SQUOTE => {
                    reader.put();
                    push(&mut bfr, &mut tokens, Kind::Literal);
                    let quoted = reader.read_quoted()?;

                    tokens.push(quoted);
                }
                SPACE => push(&mut bfr, &mut tokens, Kind::Literal),
                LPAREN => {
                    bfr.push(ch);
                    push(&mut bfr, &mut tokens, Kind::Lparen);
                }
                RPAREN => {
                    push(&mut bfr, &mut tokens, Kind::Literal);
                    bfr.push(ch);
                    push(&mut bfr, &mut tokens, Kind::Rparen);
                }
                COLON | COMMA | OR | EQ | LIKE | NOT | LT | GT => {
                    reader.put();
                    push(&mut bfr, &mut tokens, Kind::Literal);
                    let operator = reader.read_operator()?;

                    tokens.push(operator);
                }
                _ => bfr.push(ch),
            }
        }

        push(&mut bfr, &mut tokens, Kind::Literal);
        Ok(tokens)
    }
}

pub enum Kind {
    Literal,
    String,
    Operator,
    Lparen,
    Rparen,
}

pub struct Token {
    kind: Kind,
    value: String,
}

//

pub struct Reader {
    chars: Vec<char>,
    index: usize,
}

impl Reader {
    pub fn from(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        Self { chars, index: 0 }
    }

    // Next character.
    pub fn next(&mut self) -> Option<char> {
        if self.index < self.chars.len() {
            self.index += 1;
        }
        self.chars.get(self.index).cloned()
    }

    // Put rewinds one character.
    pub fn put(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }

    // Read token
    pub fn read_quoted(&mut self) -> Result<Token, String> {
        let mut last_ch: Option<char> = None;
        let mut bfr: String = String::new();
        let quote = self
            .next()
            .ok_or("Could not get first quoted character".to_string())?;

        loop {
            match self.next() {
                Some(ch) => {
                    if ch == quote {
                        if let Some(last_ch) = last_ch {
                            if last_ch != ESCAPE {
                                break Ok(Token {
                                    kind: Kind::String,
                                    value: bfr.clone(),
                                });
                            }
                        }
                    } else {
                        bfr.push(ch);
                    }
                    last_ch = Some(ch);
                }
                None => break Err(format!("End {} not found.", quote)),
            }
        }
    }

    // Read token
    pub fn read_operator(&mut self) -> Result<Token, String> {
        let mut bfr: String = String::new();
        loop {
            match self.next() {
                Some(ch) => match ch {
                    COLON | COMMA | OR | EQ | LIKE | NOT | LT | GT => bfr.push(ch),
                    _ => {
                        self.put();
                        break Ok(Token {
                            kind: Kind::Operator,
                            value: bfr,
                        });
                    }
                },
                None => break Err("End of operator not found".to_string()),
            }
        }
    }
}
