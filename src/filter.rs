use crate::lexer::Token;
use crate::parser::{Predicate, Value};

#[derive(Debug, PartialEq)]
pub struct Filter {
    pub predicates: Vec<Predicate>,
}

impl Filter {
    // Field returns a field.
    pub fn field(&self, name: &str) -> Option<Field> {
        let fields = self.fields(name);
        let field = fields.first();
        field.cloned()
    }

    // Fields returns fields.
    pub fn fields(&self, name: &str) -> Vec<Field> {
        let mut fields: Vec<Field> = vec![];

        let name = name.to_lowercase();
        for p in self.predicates.iter() {
            let predicate_name: String = p.field.value.iter().collect();
            if predicate_name.to_lowercase() == name {
                let f = Field {
                    predicate: p.clone(),
                };
                fields.push(f);
            }
        }

        fields
    }

    // Resource returns a filter scoped to resource.
    pub fn resource(&self, r: &str) -> Filter {
        let r = r.to_lowercase();
        let mut predicates: Vec<Predicate> = vec![];
        for p in self.predicates.iter() {
            let field = Field {
                predicate: p.clone(),
            };
            if let Some(fr) = &field.resource() {
                if fr.to_lowercase() == r {
                    let field_name = field.name().chars().collect::<Vec<char>>();
                    predicates.push(Predicate {
                        field: Token {
                            kind: p.field.kind.clone(),
                            value: field_name,
                        },
                        ..p.clone()
                    });
                }
            }
        }
        Filter { predicates }
    }

    pub fn is_empty(&self) -> bool {
        self.predicates.is_empty()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Field {
    pub predicate: Predicate,
}

impl Field {
    // Name returns the field name.
    pub fn name(&self) -> String {
        let (_, s) = self.split();
        s
    }

    // Name returns the field name.
    pub fn resource(&self) -> Option<String> {
        let (s, _) = self.split();
        s
    }

    pub fn value(&self) -> Value {
        let value = &self.predicate.value;
        value.clone()
    }

    pub fn operator(&self) -> Token {
        let token = &self.predicate.operator;
        token.clone()
    }

    // SQL builds SQL.
    // Returns statement and values (for ?).
    // pub fn sql(&self) {
    //     let name = self.name();
    //     let r = match self.predicate.value.0.len() {
    //         0 => None,
    //         1 => {
    //             let operator_value: String = self.predicate.operator.value.iter().collect();
    //             if operator_value == LIKE.to_string() {
    //                 if let Some(token) = self.predicate.value.0.get(0) {
    //                     let token_value: String = token.value.iter().collect();
    //                     let v = token_value.replace("*", "%");
    //                     let v_list = vec![TokenValue::String(v)];
    //                     let s = format!("{}{}{}{}", name, &self.operator(), "?", " ");
    //                     Some((s, v_list))
    //                 } else {
    //                     None
    //                 }
    //             } else {
    //                 if let Some(token) = self.predicate.value.0.get(0) {
    //                     let v_list = vec![token.as_value()];
    //                     let s = format!("{}{}{}{}", name, &self.operator(), "?", " ");
    //                     Some((s, v_list))
    //                 } else {
    //                     None
    //                 }
    //             }
    //         }
    //         _ => {
    //             // if f.Value.Operator(AND) {
    //             //     // not supported.
    //             //     break
    //             // }
    //             self.predicate.operator.value
    //             None
    //         },
    //     };
    // }

    // split field name.
    // format: resource.name
    // The resource may be "" (anonymous).
    // The (.) separator is escaped when preceded by (\).
    pub fn split(&self) -> (Option<String>, String) {
        let s = &self.predicate.field.value;
        let mark = s.iter().position(|c| *c == '.');
        if let Some(mark) = mark {
            if mark > 0 && s.get(mark - 1) == Some('\\').as_ref() {
                let name: String = [&s[0..mark - 1], &s[mark..]].concat().iter().collect();
                (None, name)
            } else {
                let relation: String = s[0..mark].iter().collect();
                let name: String = s[mark + 1..].iter().collect();
                (Some(relation), name)
            }
        } else {
            let name: String = s.iter().collect();
            (None, name)
        }
    }

    // // operator returns SQL operator.
    // pub fn operator(&self) -> String {
    //     if self.predicate.value.0.len() == 1 {
    //         let s: String = self.predicate.operator.value.iter().collect();
    //         if s == COLON.to_string() {
    //             "=".to_string()
    //         } else if s == LIKE.to_string() {
    //             "LIKE".to_string()
    //         } else {
    //             s
    //         }
    //     } else {
    //         "IN".to_string()
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{Kind, Token, TokenValue, OR};
    use crate::parser::Parser;

    #[test]
    fn test_filter_empty() {
        let p = Parser::filter("");
        assert_eq!(p, Ok(Filter { predicates: vec![] }));
    }

    #[test]
    fn test_filter_complex() {
        let p =
            Parser::filter("name:elmer,age:20,category=(a|b|c),name.first:elmer,name.last=fudd");
        assert!(p.is_ok());
        let filter = p.unwrap();
        assert_eq!(filter.predicates.len(), 5);
        assert_eq!(filter.is_empty(), false);

        // Test name:elmer
        let field = filter.field("name");
        assert!(field.is_some());
        let field = field.unwrap();
        assert_eq!(field.name(), "name");
        let option = field.predicate.value.0.get(0).map(|c| c.as_value());
        assert_eq!(option, Some(TokenValue::String("elmer".to_string())));

        // Test category
        let field = filter.field("category");
        assert!(field.is_some());
        let field = field.unwrap();
        assert_eq!(field.name(), "category");
        assert_eq!(
            field.predicate.value.by_kind(vec![Kind::Operator]),
            vec![
                Token {
                    kind: Kind::Operator,
                    value: vec![OR]
                },
                Token {
                    kind: Kind::Operator,
                    value: vec![OR]
                }
            ]
        );
        assert_eq!(
            field
                .predicate
                .value
                .by_kind(vec![Kind::Literal, Kind::String]),
            vec![
                Token {
                    kind: Kind::Literal,
                    value: vec!['a']
                },
                Token {
                    kind: Kind::Literal,
                    value: vec!['b']
                },
                Token {
                    kind: Kind::Literal,
                    value: vec!['c']
                }
            ]
        );

        // Test name.first
        let field = filter.field("name.first");
        assert!(field.is_some());
        let field = field.unwrap();
        let option = field.predicate.value.0.get(0).map(|c| c.as_value());
        assert_eq!(option, Some(TokenValue::String("elmer".to_string())));

        // Test name.last
        let field = filter.field("name.last");
        assert!(field.is_some());
        let field = field.unwrap();
        let value = field.predicate.value.0.get(0).map(|c| c.as_value());
        assert_eq!(value, Some(TokenValue::String("fudd".to_string())));

        // Test Resource name.first
        let resource = filter.resource("name");
        let field = resource.field("first");
        assert!(field.is_some());
        let field = field.unwrap();
        assert_eq!(field.name(), "first");
        let value = field.predicate.value.0.get(0).map(|c| c.as_value());
        assert_eq!(value, Some(TokenValue::String("elmer".to_string())));

        // Test Resource Name.First
        let resource = filter.resource("Name");
        let field = resource.field("First");
        assert!(field.is_some());
        let field = field.unwrap();
        assert_eq!(field.name(), "first");
    }

    #[test]
    fn test_filter_resource_numeric() {
        let p = Parser::filter("app.name=test,app.tag.id=0");
        assert!(p.is_ok());

        // Test app.name
        let filter = p.unwrap();
        let filter = filter.resource("app");
        let option = filter.field("name");
        assert!(option.is_some());
        let field = option.unwrap();
        let value = field.predicate.value.0.get(0).map(|c| c.as_value());
        assert_eq!(value, Some(TokenValue::String("test".to_string())));

        // Test Resource app.tag.id
        let filter = filter.resource("tag");
        let option = filter.field("id");
        assert!(option.is_some());
        let field = option.unwrap();
        let value = field.predicate.value.0.get(0).map(|c| c.as_value());
        assert_eq!(value, Some(TokenValue::Number(0)));
    }
}
