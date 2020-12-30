extern crate lib;

fn main() {
    match lib::parse_file("test.pkt") {
        Ok(ast) => println!("{:#?}", ast),
        Err(err) => println!("Parsing failed: {}", err),
    };
}
