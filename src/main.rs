extern crate libpacketc;

fn main() {
    match libpacketc::parse_file("test.pkt") {
        Ok(ast) => println!("{:#?}", ast),
        Err(err) => println!("Parsing failed: {}", err),
    };
}
