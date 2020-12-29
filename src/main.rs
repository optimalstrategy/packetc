extern crate parser;

fn main() {
    match parser::parse_file("test.pkt") {
        Ok(ast) => println!("{:#?}", ast),
        Err(err) => println!("Parser failed: {}", err),
    };
}
