use super::ast;

peg::parser!(pub grammar pkt() for str {
    /// Parses whitespace
    rule _() = [' ']*
    /// Parses newlines
    rule __() = ['\n'|'\r']*

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

    rule base_type() -> ast::Type
        = "uint8" { ast::Type::Uint8 }
        / "uint16" { ast::Type::Uint16 }
        / "uint32" { ast::Type::Uint32 }
        / "int8" { ast::Type::Int8 }
        / "int16" { ast::Type::Int16 }
        / "int32" { ast::Type::Int32 }
        / "float" { ast::Type::Float }
        / "string" { ast::Type::String }

    rule array() -> ast::Type
        = base:base_type() nesting:$("[]"+) {
            let mut root = ast::Type::Array{ r#type: Box::new(base) };
            let mut count = nesting.len() / 2;
            while count > 1 {
                root = ast::Type::Array{ r#type: Box::new(root) };
                count -= 1;
            }
            root
        }

    /// Recursively parses a type
    rule r#type() -> ast::Type
        = n:array() { n }
        / "flag" { ast::Type::Flag }
        / "tuple" { ast::Type::Tuple }
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
        = __ i:ident() _ ":" _ t:r#type() __ { (i, t) }
        / expected!("declaration")

    /// Parses a schema file
    pub rule schema() -> ast::AST
        = __ s:(decl()*) __ { s }
});
