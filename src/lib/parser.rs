use super::ast;

peg::parser!(pub grammar pkt() for str {
    /// Parses whitespace
    rule _() = [' ']*
    /// Parses newlines
    rule __() = ['\n'|'\r']*
    /// Parses whitespace or newlines
    rule ___() = [' ' | '\n' | '\r']*

    /// Parses a single-line comment
    rule comment()
        = "#" [ch if ch != '\n']* __

    /// Parses reserved keywords
    rule reserved()
        = "uint8"
        / "uint16"
        / "uint32"
        / "int8"
        / "int16"
        / "int32"
        / "float"
        / "string"
    /// Parses the first character of an identifier, which cannot contain numbers
    rule ident_start() -> String = s:$(['a'..='z'|'A'..='Z'|'_']) { s.to_string() }
    /// Parses any alphanumeric characters as part of an identifier
    rule ident_chars() -> String = s:$(['a'..='z'|'A'..='Z'|'0'..='9'|'_']) { s.to_string() }
    /// Parses a single identifier
    rule ident() -> String
        = i:quiet!{ $(!reserved() ident_start() ident_chars()*) } { i.to_string() }

    rule array_type() -> ast::Type
        = f:flag() { f }
        / t:tuple() { t }
        / "uint8" { ast::Type::Uint8 }
        / "uint16" { ast::Type::Uint16 }
        / "uint32" { ast::Type::Uint32 }
        / "int8" { ast::Type::Int8 }
        / "int16" { ast::Type::Int16 }
        / "int32" { ast::Type::Int32 }
        / "float" { ast::Type::Float }
        / "string" { ast::Type::String }

    rule array() -> ast::Type
        = base:array_type() nesting:$("[]"+) {
            let mut root = ast::Type::Array{ r#type: Box::new(base) };
            let mut count = nesting.len() / 2;
            for _ in 1..count {
                root = ast::Type::Array{ r#type: Box::new(root) };
            }
            root
        }

    rule flag() -> ast::Type
        = "{" ___ variants:$(ident() ** (___ "," ___)) ___ "}" {
            ast::Type::Flag {
                variants: variants.split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            }
        }

    rule tuple_element() -> (String, ast::Type)
        = i:ident() _ ":" _ t:r#type() ___ ","? ___ { (i, t) }

    rule tuple() -> ast::Type
        = "(" ___ elements:tuple_element()+ ___ ")" {
            ast::Type::Tuple { elements }
        }

    /// Recursively parses a type
    rule r#type() -> ast::Type
        = n:array() { n }
        / f:flag() { f }
        / t:tuple() { t }
        / "uint8" { ast::Type::Uint8 }
        / "uint16" { ast::Type::Uint16 }
        / "uint32" { ast::Type::Uint32 }
        / "int8" { ast::Type::Int8 }
        / "int16" { ast::Type::Int16 }
        / "int32" { ast::Type::Int32 }
        / "float" { ast::Type::Float }
        / "string" { ast::Type::String }

    /// Parses a declaration in the form `identifier : type`
    rule decl() -> ast::Node
        = _ i:ident() _ ":" _ t:r#type() ___ { (i, t) }
        / expected!("declaration")

    rule line() -> Option<ast::Node>
        = _ comment() __ { None }
        / _ s:(decl()) __ { Some(s) }

    /// Parses a schema file
    pub rule schema() -> ast::AST
        = __? lines:(line()*) {
            lines.into_iter()
                .filter_map(|x| x)
                .collect()
        }
});

// TODO: check AST instead of .unwrap()
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_numeric() {
        let test_str = r#"
u8: uint8
u16: uint16
u32: uint32
i8: int8
i16: int16
i32: int32
f32: float
"#;
        pkt::schema(test_str).unwrap();
    }

    #[test]
    fn parse_array() {
        let test_str = r#"
u8: uint8[]
u16: uint16[]
u32: uint32[]
i8: int8[]
i16: int16[]
i32: int32[]
f32: float[]
"#;
        pkt::schema(test_str).unwrap();
    }

    #[test]
    fn parse_array_nested() {
        let test_str = r#"
u8: uint8[][]
u16: uint16[][][][][]
u32: uint32[][]
i8: int8[][]
i16: int16[][]
i32: int32[][]
f32: float[][]
"#;
        pkt::schema(test_str).unwrap();
    }

    #[test]
    fn parse_flag() {
        let test_str = r#"
flag: { A, B }
"#;
        pkt::schema(test_str).unwrap();
    }

    #[test]
    fn parse_flag_array() {
        let test_str = r#"
flag: { A, B }[]
"#;
        pkt::schema(test_str).unwrap();
    }

    #[test]
    fn parse_tuple() {
        let test_str = r#"
tuple: (
    u8: uint8,
    u16: uint16,
    u32: uint32,
    i8: int8,
    i16: int16,
    i32: int32,
    f32: float
)
"#;
        pkt::schema(test_str).unwrap();
    }

    #[test]
    fn parse_tuple_trailing_comma() {
        let test_str = r#"
tuple: (
    u8: uint8,
    f32: float,
)
"#;
        pkt::schema(test_str).unwrap();
    }

    #[test]
    fn parse_tuple_array() {
        let test_str = r#"
tuple: (
    u8: uint8,
    f32: float,
)[]
"#;
        pkt::schema(test_str).unwrap();
    }

    #[test]
    fn parse_tuple_of_array() {
        let test_str = r#"
tuple: (
    u8: uint8[],
    f32: float[],
)
"#;
        pkt::schema(test_str).unwrap();
    }

    #[test]
    fn parse_complex() {
        let test_str = r#"
complex_type: (
    flag: { A, B },
    positions: (x: float, y: float)[],
    names: string[],
    values: (
        a: uint32,
        b: int32,
        c: uint8,
        d: uint8
    )[]
)
"#;
        pkt::schema(test_str).unwrap();
    }

    #[test]
    fn parse_complex_weird_whitespace() {
        let test_str = r#"
complex_type: ( 
    flag : {   A,  B },
     positions: ( x : float, y: float )[],
    names:  string[],
    values : (
  a : uint32,
        b: int32,
        c:  uint8,  d : uint8
      )[]
)
"#;
        pkt::schema(test_str).unwrap();
    }
}
