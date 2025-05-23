use crate::parser::Predicate;
use std::io::BufRead;

#[derive(Debug, PartialEq)]
pub struct Filter {
    pub predicates: Vec<Predicate>,
}

impl Filter {
    // Field returns a field.
    pub fn field(&self, name: &str) -> Option<Field> {
        let fields = self.fields(name);
        let field = fields.get(0);
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
                let name: String = s[0..mark + 1].iter().collect();
                (Some(relation), name)
            }
        } else {
            let name: String = s.iter().collect();
            (None, name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::TokenValue;
    use crate::parser::Parser;

    #[test]
    fn test_filter() {
        let p = Parser::filter("");
        assert_eq!(p, Ok(Filter { predicates: vec![] }));

        //

        let p =
            Parser::filter("name:elmer,age:20,category=(a|b|c),name.first:elmer,name.last=fudd");
        assert!(p.is_ok());
        let filter = p.unwrap();
        assert_eq!(filter.predicates.len(), 5);
        assert_eq!(filter.is_empty(), false);

        let field = filter.field("name");
        assert!(field.is_some());
        let field = field.unwrap();
        assert_eq!(field.name(), "name");
        let option = field.predicate.value.0.get(0).map(|c| c.as_value());
        assert_eq!(option, Some(TokenValue::String("elmer".to_string())));

        let field = filter.field("category");
        assert!(field.is_some());
        let field = field.unwrap();
        assert_eq!(field.name(), "category");
        let a = field.predicate.value;
        //
    }
}
