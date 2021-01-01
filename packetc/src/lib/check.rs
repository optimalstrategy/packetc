//! Contains all the type-checking code
//!
//! Type-checking is done in two passes, so that it's possible to have lexical scoping.
#![allow(clippy::needless_collect)]
use super::*;
use std::collections::HashMap;
use std::rc::Rc;

// TODO: real error type + report in a nice way
// TODO: DRY this code

fn get_export(ast: &[ast::Node]) -> Result<String, String> {
    let mut export = None;
    for node in ast {
        match node {
            ast::Node::Export(n) if export.is_none() => export = Some(n.clone()),
            ast::Node::Export(_) => return Err("Schema has more than one export".to_string()),
            _ => (),
        }
    }
    match export {
        Some(e) => Ok(e),
        None => Err("Schema has no export".to_string()),
    }
}

fn collect_types(ast: &[ast::Node]) -> Result<HashMap<String, ast::Type>, String> {
    let mut cache = HashMap::new();

    for node in ast {
        if let ast::Node::Decl(n, t) = node {
            if cache.contains_key(n) {
                return Err(format!("Schema has duplicate declaration: {}", n));
            }
            cache.insert(n.clone(), t.clone());
        }
    }

    Ok(cache)
}

#[derive(Clone, PartialEq, Debug)]
pub enum Builtin {
    Uint8,
    Uint16,
    Uint32,
    Int8,
    Int16,
    Int32,
    Float,
    String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct EnumVariant {
    pub name: String,
    pub value: usize,
}
#[derive(Clone, PartialEq, Debug)]
pub enum EnumRepr {
    U8,
    U16,
    U32,
}
#[derive(Clone, PartialEq, Debug)]
pub struct Enum {
    pub repr: EnumRepr,
    pub variants: Vec<EnumVariant>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct StructField {
    pub name: String,
    pub r#type: Rc<ResolvedType>,
    pub array: bool,
}
#[derive(Clone, PartialEq, Debug)]
pub struct Struct {
    pub fields: Vec<StructField>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ResolvedType {
    Builtin(Builtin),
    Enum {
        repr: EnumRepr,
        variants: Vec<EnumVariant>,
    },
    Struct {
        fields: Vec<StructField>,
    },
}

fn get_builtins() -> HashMap<String, Rc<ResolvedType>> {
    vec![
        (
            "uint8".to_string(),
            Rc::new(ResolvedType::Builtin(Builtin::Uint8)),
        ),
        (
            "uint16".to_string(),
            Rc::new(ResolvedType::Builtin(Builtin::Uint16)),
        ),
        (
            "uint32".to_string(),
            Rc::new(ResolvedType::Builtin(Builtin::Uint32)),
        ),
        (
            "int8".to_string(),
            Rc::new(ResolvedType::Builtin(Builtin::Int8)),
        ),
        (
            "int16".to_string(),
            Rc::new(ResolvedType::Builtin(Builtin::Int16)),
        ),
        (
            "int32".to_string(),
            Rc::new(ResolvedType::Builtin(Builtin::Int32)),
        ),
        (
            "float".to_string(),
            Rc::new(ResolvedType::Builtin(Builtin::Float)),
        ),
        (
            "string".to_string(),
            Rc::new(ResolvedType::Builtin(Builtin::String)),
        ),
    ]
    .into_iter()
    .collect()
}

fn resolve_one_first_pass(
    name: String,
    ty: ast::Type,
    resolved: &mut HashMap<String, Rc<ResolvedType>>,
    unresolved: &mut HashMap<String, ast::Type>,
) -> Result<(), String> {
    match ty {
        ast::Type::Enum(e) => {
            // resolve the variants by giving each one a value
            let mut count = 0usize;
            let variants =
                e.0.into_iter()
                    .map(|s| EnumVariant {
                        name: s,
                        value: {
                            count += 1;
                            count - 1
                        },
                    })
                    .collect();
            // find the smallest possible representation for this enum
            let repr = match count {
                n if n <= 8 => EnumRepr::U8,
                n if n <= 16 => EnumRepr::U16,
                n if n <= 32 => EnumRepr::U32,
                n => return Err(format!("Enum {} has too many variants ({})", name, n)),
            };
            // enums can always be resolved immediately
            resolved.insert(name, Rc::new(ResolvedType::Enum { repr, variants }));
        }
        ast::Type::Struct(s) => {
            // iterate over each field in the struct
            // if its type can't be found in `resolved` types,
            // add 1 to not_resolved
            let mut not_resolved = 0usize;
            let fields =
                s.0.iter()
                    .map(|(fname, tty)| {
                        if let Some(r#type) = resolved.get(&tty.0) {
                            Some(StructField {
                                name: fname.clone(),
                                r#type: r#type.clone(),
                                array: tty.1,
                            })
                        } else {
                            not_resolved += 1;
                            None
                        }
                    })
                    .collect::<Vec<Option<StructField>>>();
            // if we're resolved all types, add the whole struct to `resolved` types
            // otherwise add it back to `unresolved`
            if not_resolved == 0 {
                resolved.insert(
                    name,
                    Rc::new(ResolvedType::Struct {
                        fields: fields
                            .into_iter()
                            .map(|field| field.unwrap())
                            .collect::<Vec<StructField>>(),
                    }),
                );
            } else {
                unresolved.insert(name, ast::Type::Struct(s));
            }
        }
    }
    Ok(())
}

// This should consume the AST and return a type-checked version
fn resolve_first_pass(
    ast: ast::AST,
    resolved: &mut HashMap<String, Rc<ResolvedType>>,
    unresolved: &mut HashMap<String, ast::Type>,
) -> Result<(), String> {
    for node in ast {
        if let ast::Node::Decl(name, ty) = node {
            if let Err(err) = resolve_one_first_pass(name, ty, resolved, unresolved) {
                return Err(err);
            }
        }
    }
    Ok(())
}

fn resolve_one_second_pass(
    name: String,
    ty: ast::Type,
    resolved: &mut HashMap<String, Rc<ResolvedType>>,
) -> Result<(), String> {
    // we only have to resolve structs
    if let ast::Type::Struct(s) = ty {
        let mut not_resolved = Vec::new();
        let fields =
            s.0.into_iter()
                .map(|(fname, tty)| {
                    if let Some(r#type) = resolved.get(&tty.0) {
                        Some(StructField {
                            name: fname,
                            r#type: r#type.clone(),
                            array: tty.1,
                        })
                    } else {
                        not_resolved.push(fname);
                        None
                    }
                })
                .collect::<Vec<Option<StructField>>>();
        if not_resolved.is_empty() {
            resolved.insert(
                name,
                Rc::new(ResolvedType::Struct {
                    fields: fields
                        .into_iter()
                        .map(|field| field.unwrap())
                        .collect::<Vec<StructField>>(),
                }),
            );
        } else {
            return Err(format!(
                "Fields {} in struct {} could not be resolved",
                not_resolved.join(", "),
                name
            ));
        }
    };
    Ok(())
}

fn resolve_second_pass(
    resolved: &mut HashMap<String, Rc<ResolvedType>>,
    mut unresolved: HashMap<String, ast::Type>,
) -> Result<(), String> {
    for (name, ty) in unresolved.drain() {
        if let Err(err) = resolve_one_second_pass(name, ty, resolved) {
            return Err(err);
        }
    }
    Ok(())
}

#[derive(Clone, PartialEq, Debug)]
pub struct Resolved {
    pub export: String,
    pub types: HashMap<String, Rc<ResolvedType>>,
}

pub fn type_check(ast: ast::AST) -> Result<Resolved, String> {
    let export = match get_export(&ast) {
        Ok(e) => e,
        Err(e) => return Err(e),
    };
    let mut resolved = get_builtins();
    let mut unresolved = match collect_types(&ast) {
        Ok(t) => t,
        Err(e) => return Err(e),
    };
    if let Err(err) = resolve_first_pass(ast, &mut resolved, &mut unresolved) {
        return Err(err);
    };
    if let Err(err) = resolve_second_pass(&mut resolved, unresolved) {
        return Err(err);
    };

    Ok(Resolved {
        export,
        types: resolved,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_check_passes() {
        use ast::*;
        let test: AST = vec![
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
        println!("{:#?}", type_check(test).unwrap())
    }
}
