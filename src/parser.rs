use crate::lexer::{AND, Kind, OR};
use crate::{
    filter::Filter,
    lexer::{COMMA, Lexer, Token},
};

pub struct Parser;

impl Parser {
    pub fn filter(filter: &str) -> Result<Filter, String> {
        if filter.len() == 0 {
            return Ok(Filter { predicates: vec![] });
        }

        let mut predicates: Vec<Predicate> = vec![];

        let mut lexer = Lexer::with(format!("{}{}", COMMA, filter))?;
        let mut brf: Vec<Token> = vec![];
        loop {
            if let Some(token) = &lexer.next() {
                if brf.len() > 2 {
                    match (brf.get(0), brf.get(1), brf.get(2)) {
                        (Some(first), Some(second), Some(third)) => {
                            match (&first.kind, &second.kind, &third.kind) {
                                (Kind::Operator, _, Kind::Operator) => match token.kind {
                                    Kind::Literal | Kind::String => {
                                        let p = Predicate {
                                            unused: first.clone(),
                                            field: second.clone(),
                                            operator: third.clone(),
                                            value: Value(vec![token.clone()]),
                                        };
                                        predicates.push(p);
                                        brf.clear();
                                    }
                                    Kind::Lparen => {
                                        lexer.put();
                                        let mut list = List { lexer: &mut lexer };
                                        let v = list.build()?;
                                        let p = Predicate {
                                            unused: first.clone(),
                                            field: second.clone(),
                                            operator: third.clone(),
                                            value: v,
                                        };
                                        predicates.push(p);
                                        brf.clear();
                                    }
                                    Kind::Operator | Kind::Rparen => {
                                        // Do nothing.
                                    }
                                },
                                _ => break Err("Syntax error.".to_string()),
                            }
                        }
                        _ => break Err(format!("Could not find index {} and {}", 0, 2)),
                    }
                } else {
                    brf.push(token.clone());
                }
            } else {
                break Ok(());
            }
        }?;

        if brf.len() == 0 {
            Ok(Filter { predicates })
        } else {
            Err("Syntax error.".to_string())
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Predicate {
    pub unused: Token,
    pub field: Token,
    pub operator: Token,
    pub value: Value,
}

// Value term value.
#[derive(Debug, PartialEq)]
pub struct Value(Vec<Token>);

impl Value {
    // ByKind returns values by kind.
    pub fn by_kind(&self, kind: Vec<Kind>) -> Vec<Token> {
        let mut matched: Vec<Token> = vec![];
        for t in self.0.iter() {
            for k in kind.iter() {
                if &t.kind == k {
                    matched.push(t.clone());
                }
            }
        }
        matched
    }

    pub fn operator(&self, operator: Kind) -> bool {
        let operators = self.by_kind(vec![operator]);
        if operators.len() > 0 {
            operators[0].value.get(0) == None
        } else {
            false
        }
    }

    pub fn join(&self, operator: Kind) -> Value {
        let mut tokens: Vec<Token> = vec![];
        for i in 0..self.by_kind(vec![Kind::Literal, Kind::String]).len() {
            if i > 0 {
                tokens.push(Token {
                    kind: Kind::Operator,
                    value: vec![],
                });
            }
            tokens.push(self.0[i].clone());
        }
        Value(tokens)
    }
}

// List construct.
// Example: (red|blue|green)
pub struct List<'a> {
    lexer: &'a mut Lexer,
}

impl<'a> List<'a> {
    // Build the value.
    pub fn build(&mut self) -> Result<Value, String> {
        let mut v = Value(vec![]);

        loop {
            if let Some(token) = self.lexer.next() {
                match token.kind {
                    Kind::Literal | Kind::String => v.0.push(token),
                    Kind::Operator => {
                        let value_string: String = token.value.clone().into_iter().collect();
                        if value_string == AND.to_string() || value_string == OR.to_string() {
                            v.0.push(token.clone());
                        } else {
                            break Err("List separator must be `,` `|`".to_string());
                        }
                    }
                    Kind::Lparen => {
                        // ignored
                    }
                    Kind::Rparen => {
                        self.validate(&v)?;
                        break Ok(v);
                    }
                }
            } else {
                break Err("End ')' not found.".to_string());
            }
        }
    }

    // validate the result.
    pub fn validate(&mut self, v: &Value) -> Result<(), String> {
        let mut last_op: Option<char> = None;

        let mut i = 0;
        loop {
            if v.0.len() == 0 {
                break Err("List cannot be empty.".to_string());
            }
            if i >= v.0.len() {
                break Ok(());
            }

            if let Some(token) = v.0.get(i) {
                match (i % 2 == 0, &token.kind) {
                    (true, Kind::Literal | Kind::String) => {
                        // do nothing
                    }
                    (true, Kind::Operator | Kind::Lparen | Kind::Rparen) => {
                        break Err("(LITERAL|STRING) not expected in ()".to_string());
                    }
                    (false, Kind::Operator) => {
                        if let Some(operator) = token.value.get(0) {
                            let operator = operator.clone();
                            if let Some(last_op) = last_op {
                                if operator != last_op {
                                    break Err("Mixed operator detected in ().".to_string());
                                }
                            }
                            last_op = Some(operator);
                        } else {
                            last_op = None
                        }
                    }
                    (false, Kind::Literal | Kind::String | Kind::Lparen | Kind::Rparen) => {
                        break Err("OPERATOR expected in ()".to_string());
                    }
                };
            } else {
                break Err(format!("Expected element at index `{}`", i));
            }

            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{COLON, EQ};
    #[test]
    fn test_parser() {
        let p = Parser::filter("");
        assert!(p.is_ok());

        let p = Parser::filter("name:elmer,age:20");
        assert_eq!(
            p,
            Ok(Filter {
                predicates: vec![
                    Predicate {
                        unused: Token {
                            kind: Kind::Operator,
                            value: vec![COMMA]
                        },
                        field: Token {
                            kind: Kind::Literal,
                            value: "name".chars().collect()
                        },
                        operator: Token {
                            kind: Kind::Operator,
                            value: vec![COLON]
                        },
                        value: Value(vec![Token {
                            kind: Kind::Literal,
                            value: "elmer".chars().collect()
                        }]),
                    },
                    Predicate {
                        unused: Token {
                            kind: Kind::Operator,
                            value: vec![COMMA]
                        },
                        field: Token {
                            kind: Kind::Literal,
                            value: "age".chars().collect()
                        },
                        operator: Token {
                            kind: Kind::Operator,
                            value: vec![COLON]
                        },
                        value: Value(vec![Token {
                            kind: Kind::Literal,
                            value: "20".chars().collect()
                        }]),
                    }
                ]
            })
        );

        let p = Parser::filter("name:elmer,category=(one|two|three),age:20");
        assert_eq!(
            p,
            Ok(Filter {
                predicates: vec![
                    Predicate {
                        unused: Token {
                            kind: Kind::Operator,
                            value: vec![COMMA]
                        },
                        field: Token {
                            kind: Kind::Literal,
                            value: "name".chars().collect()
                        },
                        operator: Token {
                            kind: Kind::Operator,
                            value: vec![COLON]
                        },
                        value: Value(vec![Token {
                            kind: Kind::Literal,
                            value: "elmer".chars().collect()
                        }]),
                    },
                    Predicate {
                        unused: Token {
                            kind: Kind::Operator,
                            value: vec![COMMA]
                        },
                        field: Token {
                            kind: Kind::Literal,
                            value: "category".chars().collect()
                        },
                        operator: Token {
                            kind: Kind::Operator,
                            value: vec![EQ]
                        },
                        value: Value(vec![
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
                            }
                        ]),
                    },
                    Predicate {
                        unused: Token {
                            kind: Kind::Operator,
                            value: vec![COMMA]
                        },
                        field: Token {
                            kind: Kind::Literal,
                            value: "age".chars().collect()
                        },
                        operator: Token {
                            kind: Kind::Operator,
                            value: vec![COLON]
                        },
                        value: Value(vec![Token {
                            kind: Kind::Literal,
                            value: "20".chars().collect()
                        }]),
                    },
                ]
            })
        );

        let p = Parser::filter("cat=()");
        assert!(p.is_err());

        let p = Parser::filter("cat=(one|two,three)");
        assert!(p.is_err());
    }
}
