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
    /// Parses the first character of an identifier, which cannot contain numbers
    rule ident_start() -> String = s:$(['a'..='z'|'A'..='Z'|'_']) { s.to_string() }
    /// Parses any alphanumeric characters as part of an identifier
    rule ident_chars() -> String = s:$(['a'..='z'|'A'..='Z'|'0'..='9'|'_']) { s.to_string() }
    /// Parses an entire identifier, ensuring no reserved keywords are used
    rule ident() -> String
        = i:quiet!{ $(!reserved() ident_start() ident_chars()*) } { i.to_string() }

    /// Parses the array base type - this is different from r#type()
    /// by not including array() in the descent list, which could
    /// create an infinite loop
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

    /// Parses a array in the form `identifier: type[]`
    rule array() -> ast::Type
        = base:array_type() nesting:$("[]"+) {
            let mut root = ast::Type::Array{ r#type: Box::new(base) };
            let mut count = nesting.len() / 2;
            for _ in 1..count {
                root = ast::Type::Array{ r#type: Box::new(root) };
            }
            root
        }

    /// Parses a flag in the form `identifier: { VARIANT_A, VARIANT_B }`
    rule flag() -> ast::Type
        = "{" ___ variants:$(ident() ** (___ "," ___)) ___ "}" {
            ast::Type::Flag {
                variants: variants.split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            }
        }

    /// Parses a tuple element in the form `identifier: type`
    /// it's separate because PEG doesn't allow binding inside of
    /// nested expressions, so the binding is done here
    rule tuple_element() -> (String, ast::Type)
        = i:ident() _ ":" _ t:r#type() ___ ","? ___ { (i, t) }

    /// Parses a tuple in the form
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

#[cfg(test)]
mod tests {
    use super::*;

    use peg::str::LineCol;

    #[test]
    fn parse_whitespace_before_array_bracket() {
        let test_str = r#"
arr: uint8 []
"#;
        let expected = LineCol {
            line: 2,
            column: 12,
            offset: 12,
        };
        assert_eq!(pkt::schema(test_str).unwrap_err().location, expected);
    }

    #[test]
    fn parse_undefined_type() {
        let test_str = r#"
aaa: uint
"#;
        let expected = LineCol {
            line: 2,
            column: 6,
            offset: 6,
        };
        assert_eq!(pkt::schema(test_str).unwrap_err().location, expected);
    }

    #[test]
    fn parse_unclosed_tuple_parentheses() {
        let test_str = r#"
aaa: (x: float, y: float
"#;
        let expected = LineCol {
            line: 3,
            column: 1,
            offset: 26,
        };
        assert_eq!(pkt::schema(test_str).unwrap_err().location, expected);
    }

    #[test]
    fn parse_unclosed_flag_brackets() {
        let test_str = r#"
aaa: { A, B 
"#;
        let expected = LineCol {
            line: 3,
            column: 1,
            offset: 14,
        };
        assert_eq!(pkt::schema(test_str).unwrap_err().location, expected);
    }

    #[test]
    fn parse_unclosed_array_brackets() {
        let test_str = r#"
aaa: uint8[
"#;
        let expected = LineCol {
            line: 2,
            column: 11,
            offset: 11,
        };
        assert_eq!(pkt::schema(test_str).unwrap_err().location, expected);
    }

    #[test]
    fn parse_reserved_identifier() {
        let test_str = r#"
uint8: uint8
"#;
        let expected = LineCol {
            line: 2,
            column: 1,
            offset: 1,
        };
        assert_eq!(pkt::schema(test_str).unwrap_err().location, expected);
    }

    #[test]
    fn parse_first_char_numeric_bad_identifier() {
        let test_str = r#"
0aaa: uint8
"#;
        let expected = LineCol {
            line: 2,
            column: 1,
            offset: 1,
        };
        assert_eq!(pkt::schema(test_str).unwrap_err().location, expected);
    }

    #[test]
    fn parse_comment() {
        let test_str = r#"
# this is a comment.
"#;
        let expected: ast::AST = vec![];
        assert_eq!(pkt::schema(test_str).unwrap(), expected);
    }

    #[test]
    fn parse_comment_right_of_line() {
        let test_str = r#"
aaa: uint8 # this is a comment placed to the right of a line.
"#;
        let expected: ast::AST = vec![("aaa".to_string(), ast::Type::Uint8)];
        assert_eq!(pkt::schema(test_str).unwrap(), expected);
    }

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
        let expected: ast::AST = vec![
            ("u8".to_string(), ast::Type::Uint8),
            ("u16".to_string(), ast::Type::Uint16),
            ("u32".to_string(), ast::Type::Uint32),
            ("i8".to_string(), ast::Type::Int8),
            ("i16".to_string(), ast::Type::Int16),
            ("i32".to_string(), ast::Type::Int32),
            ("f32".to_string(), ast::Type::Float),
        ];
        assert_eq!(pkt::schema(test_str).unwrap(), expected);
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
        let expected: ast::AST = vec![
            (
                "u8".to_string(),
                ast::Type::Array {
                    r#type: Box::new(ast::Type::Uint8),
                },
            ),
            (
                "u16".to_string(),
                ast::Type::Array {
                    r#type: Box::new(ast::Type::Uint16),
                },
            ),
            (
                "u32".to_string(),
                ast::Type::Array {
                    r#type: Box::new(ast::Type::Uint32),
                },
            ),
            (
                "i8".to_string(),
                ast::Type::Array {
                    r#type: Box::new(ast::Type::Int8),
                },
            ),
            (
                "i16".to_string(),
                ast::Type::Array {
                    r#type: Box::new(ast::Type::Int16),
                },
            ),
            (
                "i32".to_string(),
                ast::Type::Array {
                    r#type: Box::new(ast::Type::Int32),
                },
            ),
            (
                "f32".to_string(),
                ast::Type::Array {
                    r#type: Box::new(ast::Type::Float),
                },
            ),
        ];
        assert_eq!(pkt::schema(test_str).unwrap(), expected);
    }

    #[test]
    fn parse_array_nested() {
        let test_str = r#"
u8: uint8[][]
u16: uint16[][]
u32: uint32[][]
i8: int8[][]
i16: int16[][]
i32: int32[][]
f32: float[][]
"#;
        let expected: ast::AST = vec![
            (
                "u8".to_string(),
                ast::Type::Array {
                    r#type: Box::new(ast::Type::Array {
                        r#type: Box::new(ast::Type::Uint8),
                    }),
                },
            ),
            (
                "u16".to_string(),
                ast::Type::Array {
                    r#type: Box::new(ast::Type::Array {
                        r#type: Box::new(ast::Type::Uint16),
                    }),
                },
            ),
            (
                "u32".to_string(),
                ast::Type::Array {
                    r#type: Box::new(ast::Type::Array {
                        r#type: Box::new(ast::Type::Uint32),
                    }),
                },
            ),
            (
                "i8".to_string(),
                ast::Type::Array {
                    r#type: Box::new(ast::Type::Array {
                        r#type: Box::new(ast::Type::Int8),
                    }),
                },
            ),
            (
                "i16".to_string(),
                ast::Type::Array {
                    r#type: Box::new(ast::Type::Array {
                        r#type: Box::new(ast::Type::Int16),
                    }),
                },
            ),
            (
                "i32".to_string(),
                ast::Type::Array {
                    r#type: Box::new(ast::Type::Array {
                        r#type: Box::new(ast::Type::Int32),
                    }),
                },
            ),
            (
                "f32".to_string(),
                ast::Type::Array {
                    r#type: Box::new(ast::Type::Array {
                        r#type: Box::new(ast::Type::Float),
                    }),
                },
            ),
        ];
        assert_eq!(pkt::schema(test_str).unwrap(), expected);
    }

    #[test]
    fn parse_flag() {
        let test_str = r#"
flag: { A, B }
"#;
        let expected: ast::AST = vec![(
            "flag".to_string(),
            ast::Type::Flag {
                variants: vec!["A".to_string(), "B".to_string()],
            },
        )];
        assert_eq!(pkt::schema(test_str).unwrap(), expected);
    }

    #[test]
    fn parse_flag_array() {
        let test_str = r#"
flag: { A, B }[]
"#;
        let expected: ast::AST = vec![(
            "flag".to_string(),
            ast::Type::Array {
                r#type: Box::new(ast::Type::Flag {
                    variants: vec!["A".to_string(), "B".to_string()],
                }),
            },
        )];
        assert_eq!(pkt::schema(test_str).unwrap(), expected);
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
        let expected: ast::AST = vec![(
            "tuple".to_string(),
            ast::Type::Tuple {
                elements: vec![
                    ("u8".to_string(), ast::Type::Uint8),
                    ("u16".to_string(), ast::Type::Uint16),
                    ("u32".to_string(), ast::Type::Uint32),
                    ("i8".to_string(), ast::Type::Int8),
                    ("i16".to_string(), ast::Type::Int16),
                    ("i32".to_string(), ast::Type::Int32),
                    ("f32".to_string(), ast::Type::Float),
                ],
            },
        )];
        assert_eq!(pkt::schema(test_str).unwrap(), expected);
    }

    #[test]
    fn parse_tuple_trailing_comma() {
        let test_str = r#"
tuple: (
    u8: uint8,
    f32: float,
)
"#;
        let expected: ast::AST = vec![(
            "tuple".to_string(),
            ast::Type::Tuple {
                elements: vec![
                    ("u8".to_string(), ast::Type::Uint8),
                    ("f32".to_string(), ast::Type::Float),
                ],
            },
        )];
        assert_eq!(pkt::schema(test_str).unwrap(), expected);
    }

    #[test]
    fn parse_tuple_array() {
        let test_str = r#"
tuple: (
    u8: uint8,
    f32: float,
)[]
"#;
        let expected: ast::AST = vec![(
            "tuple".to_string(),
            ast::Type::Array {
                r#type: Box::new(ast::Type::Tuple {
                    elements: vec![
                        ("u8".to_string(), ast::Type::Uint8),
                        ("f32".to_string(), ast::Type::Float),
                    ],
                }),
            },
        )];
        assert_eq!(pkt::schema(test_str).unwrap(), expected);
    }

    #[test]
    fn parse_tuple_of_array() {
        let test_str = r#"
tuple: (
    u8: uint8[],
    f32: float[],
)
"#;
        let expected: ast::AST = vec![(
            "tuple".to_string(),
            ast::Type::Tuple {
                elements: vec![
                    (
                        "u8".to_string(),
                        ast::Type::Array {
                            r#type: Box::new(ast::Type::Uint8),
                        },
                    ),
                    (
                        "f32".to_string(),
                        ast::Type::Array {
                            r#type: Box::new(ast::Type::Float),
                        },
                    ),
                ],
            },
        )];
        assert_eq!(pkt::schema(test_str).unwrap(), expected);
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
        let expected: ast::AST = vec![(
            "complex_type".to_string(),
            ast::Type::Tuple {
                elements: vec![
                    (
                        "flag".to_string(),
                        ast::Type::Flag {
                            variants: vec!["A".to_string(), "B".to_string()],
                        },
                    ),
                    (
                        "positions".to_string(),
                        ast::Type::Array {
                            r#type: Box::new(ast::Type::Tuple {
                                elements: vec![
                                    ("x".to_string(), ast::Type::Float),
                                    ("y".to_string(), ast::Type::Float),
                                ],
                            }),
                        },
                    ),
                    (
                        "names".to_string(),
                        ast::Type::Array {
                            r#type: Box::new(ast::Type::String),
                        },
                    ),
                    (
                        "values".to_string(),
                        ast::Type::Array {
                            r#type: Box::new(ast::Type::Tuple {
                                elements: vec![
                                    ("a".to_string(), ast::Type::Uint32),
                                    ("b".to_string(), ast::Type::Int32),
                                    ("c".to_string(), ast::Type::Uint8),
                                    ("d".to_string(), ast::Type::Uint8),
                                ],
                            }),
                        },
                    ),
                ],
            },
        )];
        assert_eq!(pkt::schema(test_str).unwrap(), expected);
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
        let expected: ast::AST = vec![(
            "complex_type".to_string(),
            ast::Type::Tuple {
                elements: vec![
                    (
                        "flag".to_string(),
                        ast::Type::Flag {
                            variants: vec!["A".to_string(), "B".to_string()],
                        },
                    ),
                    (
                        "positions".to_string(),
                        ast::Type::Array {
                            r#type: Box::new(ast::Type::Tuple {
                                elements: vec![
                                    ("x".to_string(), ast::Type::Float),
                                    ("y".to_string(), ast::Type::Float),
                                ],
                            }),
                        },
                    ),
                    (
                        "names".to_string(),
                        ast::Type::Array {
                            r#type: Box::new(ast::Type::String),
                        },
                    ),
                    (
                        "values".to_string(),
                        ast::Type::Array {
                            r#type: Box::new(ast::Type::Tuple {
                                elements: vec![
                                    ("a".to_string(), ast::Type::Uint32),
                                    ("b".to_string(), ast::Type::Int32),
                                    ("c".to_string(), ast::Type::Uint8),
                                    ("d".to_string(), ast::Type::Uint8),
                                ],
                            }),
                        },
                    ),
                ],
            },
        )];
        assert_eq!(pkt::schema(test_str).unwrap(), expected);
    }
}
