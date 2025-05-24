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

// Lexer token reader.
#[derive(Debug, PartialEq)]
pub struct Lexer {
    pub tokens: Vec<Token>,
    pub index: usize,
}

impl Lexer {
    // With builds with the specified filter.
    pub fn with(filter: String) -> Result<Self, String> {
        let mut tokens: Vec<Token> = vec![];

        let mut reader = Reader::from(&filter);
        let mut bfr: Vec<char> = vec![];

        let push = |bfr: &mut Vec<char>, tokens: &mut Vec<Token>, kind: Kind| {
            if !bfr.is_empty() {
                tokens.push(Token {
                    kind,
                    value: bfr.clone(),
                });
                bfr.clear();
            }
        };

        while let Some(ch) = reader.next() {
            match ch {
                QUOTE | SQUOTE => {
                    reader.put();
                    push(&mut bfr, &mut tokens, Kind::Literal);
                    let mut quoted = Quoted {
                        reader: &mut reader,
                    };
                    let quoted = quoted.read()?;

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

                    let mut operator = Operator {
                        reader: &mut reader,
                    };
                    let operator = operator.read()?;

                    tokens.push(operator);
                }
                _ => bfr.push(ch),
            }
        }

        push(&mut bfr, &mut tokens, Kind::Literal);
        Ok(Self { tokens, index: 0 })
    }

    // next returns the next token.
    pub fn next(&mut self) -> Option<Token> {
        if self.index < self.tokens.len() {
            let token = self.tokens.get(self.index);
            self.index += 1;

            token.cloned()
        } else {
            None
        }
    }

    // Put rewinds the lexer by 1 token.
    pub fn put(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Kind {
    Literal,
    String,
    Operator,
    Lparen,
    Rparen,
}

// Token scanned token.
#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: Kind,
    pub value: Vec<char>,
}

#[derive(Debug, PartialEq)]
pub enum TokenValue {
    String(String),
    Number(usize),
    Bool(bool),
}

impl Token {
    pub fn as_value(&self) -> TokenValue {
        let v: String = self.value.iter().collect();
        match self.kind {
            Kind::Literal => {
                if let Ok(n) = v.parse::<usize>() {
                    TokenValue::Number(n)
                } else if let Ok(b) = v.parse::<bool>() {
                    TokenValue::Bool(b)
                } else {
                    TokenValue::String(v)
                }
            }
            _ => TokenValue::String(v),
        }
    }
}

// Reader scan the input.
pub struct Reader {
    pub chars: Vec<char>,
    pub index: usize,
}

impl Reader {
    pub fn from(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        Self { chars, index: 0 }
    }

    // Next character.
    pub fn next(&mut self) -> Option<char> {
        if self.index < self.chars.len() {
            let current_index = self.index;
            self.index += 1;

            self.chars.get(current_index).cloned()
        } else {
            None
        }
    }

    // Put rewinds one character.
    pub fn put(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }
}

// Quoted string token reader.
pub struct Quoted<'a> {
    reader: &'a mut Reader,
}

impl Quoted<'_> {
    // Read token
    pub fn read(&mut self) -> Result<Token, String> {
        let mut last_ch: Option<char> = None;
        let mut bfr: Vec<char> = vec![];
        let quote = self
            .reader
            .next()
            .ok_or("Could not get first quoted character".to_string())?;

        loop {
            match self.reader.next() {
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
}

// Operator token reader.
pub struct Operator<'a> {
    reader: &'a mut Reader,
}

impl Operator<'_> {
    // Read token
    pub fn read(&mut self) -> Result<Token, String> {
        let mut bfr: Vec<char> = vec![];
        loop {
            match self.reader.next() {
                Some(ch) => match ch {
                    COLON | COMMA | OR | EQ | LIKE | NOT | LT | GT => bfr.push(ch),
                    _ => {
                        self.reader.put();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer1() {
        let result = Lexer::with("name:elmer,age:20".to_string());
        assert_eq!(
            result,
            Ok(Lexer {
                tokens: vec![
                    Token {
                        kind: Kind::Literal,
                        value: "name".chars().collect()
                    },
                    Token {
                        kind: Kind::Operator,
                        value: vec![COLON]
                    },
                    Token {
                        kind: Kind::Literal,
                        value: "elmer".chars().collect()
                    },
                    //
                    Token {
                        kind: Kind::Operator,
                        value: vec![COMMA]
                    },
                    //
                    Token {
                        kind: Kind::Literal,
                        value: "age".chars().collect()
                    },
                    Token {
                        kind: Kind::Operator,
                        value: vec![COLON]
                    },
                    Token {
                        kind: Kind::Literal,
                        value: "20".chars().collect()
                    },
                ],
                index: 0
            })
        );
    }

    #[test]
    fn test_or() {
        let result = Lexer::with("name:\"one|two\"".to_string());
        assert_eq!(
            result,
            Ok(Lexer {
                tokens: vec![
                    Token {
                        kind: Kind::Literal,
                        value: "name".chars().collect()
                    },
                    Token {
                        kind: Kind::Operator,
                        value: vec![COLON]
                    },
                    Token {
                        kind: Kind::String,
                        value: "one|two".chars().collect()
                    },
                ],
                index: 0
            })
        );
    }

    #[test]
    fn test_equal() {
        let result = Lexer::with("name:\"one=two\"".to_string());
        assert_eq!(
            result,
            Ok(Lexer {
                tokens: vec![
                    Token {
                        kind: Kind::Literal,
                        value: "name".chars().collect()
                    },
                    Token {
                        kind: Kind::Operator,
                        value: vec![COLON]
                    },
                    Token {
                        kind: Kind::String,
                        value: "one=two".chars().collect()
                    },
                ],
                index: 0
            })
        );
    }

    #[test]
    fn test_grouping() {
        let result = Lexer::with("name:\"(one|two)\"".to_string());
        assert_eq!(
            result,
            Ok(Lexer {
                tokens: vec![
                    Token {
                        kind: Kind::Literal,
                        value: "name".chars().collect()
                    },
                    Token {
                        kind: Kind::Operator,
                        value: vec![COLON]
                    },
                    Token {
                        kind: Kind::String,
                        value: "(one|two)".chars().collect()
                    },
                ],
                index: 0
            })
        );
    }

    #[test]
    fn test_escaped() {
        let result = Lexer::with("name:\"hello world\"".to_string());
        assert_eq!(
            result,
            Ok(Lexer {
                tokens: vec![
                    Token {
                        kind: Kind::Literal,
                        value: "name".chars().collect()
                    },
                    Token {
                        kind: Kind::Operator,
                        value: vec![COLON]
                    },
                    Token {
                        kind: Kind::String,
                        value: "hello world".chars().collect()
                    },
                ],
                index: 0
            })
        );
    }

    #[test]
    fn test_escaped_and_operator() {
        let result = Lexer::with("name = \"elmer\" , age > 20".to_string());
        assert_eq!(
            result,
            Ok(Lexer {
                tokens: vec![
                    Token {
                        kind: Kind::Literal,
                        value: "name".chars().collect()
                    },
                    Token {
                        kind: Kind::Operator,
                        value: vec![EQ]
                    },
                    Token {
                        kind: Kind::String,
                        value: "elmer".chars().collect()
                    },
                    //
                    Token {
                        kind: Kind::Operator,
                        value: vec![COMMA]
                    },
                    //
                    Token {
                        kind: Kind::Literal,
                        value: "age".chars().collect()
                    },
                    Token {
                        kind: Kind::Operator,
                        value: vec![GT]
                    },
                    Token {
                        kind: Kind::Literal,
                        value: "20".chars().collect()
                    },
                ],
                index: 0
            })
        );
    }

    #[test]
    fn test_like_operator() {
        let result = Lexer::with("name~elmer*".to_string());
        assert_eq!(
            result,
            Ok(Lexer {
                tokens: vec![
                    Token {
                        kind: Kind::Literal,
                        value: "name".chars().collect()
                    },
                    Token {
                        kind: Kind::Operator,
                        value: vec![LIKE]
                    },
                    Token {
                        kind: Kind::Literal,
                        value: "elmer*".chars().collect()
                    },
                ],
                index: 0
            })
        );
    }

    #[test]
    fn test_grouping_with_or_operator() {
        let result = Lexer::with("name=(one|two|three)".to_string());
        assert_eq!(
            result,
            Ok(Lexer {
                tokens: vec![
                    Token {
                        kind: Kind::Literal,
                        value: "name".chars().collect()
                    },
                    Token {
                        kind: Kind::Operator,
                        value: vec![EQ]
                    },
                    Token {
                        kind: Kind::Lparen,
                        value: vec![LPAREN]
                    },
                    Token {
                        kind: Kind::Literal,
                        value: "one".chars().collect()
                    },
                    Token {
                        kind: Kind::Operator,
                        value: vec![OR]
                    },
                    Token {
                        kind: Kind::Literal,
                        value: "two".chars().collect()
                    },
                    Token {
                        kind: Kind::Operator,
                        value: vec![OR]
                    },
                    Token {
                        kind: Kind::Literal,
                        value: "three".chars().collect()
                    },
                    Token {
                        kind: Kind::Rparen,
                        value: vec![RPAREN]
                    },
                ],
                index: 0
            })
        );
    }

    #[test]
    fn test_grouping_with_and_operator() {
        let result = Lexer::with("name=(one,two,three)".to_string());
        assert_eq!(
            result,
            Ok(Lexer {
                tokens: vec![
                    Token {
                        kind: Kind::Literal,
                        value: "name".chars().collect()
                    },
                    Token {
                        kind: Kind::Operator,
                        value: vec![EQ]
                    },
                    Token {
                        kind: Kind::Lparen,
                        value: vec![LPAREN]
                    },
                    Token {
                        kind: Kind::Literal,
                        value: "one".chars().collect()
                    },
                    Token {
                        kind: Kind::Operator,
                        value: vec![AND]
                    },
                    Token {
                        kind: Kind::Literal,
                        value: "two".chars().collect()
                    },
                    Token {
                        kind: Kind::Operator,
                        value: vec![AND]
                    },
                    Token {
                        kind: Kind::Literal,
                        value: "three".chars().collect()
                    },
                    Token {
                        kind: Kind::Rparen,
                        value: vec![RPAREN]
                    },
                ],
                index: 0
            })
        );
    }

    #[test]
    fn test_quoted() {
        let result = Lexer::with("name:'elmer'".to_string());
        assert_eq!(
            result,
            Ok(Lexer {
                tokens: vec![
                    Token {
                        kind: Kind::Literal,
                        value: "name".chars().collect()
                    },
                    Token {
                        kind: Kind::Operator,
                        value: vec![COLON]
                    },
                    Token {
                        kind: Kind::String,
                        value: "elmer".chars().collect()
                    },
                ],
                index: 0
            })
        );
    }
}
