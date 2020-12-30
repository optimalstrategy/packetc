extern crate peg;

use peg::error::ParseError;
use peg::str::LineCol;
use std::fs;

pub mod ast;
pub mod gen;
pub mod parser;

fn pretty_error(file: &str, err: ParseError<LineCol>) -> String {
    let mut out_str = String::new();
    out_str += &format!(
        "\nUnexpected token '{}' at line {}, column {}",
        file.chars().nth(err.location.offset).unwrap_or('\0'),
        err.location.line,
        err.location.column
    );
    let line = file.split('\n').nth(err.location.line - 1).unwrap_or("\0");
    out_str += &format!("\n|\n|  {}\n", line);
    let mark_column = match err.location.column {
        n if n < 1 => 0,
        n => n - 1,
    };
    out_str += &format!("|~~{}^\n", "~".repeat(mark_column));

    out_str
}

pub fn parse_string(s: &str) -> Result<ast::AST, String> {
    parser::pkt::schema(s).map_err(|err| pretty_error(s, err))
}

pub fn parse_file(path: &str) -> Result<ast::AST, String> {
    let file = fs::read_to_string(path).map_err(|_| format!("Failed to read {}", path))?;
    parser::pkt::schema(&file).map_err(|err| pretty_error(&file, err))
}
