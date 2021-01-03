use super::ast;
use ast::*;

// TODO: allow specifying max array size -> use it to shrink array len encoding if possible
// right now, it's uint32 by default, which is very wasteful

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

    rule string() -> String
        = s:$(['a'..='z'|'A'..='Z'|'0'..='9'|'_']*) { s.to_string() }

    /// Parses reserved keywords (the base types)
    rule reserved()
        = "uint8"
        / "uint16"
        / "uint32"
        / "int8"
        / "int16"
        / "int32"
        / "float"
        / "string"
        / "enum"
        / "struct"
    /// Parses the first character of an identifier, which cannot contain numbers
    rule ident_start() -> String = s:$(['a'..='z'|'A'..='Z'|'_']) { s.to_string() }
    /// Parses any alphanumeric characters as part of an identifier
    rule ident_chars() -> String = s:$(['a'..='z'|'A'..='Z'|'0'..='9'|'_']) { s.to_string() }
    /// Parses an entire identifier, ensuring no reserved keywords are used
    rule ident() -> String
        = i:quiet!{ $(!reserved() ident_start() ident_chars()*) } { i.to_string() }

    rule enum_variant() -> String
        = s:ident() ___ ","? ___ { s }
    /// Parses an enum in the form `identifier: enum { VARIANT_A, ... }`
    rule enum_type() -> Enum
        = _ "enum" _ "{" ___ variants:(enum_variant()*) ___ "}" { Enum(variants) }

    rule struct_field() -> (String, Unresolved)
        = i:ident() _ ":" _ t:string() a:("[]"?) ___ ","? ___ { (i, Unresolved(t, a.is_some())) }
    /// Parses a struct in the from `identifier: struct { name: type or type[], ... }
    rule struct_type() -> Struct
        = _ "struct" _ "{" ___ fields:(struct_field()*) ___ "}" { Struct(fields) }

    /// Recursively parses a type
    rule r#type() -> Type
        = e:enum_type() { Type::Enum(e) }
        / s:struct_type() { Type::Struct(s) }

    /// Parses a declaration in the form `identifier : type`
    rule decl() -> Node
        = _ i:ident() _ ":" _ t:r#type() ___ {
            Node::Decl(i, t)
        }

    rule export() -> Node
        = "export" _ s:string() {
            Node::Export(s)
        }

    rule line() -> Option<Node>
        = _ comment() __ { None }
        / _ e:(export()) __ { Some(e) }
        / _ s:(decl()) __ { Some(s) }

    /// Parses a schema file
    pub rule schema() -> AST
        = __? lines:(line()*) {
            lines.into_iter()
                .filter_map(|x| x)
                .collect()
        }
});

#[cfg(test)]
mod tests {
    use super::*;

    use peg::str::LineCol;

    // This exists so that test case strings are leading whitespace insensitive
    trait TestCaseString {
        fn build(&self) -> String;
    }
    impl TestCaseString for str {
        fn build(&self) -> String {
            self.split('\n')
                .map(|s| s.trim_start())
                .collect::<Vec<&str>>()
                .join("\n")
        }
    }

    #[test]
    fn parse_whitespace_before_array_bracket() {
        let test = r#"
        a: struct { v: uint8 [] }
        "#
        .build();
        let expected = LineCol {
            line: 2,
            column: 22,
            offset: 22,
        };
        let actual = pkt::schema(&test).unwrap_err().location;
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_unclosed_struct_brackets() {
        let test = r#"
        aaa: struct { x:float, y:float
        "#
        .build();
        let expected = LineCol {
            line: 3,
            column: 1,
            offset: 32,
        };
        let actual = pkt::schema(&test).unwrap_err().location;
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_unclosed_enum_brackets() {
        let test = r#"
        aaa: enum { A, B
        "#
        .build();
        let expected = LineCol {
            line: 3,
            column: 1,
            offset: 18,
        };
        let actual = pkt::schema(&test).unwrap_err().location;
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_unclosed_array_brackets() {
        let test = r#"
        a: struct { v: uint8[ }
        "#
        .build();
        let expected = LineCol {
            line: 2,
            column: 21,
            offset: 21,
        };
        let actual = pkt::schema(&test).unwrap_err().location;
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_reserved_identifier() {
        let test = r#"
        uint8: uint8
        "#
        .build();
        let expected = LineCol {
            line: 2,
            column: 1,
            offset: 1,
        };
        let actual = pkt::schema(&test).unwrap_err().location;
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_first_char_numeric_bad_identifier() {
        let test = r#"
        0aaa: uint8
        "#
        .build();
        let expected = LineCol {
            line: 2,
            column: 1,
            offset: 1,
        };
        let actual = pkt::schema(&test).unwrap_err().location;
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_comment() {
        let test = r#"
        # this is a comment.
        "#
        .build();
        let expected: AST = vec![];
        assert_eq!(pkt::schema(&test).unwrap(), expected);
    }

    #[test]
    fn parse_comment_right_of_line() {
        let test = r#"
        a: struct { v: uint8 } # this is a comment placed to the right of a line.
        "#
        .build();
        let expected: AST = vec![Node::Decl(
            "a".to_string(),
            Type::Struct(Struct(vec![(
                "v".to_string(),
                Unresolved("uint8".to_string(), false),
            )])),
        )];
        assert_eq!(pkt::schema(&test).unwrap(), expected);
    }

    #[test]
    fn parse_enum() {
        let test = r#"
        asdf: enum { A, B }
        "#
        .build();
        let expected: AST = vec![Node::Decl(
            "asdf".to_string(),
            Type::Enum(Enum(vec!["A".to_string(), "B".to_string()])),
        )];
        assert_eq!(pkt::schema(&test).unwrap(), expected);
    }

    #[test]
    fn parse_struct() {
        let test = r#"
        asdf: struct {
            x: float,
            y: float
        }"#
        .build();
        let expected: AST = vec![Node::Decl(
            "asdf".to_string(),
            Type::Struct(Struct(vec![
                ("x".to_string(), Unresolved("float".to_string(), false)),
                ("y".to_string(), Unresolved("float".to_string(), false)),
            ])),
        )];
        assert_eq!(pkt::schema(&test).unwrap(), expected);
    }

    #[test]
    fn parse_struct_trailing_comma() {
        let test = r#"
        asdf: struct {
            a: A,
            b: B,
        }
        "#
        .build();
        let expected: AST = vec![Node::Decl(
            "asdf".to_string(),
            Type::Struct(Struct(vec![
                ("a".to_string(), Unresolved("A".to_string(), false)),
                ("b".to_string(), Unresolved("B".to_string(), false)),
            ])),
        )];
        assert_eq!(pkt::schema(&test).unwrap(), expected);
    }

    #[test]
    fn parse_struct_with_arrays() {
        let test = r#"
        asdf: struct {
            a: A[],
            b: B[],
        }
        "#
        .build();
        let expected: AST = vec![Node::Decl(
            "asdf".to_string(),
            Type::Struct(Struct(vec![
                ("a".to_string(), Unresolved("A".to_string(), true)),
                ("b".to_string(), Unresolved("B".to_string(), true)),
            ])),
        )];
        assert_eq!(pkt::schema(&test).unwrap(), expected);
    }

    #[test]
    fn parse_export() {
        let test = r#"
        export Test
        "#
        .build();
        let expected: AST = vec![Node::Export("Test".to_string())];
        assert_eq!(pkt::schema(&test).unwrap(), expected);
    }

    #[test]
    fn parse_complex() {
        let test = r#"
        # This is a comment.
        # Below is what a fairly complex packet may look like
        Flag: enum { A, B }
        Position: struct { x: float, y: float }
        Value: struct { 
            a: uint32, b: int32, c: uint8, d: uint8
        }
        ComplexType: struct {
            flag: Flag,
            pos: Position,
            names: string[],
            values: Value[]
        }
        export ComplexType
        "#
        .build();
        let expected: AST = vec![
            Node::Decl(
                "Flag".to_string(),
                Type::Enum(Enum(vec!["A".to_string(), "B".to_string()])),
            ),
            Node::Decl(
                "Position".to_string(),
                Type::Struct(Struct(vec![
                    ("x".to_string(), Unresolved("float".to_string(), false)),
                    ("y".to_string(), Unresolved("float".to_string(), false)),
                ])),
            ),
            Node::Decl(
                "Value".to_string(),
                Type::Struct(Struct(vec![
                    ("a".to_string(), Unresolved("uint32".to_string(), false)),
                    ("b".to_string(), Unresolved("int32".to_string(), false)),
                    ("c".to_string(), Unresolved("uint8".to_string(), false)),
                    ("d".to_string(), Unresolved("uint8".to_string(), false)),
                ])),
            ),
            Node::Decl(
                "ComplexType".to_string(),
                Type::Struct(Struct(vec![
                    ("flag".to_string(), Unresolved("Flag".to_string(), false)),
                    ("pos".to_string(), Unresolved("Position".to_string(), false)),
                    ("names".to_string(), Unresolved("string".to_string(), true)),
                    ("values".to_string(), Unresolved("Value".to_string(), true)),
                ])),
            ),
            Node::Export("ComplexType".to_string()),
        ];
        assert_eq!(pkt::schema(&test).unwrap(), expected);
    }
}
