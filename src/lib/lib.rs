extern crate peg;

use peg::error::ParseError;
use peg::str::LineCol;
use std::fs;

pub mod ast;
pub mod parser;

fn pretty_error(file: &str, err: ParseError<LineCol>) -> String {
    let token = file.chars().nth(err.location.offset).unwrap();
    format!(
        "Unexpected token '{}' at line {}, column {}, expected one of {}",
        token,
        err.location.line,
        err.location.column,
        err.expected
            .tokens()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    )
}

pub fn parse_file(path: &str) -> Result<ast::AST, String> {
    let file = fs::read_to_string(path).expect(&format!("Failed to read {}", path));
    parser::pkt::schema(&file).map_err(|err| pretty_error(&file, err))
}
