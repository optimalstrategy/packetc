extern crate peg;

use std::fs;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Type {
    Uint8,
    Uint16,
    Uint32,
    Int8,
    Int16,
    Int32,
    Float,
    String,
    Flag,
    Array,
    Tuple,
}

peg::parser!( grammar pkt() for str {
    rule _() = [' ']*
    rule __() = ['\n'|'\r']*
    rule eof() = ![_]

    rule reserved()
        = "uint8"
        / "uint16"
        / "uint32"
        / "int8"
        / "int16"
        / "int32"
        / "float"
        / "string"
        / "flag"
        / "array"
        / "tuple"

    rule ident_start() -> String = s:$(['a'..='z'|'A'..='Z'|'_']) { s.to_string() }
    rule ident_chars() -> String = s:$(['a'..='z'|'A'..='Z'|'0'..='9'|'_']) { s.to_string() }
    rule word() -> String = w:$(ident_start() ident_chars()*) { w.to_string() }
    rule ident() -> String
        = i:$(!reserved() word()) { i.to_string() }
        / expected!("identifier")

    rule r#type() -> Type
        = "uint8" { Type::Uint8 }
        / "uint16" { Type::Uint16 }
        / "uint32" { Type::Uint32 }
        / "int8" { Type::Int8 }
        / "int16" { Type::Int16 }
        / "int32" { Type::Int32 }
        / "float" { Type::Float }
        / "string" { Type::String }
        / "flag" { Type::Flag }
        / "array" { Type::Array }
        / "tuple" { Type::Tuple }
        / expected!("type")

    rule decl() -> (String, Type)
        = __ i:ident() _ ":" _ t:r#type() __ { (i, t) }
        / expected!("declaration")

    pub rule schema() -> Vec<(String, Type)>
        = __ s:(decl()*) __ { s }
        / expected!("schema")
});

fn main() {
    let schema = fs::read_to_string("test.pkt").expect("Failed to read test.pkt");
    let test_decl = pkt::schema(&schema);
    println!("{:#?}", test_decl);
}
