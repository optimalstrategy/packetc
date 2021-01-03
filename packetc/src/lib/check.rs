//! Contains all the type-checking code
//!
//! Type-checking is done in two passes, so that it's possible to have lexical scoping.
use super::*;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::{cell::RefCell, fmt, fmt::Display, fmt::Formatter};

// TODO: real error type + report in a nice way
// TODO!: enforce more than one variant on enums
// TODO!: enforce unique field names
// TODO!: enforce unique type names
// TODO!: discard empty structs + output warning
// TODO!: discard unused types (can use Rc::strong_count() > 1) + output warning

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
impl Display for EnumRepr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EnumRepr::U8 => write!(f, "u8"),
            EnumRepr::U16 => write!(f, "u16"),
            EnumRepr::U32 => write!(f, "u32"),
        }
    }
}
#[derive(Clone, PartialEq, Debug)]
pub struct Enum {
    pub repr: EnumRepr,
    pub variants: Vec<EnumVariant>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct StructField {
    pub name: String,
    pub r#type: Ptr<(String, ResolvedType)>,
    pub array: bool,
}
#[derive(Clone, PartialEq, Debug)]
pub struct Struct {
    pub fields: Vec<StructField>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ResolvedType {
    Builtin(Builtin),
    Enum(Enum),
    Struct(Struct),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Ptr<T>(pub Rc<RefCell<T>>);
impl<T> Ptr<T> {
    pub fn new(value: T) -> Ptr<T> {
        Ptr(Rc::new(RefCell::new(value)))
    }
}
impl<T> std::ops::Deref for Ptr<T> {
    type Target = Rc<RefCell<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Export {
    pub name: String,
    pub r#struct: Struct,
}

fn get_builtins() -> HashMap<String, Ptr<(String, ResolvedType)>> {
    vec![
        (
            "uint8".to_string(),
            Ptr::new(("uint8".to_string(), ResolvedType::Builtin(Builtin::Uint8))),
        ),
        (
            "uint16".to_string(),
            Ptr::new(("uint16".to_string(), ResolvedType::Builtin(Builtin::Uint16))),
        ),
        (
            "uint32".to_string(),
            Ptr::new(("uint32".to_string(), ResolvedType::Builtin(Builtin::Uint32))),
        ),
        (
            "int8".to_string(),
            Ptr::new(("int8".to_string(), ResolvedType::Builtin(Builtin::Int8))),
        ),
        (
            "int16".to_string(),
            Ptr::new(("int16".to_string(), ResolvedType::Builtin(Builtin::Int16))),
        ),
        (
            "int32".to_string(),
            Ptr::new(("int32".to_string(), ResolvedType::Builtin(Builtin::Int32))),
        ),
        (
            "float".to_string(),
            Ptr::new(("float".to_string(), ResolvedType::Builtin(Builtin::Float))),
        ),
        (
            "string".to_string(),
            Ptr::new(("string".to_string(), ResolvedType::Builtin(Builtin::String))),
        ),
    ]
    .into_iter()
    .collect()
}

fn resolve_struct_field(
    fname: String,
    fty: ast::Unresolved,
    resolved: &HashMap<String, Ptr<(String, ResolvedType)>>,
) -> Option<StructField> {
    match resolved.get(&fty.0) {
        Some(rty) => Some(StructField {
            name: fname,
            r#type: rty.clone(),
            array: fty.1,
        }),
        None => None,
    }
}

fn resolve_enum(name: &str, ty: ast::Enum) -> Result<(EnumRepr, Vec<EnumVariant>), String> {
    // resolve the variants by assigning each one to a single bit
    let mut count = 0usize;
    let variants =
        ty.0.into_iter()
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
        n => return Err(format!("Enum '{}' has too many variants ({}/32)", name, n)),
    };
    Ok((repr, variants))
}

fn resolve_one_first_pass(
    name: String,
    ty: ast::Type,
    builtins: &HashMap<String, Ptr<(String, ResolvedType)>>,
    first_pass: &mut HashMap<String, Ptr<(String, ResolvedType)>>,
    unresolved: &mut HashMap<String, ast::Type>,
) -> Result<(), String> {
    match ty {
        ast::Type::Enum(e) => {
            match resolve_enum(&name, e) {
                Ok(rty) => {
                    unresolved.remove(&name);
                    first_pass.insert(
                        name.clone(),
                        Ptr::new((
                            name,
                            ResolvedType::Enum(Enum {
                                repr: rty.0,
                                variants: rty.1,
                            }),
                        )),
                    );
                }
                Err(err) => return Err(err),
            };
        }
        ast::Type::Struct(s) => {
            // TODO: prevent recursive struct type
            // by checking if the type references itself
            let mut fields = Vec::new();
            for (fname, fty) in s.0.iter() {
                if let Some(field) = resolve_struct_field(fname.clone(), fty.clone(), builtins) {
                    fields.push(field);
                } else {
                    break;
                }
            }
            if fields.len() == s.0.len() {
                unresolved.remove(&name);
                first_pass.insert(
                    name.clone(),
                    Ptr::new((name, ResolvedType::Struct(Struct { fields }))),
                );
            }
        }
    }
    Ok(())
}

// This should consume the AST and return a type-checked version
fn resolve_first_pass(
    ast: ast::AST,
    builtins: &HashMap<String, Ptr<(String, ResolvedType)>>,
    first_pass: &mut HashMap<String, Ptr<(String, ResolvedType)>>,
    unresolved: &mut HashMap<String, ast::Type>,
) -> Result<(), String> {
    for node in ast {
        if let ast::Node::Decl(name, ty) = node {
            if let Err(err) = resolve_one_first_pass(name, ty, builtins, first_pass, unresolved) {
                return Err(err);
            }
        }
    }
    Ok(())
}

fn resolve_one_second_pass(
    name: String,
    ty: ast::Type,
    cache: &mut HashMap<String, Ptr<(String, ResolvedType)>>,
    visited: &mut HashSet<String>,
    unresolved: &mut HashMap<String, ast::Type>,
) -> Result<(), String> {
    // if it's already resolved, dont resolve again
    if cache.contains_key(&name) {
        return Ok(());
    }
    // otherwise try to resolve it
    if let ast::Type::Struct(s) = ty {
        // if we've already visited this type, it's a cycle
        if visited.contains(&name) {
            return Err(format!(
                "Found a cycle between two or more top level definitions in type '{}'",
                &name
            ));
        }
        visited.insert(name.clone());
        // iterate over each field, trying to resolve it
        // store any field (+ its type) which could not be resolved
        let mut not_resolved = Vec::new();
        let mut fields = Vec::new();
        for (field_name, field_type) in s.0.into_iter() {
            if let Some(field) = resolve_struct_field(field_name.clone(), field_type.clone(), cache) {
                fields.push(field);
            } else {
                not_resolved.push((field_name, field_type));
            }
        }
        if not_resolved.is_empty() {
            // if all the fields are resolved, construct the type and cache it
            cache.insert(
                name.clone(),
                Ptr::new((name, ResolvedType::Struct(Struct { fields }))),
            );
        } else {
            // otherwise, for each field that couldn't be resolved, try to resolve it
            for (_, field_type) in not_resolved.iter() {
                let ftype_name = field_type.0.clone();
                // try to find the field's typename in whatever is left unresolved
                if let Some(utype) = unresolved.remove(&ftype_name) {
                    // if it exists, try to resolve it by recursively calling
                    // the function we're in
                    if let Err(e) = resolve_one_second_pass(ftype_name, utype, cache, visited, unresolved) {
                        // it may fail, so propagate the error out
                        return Err(e);
                    }
                } else {
                    // if the field's typename is not in unresolved, that means it doesn't exist
                    // (because it isn't resolved nor unresolved)
                    return Err(format!("Declaration for type '{}' does not exist", ftype_name));
                }
            }
            // if we get here, it means all the field's types were successfully resolved and placed in the cache
            // so finish resolving our fields
            let now_resolved = not_resolved
                .into_iter()
                .map(|(fname, fty)| resolve_struct_field(fname, fty, cache))
                .map(|f| f.unwrap())
                .collect::<Vec<StructField>>();
            // and we have a complete type
            cache.insert(
                name.clone(),
                Ptr::new((
                    name,
                    ResolvedType::Struct(Struct {
                        fields: fields.into_iter().chain(now_resolved).collect(),
                    }),
                )),
            );
        }
    } else {
        panic!(format!(
            "Something unresolved which is not a struct got into the second pass: {:#?}",
            ty
        ));
    }
    Ok(())
}

fn resolve_second_pass(
    cache: &mut HashMap<String, Ptr<(String, ResolvedType)>>,
    mut unresolved: HashMap<String, ast::Type>,
) -> Result<(), String> {
    let mut visited = HashSet::new();
    for (name, ty) in unresolved.clone() {
        if let Err(err) = resolve_one_second_pass(name, ty, cache, &mut visited, &mut unresolved) {
            return Err(err);
        }
    }
    Ok(())
}

fn get_struct_variant(ty: &ResolvedType) -> Struct {
    match ty {
        ResolvedType::Struct(s) => s.clone(),
        _ => panic!("ResolvedType is not struct"),
    }
}

fn resolve_export(
    name: String,
    resolved: &HashMap<String, Ptr<(String, ResolvedType)>>,
) -> Result<Export, String> {
    if let Some(export) = resolved.get(&name) {
        if std::mem::discriminant(&(*export.borrow()).1)
            == std::mem::discriminant(&ResolvedType::Struct(Struct { fields: Vec::new() }))
        {
            Ok(Export {
                name,
                r#struct: get_struct_variant(&(*export.borrow()).1),
            })
        } else {
            Err(format!("Attempted to export '{}', which is not a struct", name))
        }
    } else {
        Err(format!("Export '{}' could not be resolved", name))
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Resolved {
    pub export: Export,
    pub types: HashMap<String, Ptr<(String, ResolvedType)>>,
}

pub fn type_check(ast: ast::AST) -> Result<Resolved, String> {
    let export = match get_export(&ast) {
        Ok(e) => e,
        Err(e) => return Err(e),
    };
    let mut unresolved = match collect_types(&ast) {
        Ok(t) => t,
        Err(e) => return Err(e),
    };

    // pre-pass: collect builtins
    let cache = get_builtins();
    // first pass: collect enums + structs with only builtins as field types
    let mut first_pass = HashMap::new();
    if let Err(err) = resolve_first_pass(ast, &cache, &mut first_pass, &mut unresolved) {
        return Err(err);
    };
    // second pass: collect structs with other structs (made up of builtins) as field types
    let mut cache = cache.into_iter().chain(first_pass).collect::<HashMap<_, _>>();
    if let Err(err) = resolve_second_pass(&mut cache, unresolved) {
        return Err(err);
    };
    // export pass: collect the resolved type we're exporting
    let export = match resolve_export(export, &cache) {
        Ok(e) => e,
        Err(err) => return Err(err),
    };
    Ok(Resolved { export, types: cache })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_check_passes() {
        // check if a valid AST containing all language features passes the type check
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
        // TODO: check equality of Resolved AST instead of checking if this is an error
        // Rc<T> == Rc<T> if T == T, according to https://doc.rust-lang.org/src/alloc/rc.rs.html#1325
        type_check(test).unwrap();
    }

    #[test]
    fn too_many_exports() {
        // a schema file may have only one export
        use ast::*;
        let test: AST = vec![
            Node::Decl(
                "Position".to_string(),
                Type::Struct(Struct(vec![
                    ("x".to_string(), Unresolved("float".to_string(), false)),
                    ("y".to_string(), Unresolved("float".to_string(), false)),
                ])),
            ),
            Node::Export("Position".to_string()),
            Node::Export("Position".to_string()),
        ];
        assert_eq!(
            type_check(test).unwrap_err(),
            "Schema has more than one export".to_string()
        );
    }

    #[test]
    fn no_export() {
        // a schema file must export something
        use ast::*;
        let test: AST = vec![Node::Decl(
            "Position".to_string(),
            Type::Struct(Struct(vec![
                ("x".to_string(), Unresolved("float".to_string(), false)),
                ("y".to_string(), Unresolved("float".to_string(), false)),
            ])),
        )];
        assert_eq!(type_check(test).unwrap_err(), "Schema has no export".to_string());
    }

    #[test]
    fn duplicate_declaration() {
        // a top-level declaration must have a unique name
        use ast::*;
        let test: AST = vec![
            Node::Decl(
                "Position".to_string(),
                Type::Struct(Struct(vec![
                    ("x".to_string(), Unresolved("float".to_string(), false)),
                    ("y".to_string(), Unresolved("float".to_string(), false)),
                ])),
            ),
            Node::Decl(
                "Position".to_string(),
                Type::Struct(Struct(vec![
                    ("x".to_string(), Unresolved("float".to_string(), false)),
                    ("y".to_string(), Unresolved("float".to_string(), false)),
                ])),
            ),
            Node::Export("Position".to_string()),
        ];
        assert_eq!(
            type_check(test).unwrap_err(),
            "Schema has duplicate declaration: Position"
        );
    }

    #[test]
    fn too_many_enum_variants() {
        // because each variant is assigned to a bit, and
        // the maximum representation size is 32 bit,
        // an enum may have at most 32 variants
        use ast::*;
        let test: AST = vec![
            Node::Decl(
                "Flag".to_string(),
                Type::Enum(Enum(vec![
                    "A0".to_string(),
                    "A1".to_string(),
                    "A2".to_string(),
                    "A3".to_string(),
                    "A4".to_string(),
                    "A5".to_string(),
                    "A6".to_string(),
                    "A7".to_string(),
                    "A8".to_string(),
                    "A9".to_string(),
                    "A10".to_string(),
                    "A11".to_string(),
                    "A12".to_string(),
                    "A13".to_string(),
                    "A14".to_string(),
                    "A15".to_string(),
                    "A16".to_string(),
                    "A17".to_string(),
                    "A18".to_string(),
                    "A19".to_string(),
                    "A20".to_string(),
                    "A21".to_string(),
                    "A22".to_string(),
                    "A23".to_string(),
                    "A24".to_string(),
                    "A25".to_string(),
                    "A26".to_string(),
                    "A27".to_string(),
                    "A28".to_string(),
                    "A29".to_string(),
                    "A30".to_string(),
                    "A31".to_string(),
                    // one too many
                    "A32".to_string(),
                ])),
            ),
            Node::Decl(
                "Test".to_string(),
                Type::Struct(Struct(vec![(
                    "flag".to_string(),
                    Unresolved("Flag".to_string(), false),
                )])),
            ),
            Node::Export("Test".to_string()),
        ];
        assert_eq!(
            type_check(test).unwrap_err(),
            "Enum 'Flag' has too many variants (33/32)"
        );
    }

    #[test]
    fn could_not_resolve_unknown() {
        // an unresolvable type is either non-existant, nested, or recursive type
        // in this case the type that's being resolved doesn't exist
        use ast::*;
        let test: AST = vec![
            Node::Decl(
                "Test".to_string(),
                Type::Struct(Struct(vec![(
                    "flag".to_string(),
                    Unresolved("Flag".to_string(), false),
                )])),
            ),
            Node::Export("Test".to_string()),
        ];
        assert_eq!(
            type_check(test).unwrap_err(),
            "Declaration for type 'Flag' does not exist".to_string()
        );
    }

    #[test]
    fn resolve_nested() {
        // the type that's being resolved is too deeply nested
        use ast::*;
        let test: AST = vec![
            Node::Decl(
                "Flag".to_string(),
                Type::Enum(Enum(vec!["A".to_string(), "B".to_string()])),
            ),
            Node::Decl(
                "TestA".to_string(),
                Type::Struct(Struct(vec![(
                    "test".to_string(),
                    Unresolved("Flag".to_string(), false),
                )])),
            ),
            Node::Decl(
                "TestB".to_string(),
                Type::Struct(Struct(vec![(
                    "test".to_string(),
                    Unresolved("TestA".to_string(), false),
                )])),
            ),
            Node::Decl(
                "TestC".to_string(),
                Type::Struct(Struct(vec![(
                    "test".to_string(),
                    Unresolved("TestB".to_string(), false),
                )])),
            ),
            Node::Export("TestC".to_string()),
        ];
        type_check(test).unwrap();
    }

    #[test]
    fn could_not_resolve_recursive() {
        // the type that's being resolved references itself
        use ast::*;
        let test: AST = vec![
            Node::Decl(
                "Test".to_string(),
                Type::Struct(Struct(vec![(
                    "test".to_string(),
                    Unresolved("Test".to_string(), false),
                )])),
            ),
            Node::Export("Test".to_string()),
        ];
        let actual = type_check(test);
        assert_eq!(
            actual.unwrap_err(),
            "Found a cycle between two or more top level definitions in type 'Test'".to_string()
        );
    }

    #[test]
    fn could_not_resolve_export() {
        // the type does not exist
        use ast::*;
        let test: AST = vec![Node::Export("Test".to_string())];
        assert_eq!(
            type_check(test).unwrap_err(),
            "Export 'Test' could not be resolved".to_string()
        );
    }

    #[test]
    fn export_only_struct() {
        // only structs may be exported
        use ast::*;
        let test: AST = vec![
            Node::Decl(
                "Flag".to_string(),
                Type::Enum(Enum(vec!["A".to_string(), "B".to_string()])),
            ),
            Node::Export("Flag".to_string()),
        ];
        assert_eq!(
            type_check(test).unwrap_err(),
            "Attempted to export 'Flag', which is not a struct".to_string()
        );
    }
}
