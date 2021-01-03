use super::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct TypeScript {
    imports: HashSet<String>,
}
impl Language for TypeScript {}

impl Common for TypeScript {
    fn gen_common(&self, _: &mut String) {}
}

impl Definition<TypeScript> for check::Builtin {
    fn gen_def(&self, _: &mut TypeScript, name: String, out: &mut String) {
        match self {
            check::Builtin::Uint8 => append!(out, "export type {} = number;\n", name),
            check::Builtin::Uint16 => append!(out, "export type {} = number;\n", name),
            check::Builtin::Uint32 => append!(out, "export type {} = number;\n", name),
            check::Builtin::Int8 => append!(out, "export type {} = number;\n", name),
            check::Builtin::Int16 => append!(out, "export type {} = number;\n", name),
            check::Builtin::Int32 => append!(out, "export type {} = number;\n", name),
            check::Builtin::Float => append!(out, "export type {} = number;\n", name),
            _ => (),
        }
    }
}

impl Definition<TypeScript> for check::Struct {
    fn gen_def(&self, _: &mut TypeScript, name: String, out: &mut String) {
        append!(out, "export interface {} {{\n", name);
        for field in self.fields.iter() {
            let typename = &(*field.r#type.borrow()).0;
            if field.array {
                append!(out, "    {}: {}[],\n", field.name, typename);
            } else {
                append!(out, "    {}: {},\n", field.name, typename);
            }
        }
        append!(out, "}}\n");
    }
}

impl Definition<TypeScript> for check::Enum {
    fn gen_def(&self, _: &mut TypeScript, type_name: String, out: &mut String) {
        append!(out, "export const enum {} {{\n", type_name);
        for variant in self.variants.iter() {
            append!(out, "    {} = 1 << {},\n", variant.name, variant.value);
        }
        append!(out, "}}\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_struct_definition() {
        use check::*;
        let test = Struct {
            fields: vec![
                StructField {
                    name: "testA".to_string(),
                    r#type: Ptr::new((
                        "uint8".to_string(),
                        check::ResolvedType::Builtin(check::Builtin::Uint8),
                    )),
                    array: false,
                },
                StructField {
                    name: "testB".to_string(),
                    r#type: Ptr::new((
                        "uint8".to_string(),
                        check::ResolvedType::Builtin(check::Builtin::Uint8),
                    )),
                    array: false,
                },
            ],
        };
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_def("Test".to_string(), &test);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
export interface Test {
    testA: uint8,
    testB: uint8,
}
"
        );
    }

    #[test]
    fn simple_enum_definition() {
        use check::*;
        let test = Enum {
            repr: EnumRepr::U8,
            variants: vec![
                EnumVariant {
                    name: "A".to_string(),
                    value: 0,
                },
                EnumVariant {
                    name: "B".to_string(),
                    value: 1,
                },
            ],
        };
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_def("Test".to_string(), &test);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
export const enum Test {
    A = 1 << 0,
    B = 1 << 1,
}
"
        );
    }

    #[test]
    fn complex_struct_definition() {
        use check::*;
        let position = Struct {
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    r#type: Ptr::new(("float".to_string(), ResolvedType::Builtin(Builtin::Float))),
                    array: false,
                },
                StructField {
                    name: "y".to_string(),
                    r#type: Ptr::new(("float".to_string(), ResolvedType::Builtin(Builtin::Float))),
                    array: false,
                },
            ],
        };
        let test = Struct {
            fields: vec![
                StructField {
                    name: "testA".to_string(),
                    r#type: Ptr::new(("uint8".to_string(), ResolvedType::Builtin(Builtin::Uint8))),
                    array: false,
                },
                StructField {
                    name: "testB".to_string(),
                    r#type: Ptr::new(("string".to_string(), ResolvedType::Builtin(Builtin::String))),
                    array: true,
                },
                StructField {
                    name: "position".to_string(),
                    r#type: Ptr::new(("Position".to_string(), ResolvedType::Struct(position.clone()))),
                    array: false,
                },
            ],
        };

        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_def("string".to_string(), &Builtin::String);
        gen.push_def("string".to_string(), &Builtin::String);
        gen.push_def("uint8".to_string(), &Builtin::Uint8);
        gen.push_def("float".to_string(), &Builtin::Float);
        gen.push_def("Position".to_string(), &position);
        gen.push_def("Test".to_string(), &test);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
export type uint8 = number;
export type float = number;
export interface Position {
    x: float,
    y: float,
}
export interface Test {
    testA: uint8,
    testB: string[],
    position: Position,
}
"
        );
    }
}
