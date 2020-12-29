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
        / "flag"
        / "array"
        / "tuple"

    /// Parses the first character of an identifier, which cannot contain numbers
    rule ident_start() -> String = s:$(['a'..='z'|'A'..='Z'|'_']) { s.to_string() }
    /// Parses any alphanumeric characters as part of an identifier
    rule ident_chars() -> String = s:$(['a'..='z'|'A'..='Z'|'0'..='9'|'_']) { s.to_string() }
    /// Parses a single identifier
    rule ident() -> String
        = i:quiet!{ $(!reserved() ident_start() ident_chars()*) } { i.to_string() }

    /// Recursively parses a type
    rule r#type() -> ast::Type
        = "uint8" { ast::Type::Uint8 }
        / "uint16" { ast::Type::Uint16 }
        / "uint32" { ast::Type::Uint32 }
        / "int8" { ast::Type::Int8 }
        / "int16" { ast::Type::Int16 }
        / "int32" { ast::Type::Int32 }
        / "float" { ast::Type::Float }
        / "string" { ast::Type::String }
        / "flag" { ast::Type::Flag }
        / "array" { ast::Type::Array }
        / "tuple" { ast::Type::Tuple }

    /// Parses a declaration in the form `identifier : type`
    rule decl() -> ast::Node
        = __ i:ident() _ ":" _ t:r#type() __ { (i, t) }
        / expected!("declaration")

    /// Parses a schema file
    pub rule schema() -> ast::AST
        = __ s:(decl()*) __ { s }
});
