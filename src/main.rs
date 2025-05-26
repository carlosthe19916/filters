use crate::lexer::{Kind, Token};
use crate::parser::{Parser, Value};

mod filter;
mod lexer;
mod parser;

fn print_tokens(tokens: Vec<Token>) -> String {
    tokens
        .iter()
        .map(|v| format!("{}", v.as_value()))
        .collect::<Vec<_>>()
        .join(" AND ")
}

fn print_value(value: Value) -> String {
    let tokens = value.by_kind(vec![Kind::Literal, Kind::String]);
    print_tokens(tokens)
}

fn print_value_operator(value: Value) -> String {
    let tokens = value.by_kind(vec![Kind::Operator]);
    print_tokens(tokens)
}

fn print_line() {
    println!(
        "--------------------------------------------------------------------------------------------------------------------------------------------"
    )
}

fn main() {
    print_line();
    let parser = Parser::filter("filterText~'special characters like =<>~', age>=18, name=(jim|crossley), address='single quote address', nickname=\"use double quotes\", labels=('kubernetes.io/part-of: trustify', 'kubernetes.io/part-of: operator')").unwrap();

    let field_filter_text = parser.field("filterText").unwrap();
    println!(
        "FIELD:'{}'\t OPERATOR:'{}'\t VALUE:'{}'",
        field_filter_text.name(),
        field_filter_text.operator(),
        print_value(field_filter_text.value()),
    );
    print_line();

    let field_age = parser.field("age").unwrap();
    println!(
        "FIELD:'{}'\t\t OPERATOR:'{}'\t VALUE:'{}'",
        field_age.name(),
        field_age.operator(),
        print_value(field_age.value()),
    );
    print_line();

    let field_name = parser.field("name").unwrap();
    println!(
        "FIELD:'{}'\t\t OPERATOR:'{}'\t VALUE:'{}'\t\t\t\t\t\t\t VALUE_OPERATOR:'{}'",
        field_name.name(),
        field_name.operator(),
        print_value(field_name.value()),
        print_value_operator(field_name.value()),
    );
    print_line();

    let field_address = parser.field("address").unwrap();
    println!(
        "FIELD:'{}'\t\t OPERATOR:'{}'\t VALUE:'{}'\t\t\t",
        field_address.name(),
        field_address.operator(),
        print_value(field_address.value()),
    );
    print_line();

    let field_nickname = parser.field("nickname").unwrap();
    println!(
        "FIELD:'{}'\t OPERATOR:'{}'\t VALUE:'{}'",
        field_nickname.name(),
        field_nickname.operator(),
        print_value(field_nickname.value()),
    );
    print_line();

    let field_labels = parser.field("labels").unwrap();
    println!(
        "FIELD:'{}'\t\t OPERATOR:'{}'\t VALUE:'{}'\t VALUE_OPERATOR:'{}'",
        field_labels.name(),
        field_labels.operator(),
        print_value(field_labels.value()),
        print_value_operator(field_labels.value()),
    );
    print_line();
}
