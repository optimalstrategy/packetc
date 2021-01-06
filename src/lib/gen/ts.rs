use super::*;
use std::collections::HashSet;

// TODO: use (proc?) macros or some template syntax to clean up all of this duplicated code.
// TODO: change all format! from "{}", var to "{var}", var = var -> prevent some duplicates.
// TODO: change all for..of loops to for (i=0;i<len;++i) loops

#[derive(Clone, PartialEq, Debug, Default)]
pub struct TypeScript {
    imports: HashSet<String>,
}
impl Language for TypeScript {}

impl Common for TypeScript {
    fn gen_common(&self, out: &mut String) {
        append!(out, "import {{ Reader, Writer }} from \"packet\";\n");
    }
}

struct ImplCtx<'a> {
    indentation: String,
    out: &'a mut String,
    stack: Vec<String>,
}

impl<'a> ImplCtx<'a> {
    fn new(out: &'a mut String) -> ImplCtx {
        ImplCtx {
            indentation: "".to_string(),
            out,
            stack: Vec::new(),
        }
    }
    fn push_indent(&mut self) {
        self.indentation += "    ";
    }
    fn pop_indent(&mut self) {
        self.indentation.truncate(if self.indentation.len() < 4 {
            0
        } else {
            self.indentation.len() - 4
        });
    }
    fn push_fname(&mut self, name: String) {
        self.stack.push(name);
    }
    fn pop_fname(&mut self) {
        self.stack.pop();
    }
    fn swap_stack(&mut self, other: &mut Vec<String>) {
        std::mem::swap(&mut self.stack, other);
    }
}

fn varname(stack: &[String], name: &str) -> String {
    format!("{}_{}", stack.join("_"), name)
}

fn fname(stack: &[String]) -> String {
    stack.join(".")
}

fn gen_write_impl_builtin_array(ctx: &mut ImplCtx, type_info: &check::Builtin, type_name: &str) {
    let fname = fname(&ctx.stack);
    append!(
        ctx.out,
        "{}writer.write_uint32({}.length);\n",
        ctx.indentation,
        fname
    );
    let item_var = varname(&ctx.stack, "item");
    // TODO: use index-based for loop instead
    append!(
        ctx.out,
        "{}for (let {} of {}) {{\n",
        ctx.indentation,
        item_var,
        fname
    );
    let mut old_stack = Vec::new();
    ctx.swap_stack(&mut old_stack);
    ctx.push_fname(item_var.clone());
    ctx.push_indent();

    match type_info {
        check::Builtin::String => {
            append!(
                ctx.out,
                "{}writer.write_uint32({}.length);\n",
                ctx.indentation,
                item_var
            );
            append!(
                ctx.out,
                "{}writer.write_string({});\n",
                ctx.indentation,
                item_var
            );
        }
        _ => append!(
            ctx.out,
            "{}writer.write_{}({});\n",
            ctx.indentation,
            type_name,
            item_var
        ),
    }

    ctx.swap_stack(&mut old_stack);
    ctx.pop_indent();
    append!(ctx.out, "{}}}\n", ctx.indentation);
}

fn gen_write_impl_builtin(ctx: &mut ImplCtx, type_info: &check::Builtin, type_name: &str) {
    let fname = fname(&ctx.stack);
    match type_info {
        check::Builtin::String => {
            append!(
                ctx.out,
                "{}writer.write_uint32({}.length);\n",
                ctx.indentation,
                fname
            );
            append!(
                ctx.out,
                "{}writer.write_string({});\n",
                ctx.indentation,
                fname
            );
        }
        _ => append!(
            ctx.out,
            "{}writer.write_{}({});\n",
            ctx.indentation,
            type_name,
            fname,
        ),
    }
}

fn gen_write_impl_enum_array(ctx: &mut ImplCtx, type_info: &check::Enum, _: &str) {
    let fname = fname(&ctx.stack);
    append!(
        ctx.out,
        "{}writer.write_uint32({}.length);\n",
        ctx.indentation,
        fname
    );
    let item_var = varname(&ctx.stack, "item");
    // TODO: use index-based for loop instead
    append!(
        ctx.out,
        "{}for (let {} of {}) {{\n",
        ctx.indentation,
        item_var,
        fname
    );
    let mut old_stack = Vec::new();
    ctx.swap_stack(&mut old_stack);
    ctx.push_fname(item_var);
    ctx.push_indent();

    let repr_name = match &type_info.repr {
        check::EnumRepr::U8 => "uint8",
        check::EnumRepr::U16 => "uint16",
        check::EnumRepr::U32 => "uint32",
    };
    append!(
        ctx.out,
        "{}writer.write_{}({} as number);\n",
        ctx.indentation,
        repr_name,
        self::fname(&ctx.stack),
    );

    ctx.swap_stack(&mut old_stack);
    ctx.pop_indent();
    append!(ctx.out, "{}}}\n", ctx.indentation);
}

fn gen_write_impl_enum(ctx: &mut ImplCtx, type_info: &check::Enum, _: &str) {
    let repr_name = match &type_info.repr {
        check::EnumRepr::U8 => "uint8",
        check::EnumRepr::U16 => "uint16",
        check::EnumRepr::U32 => "uint32",
    };
    append!(
        ctx.out,
        "{}writer.write_{}({} as number);\n",
        ctx.indentation,
        repr_name,
        fname(&ctx.stack),
    );
}

fn gen_write_impl_struct_array(ctx: &mut ImplCtx, type_info: &check::Struct, _: &str) {
    let fname = fname(&ctx.stack);
    append!(
        ctx.out,
        "{}writer.write_uint32({}.length);\n",
        ctx.indentation,
        fname
    );
    let item_var = varname(&ctx.stack, "item");
    // TODO: use index-based for loop instead
    append!(
        ctx.out,
        "{}for (let {} of {}) {{\n",
        ctx.indentation,
        item_var,
        fname
    );
    let mut old_stack = Vec::new();
    ctx.swap_stack(&mut old_stack);
    ctx.push_fname(item_var);
    ctx.push_indent();

    for field in &type_info.fields {
        ctx.push_fname(field.name.clone());
        let field_type = &*field.r#type.borrow();
        match &field_type.1 {
            check::ResolvedType::Builtin(field_type_info) if field.array => {
                gen_write_impl_builtin_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Builtin(field_type_info) => {
                gen_write_impl_builtin(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Enum(field_type_info) if field.array => {
                gen_write_impl_enum_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Enum(field_type_info) => {
                gen_write_impl_enum(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Struct(field_type_info) if field.array => {
                gen_write_impl_struct_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Struct(field_type_info) => {
                gen_write_impl_struct(ctx, &field_type_info, &field_type.0)
            }
        }
        ctx.pop_fname();
    }

    ctx.swap_stack(&mut old_stack);
    ctx.pop_indent();
    append!(ctx.out, "{}}}\n", ctx.indentation);
}

fn gen_write_impl_struct(ctx: &mut ImplCtx, type_info: &check::Struct, _: &str) {
    for field in &type_info.fields {
        ctx.push_fname(field.name.clone());
        let field_type = &*field.r#type.borrow();
        match &field_type.1 {
            check::ResolvedType::Builtin(field_type_info) if field.array => {
                gen_write_impl_builtin_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Builtin(field_type_info) => {
                gen_write_impl_builtin(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Enum(field_type_info) if field.array => {
                gen_write_impl_enum_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Enum(field_type_info) => {
                gen_write_impl_enum(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Struct(field_type_info) if field.array => {
                gen_write_impl_struct_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Struct(field_type_info) => {
                gen_write_impl_struct(ctx, &field_type_info, &field_type.0)
            }
        }
        ctx.pop_fname();
    }
}

impl WriteImpl<TypeScript> for check::Export {
    fn gen_write_impl(&self, _: &mut TypeScript, name: String, out: &mut String) {
        let mut ctx = ImplCtx::new(out);
        ctx.push_fname("input".to_string());
        append!(
            ctx.out,
            "export function write(writer: Writer, input: {}) {{\n",
            name,
        );
        ctx.push_indent();
        gen_write_impl_struct(&mut ctx, &self.r#struct, &name);
        ctx.pop_indent();
        append!(out, "}}\n");
    }
}

fn gen_read_impl_builtin_array(ctx: &mut ImplCtx, type_info: &check::Builtin, type_name: &str) {
    let len_var = varname(&ctx.stack, "len");
    let fname = fname(&ctx.stack);
    append!(
        ctx.out,
        "{}let {} = reader.read_uint32();\n",
        ctx.indentation,
        len_var
    );
    let out_var = fname.clone();
    append!(
        ctx.out,
        "{}{} = new Array({});\n",
        ctx.indentation,
        fname,
        len_var
    );
    let idx_var = varname(&ctx.stack, "index");
    append!(
        ctx.out,
        "{}for (let {} = 0; {} < {}; ++{}) {{\n",
        ctx.indentation,
        idx_var,
        idx_var,
        len_var,
        idx_var
    );
    let item_var = varname(&ctx.stack, "item");
    let mut old_stack = Vec::new();
    ctx.swap_stack(&mut old_stack);
    ctx.push_fname(item_var);
    ctx.push_indent();

    match type_info {
        check::Builtin::String => {
            let len_var = varname(&ctx.stack, "len");
            append!(
                ctx.out,
                "{}let {} = reader.read_uint32();\n",
                ctx.indentation,
                len_var
            );
            append!(
                ctx.out,
                "{}{}[{}] = reader.read_string({});\n",
                ctx.indentation,
                out_var,
                idx_var,
                len_var
            );
        }
        _ => append!(
            ctx.out,
            "{}{}[{}] = reader.read_{}();\n",
            ctx.indentation,
            out_var,
            idx_var,
            type_name
        ),
    }

    ctx.swap_stack(&mut old_stack);
    ctx.pop_indent();
    append!(ctx.out, "{}}}\n", ctx.indentation);
}

fn gen_read_impl_builtin(ctx: &mut ImplCtx, type_info: &check::Builtin, type_name: &str) {
    match type_info {
        check::Builtin::String => {
            let len_var = varname(&ctx.stack, "len");
            append!(
                ctx.out,
                "{}let {} = reader.read_uint32();\n",
                ctx.indentation,
                len_var
            );
            append!(
                ctx.out,
                "{}{} = reader.read_string({});\n",
                ctx.indentation,
                fname(&ctx.stack),
                len_var
            );
        }
        _ => append!(
            ctx.out,
            "{}{} = reader.read_{}();\n",
            ctx.indentation,
            fname(&ctx.stack),
            type_name
        ),
    }
}

fn gen_read_impl_enum_array(ctx: &mut ImplCtx, type_info: &check::Enum, type_name: &str) {
    let len_var = varname(&ctx.stack, "len");
    let fname = fname(&ctx.stack);
    append!(
        ctx.out,
        "{}let {} = reader.read_uint32();\n",
        ctx.indentation,
        len_var
    );
    let out_var = fname.clone();
    append!(
        ctx.out,
        "{}{} = new Array({});\n",
        ctx.indentation,
        fname,
        len_var
    );
    let idx_var = varname(&ctx.stack, "index");
    append!(
        ctx.out,
        "{}for (let {} = 0; {} < {}; ++{}) {{\n",
        ctx.indentation,
        idx_var,
        idx_var,
        len_var,
        idx_var
    );
    let item_var = varname(&ctx.stack, "item");
    let mut old_stack = Vec::new();
    ctx.swap_stack(&mut old_stack);
    ctx.push_fname(item_var);
    ctx.push_indent();

    let repr_name = match type_info.repr {
        check::EnumRepr::U8 => "uint8",
        check::EnumRepr::U16 => "uint16",
        check::EnumRepr::U32 => "uint32",
    };
    append!(
        ctx.out,
        "{}{}[{}] = {}_try_from(reader.read_{}());\n",
        ctx.indentation,
        out_var,
        idx_var,
        type_name,
        repr_name
    );

    ctx.swap_stack(&mut old_stack);
    ctx.pop_indent();
    append!(ctx.out, "{}}}\n", ctx.indentation);
}

fn gen_read_impl_enum(ctx: &mut ImplCtx, type_info: &check::Enum, type_name: &str) {
    let repr_name = match type_info.repr {
        check::EnumRepr::U8 => "uint8",
        check::EnumRepr::U16 => "uint16",
        check::EnumRepr::U32 => "uint32",
    };
    append!(
        ctx.out,
        "{}{} = {}_try_from(reader.read_{}());\n",
        ctx.indentation,
        fname(&ctx.stack),
        type_name,
        repr_name
    );
}

fn gen_read_impl_struct_array(ctx: &mut ImplCtx, type_info: &check::Struct, _: &str) {
    let len_var = varname(&ctx.stack, "len");
    let fname = fname(&ctx.stack);
    append!(
        ctx.out,
        "{}let {} = reader.read_uint32();\n",
        ctx.indentation,
        len_var
    );
    append!(
        ctx.out,
        "{}{} = new Array({});\n",
        ctx.indentation,
        fname,
        len_var
    );
    let idx_var = varname(&ctx.stack, "index");
    append!(
        ctx.out,
        "{}for (let {} = 0; {} < {}; ++{}) {{\n",
        ctx.indentation,
        idx_var,
        idx_var,
        len_var,
        idx_var
    );
    let item_var = varname(&ctx.stack, "item");
    let mut old_stack = Vec::new();
    ctx.swap_stack(&mut old_stack);
    ctx.push_fname(item_var.clone());
    ctx.push_indent();

    append!(
        ctx.out,
        "{}let {}: any = {{}};\n",
        ctx.indentation,
        item_var
    );
    for field in &type_info.fields {
        ctx.push_fname(field.name.clone());
        let field_type = &*field.r#type.borrow();
        match &field_type.1 {
            check::ResolvedType::Builtin(field_type_info) if field.array => {
                gen_read_impl_builtin_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Builtin(field_type_info) => {
                gen_read_impl_builtin(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Enum(field_type_info) if field.array => {
                gen_read_impl_enum_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Enum(field_type_info) => {
                gen_read_impl_enum(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Struct(field_type_info) if field.array => {
                gen_read_impl_struct_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Struct(field_type_info) => {
                gen_read_impl_struct(ctx, &field_type_info, &field_type.0)
            }
        }
        ctx.pop_fname();
    }

    ctx.swap_stack(&mut old_stack);
    append!(
        ctx.out,
        "{}{}[{}] = {};\n",
        ctx.indentation,
        self::fname(&ctx.stack),
        idx_var,
        item_var
    );
    ctx.pop_indent();
    append!(ctx.out, "{}}}\n", ctx.indentation);
}

fn gen_read_impl_struct(ctx: &mut ImplCtx, type_info: &check::Struct, _: &str) {
    for field in &type_info.fields {
        ctx.push_fname(field.name.clone());
        let field_type = &*field.r#type.borrow();
        match &field_type.1 {
            check::ResolvedType::Builtin(field_type_info) if field.array => {
                gen_read_impl_builtin_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Builtin(field_type_info) => {
                gen_read_impl_builtin(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Enum(field_type_info) if field.array => {
                gen_read_impl_enum_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Enum(field_type_info) => {
                gen_read_impl_enum(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Struct(field_type_info) if field.array => {
                gen_read_impl_struct_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Struct(field_type_info) => {
                gen_read_impl_struct(ctx, &field_type_info, &field_type.0)
            }
        }
        ctx.pop_fname();
    }
}

impl ReadImpl<TypeScript> for check::Export {
    fn gen_read_impl(&self, _: &mut TypeScript, name: String, out: &mut String) {
        let mut ctx = ImplCtx::new(out);
        ctx.push_fname("output".to_string());
        append!(
            ctx.out,
            "export function read(reader: Reader, output: {}) {{\n",
            name
        );
        ctx.push_indent();
        gen_read_impl_struct(&mut ctx, &self.r#struct, &name);
        ctx.pop_indent();
        append!(ctx.out, "}}\n");
    }
}

impl Definition<TypeScript> for check::Struct {
    fn gen_def(&self, _: &mut TypeScript, name: String, out: &mut String) {
        append!(out, "export interface {} {{\n", name);
        for field in self.fields.iter() {
            let type_info = &*field.r#type.borrow();
            let typename: &str = match &type_info.1 {
                check::ResolvedType::Builtin(b) => match b {
                    check::Builtin::String => "string",
                    _ => "number",
                },
                _ => &type_info.0,
            };
            if field.array {
                append!(out, "    {}: {}[],\n", field.name, typename);
            } else {
                append!(out, "    {}: {},\n", field.name, typename);
            }
        }
        append!(out, "}}\n");
    }
}

fn gen_def_enum_tryfrom_impl(name: String, ty: &check::Enum, out: &mut String) {
    let mut indent = "".to_string();
    append!(
        out,
        "{}function {}_try_from(value: number): {} {{\n",
        indent,
        name,
        name
    );
    indent += "    ";
    // this will not panic, because enums are never empty
    let (min, max) = (&ty.variants[0], &ty.variants[ty.variants.len() - 1]);
    append!(
        out,
        "{}if ({}.{} <= value && value <= {}.{}) {{ return value; }}\n",
        indent,
        name,
        min.name,
        name,
        max.name
    );
    append!(
        out,
        "{}else throw new Error(`'${{value}}' is not a valid '{}' value`);\n",
        indent,
        name
    );
    indent.truncate(indent.len() - 4);
    append!(out, "}}\n");
}

impl Definition<TypeScript> for check::Enum {
    fn gen_def(&self, _: &mut TypeScript, name: String, out: &mut String) {
        append!(out, "export const enum {} {{\n", name);
        for variant in self.variants.iter() {
            append!(out, "    {} = 1 << {},\n", variant.name, variant.value);
        }
        append!(out, "}}\n");
        gen_def_enum_tryfrom_impl(name, &self, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commmon_gen() {
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_common();
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
import { Reader, Writer } from \"packet\";
"
        );
    }

    #[test]
    fn simple_struct_gen() {
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
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_def("Position".to_string(), &position);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
export interface Position {
    x: number,
    y: number,
}
"
        );
    }

    #[test]
    fn enum_gen() {
        use check::*;
        let flag = Enum {
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
        gen.push_def("Flag".to_string(), &flag);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
export const enum Flag {
    A = 1 << 0,
    B = 1 << 1,
}
function Flag_try_from(value: number): Flag {
    if (Flag.A <= value && value <= Flag.B) { return value; }
    else throw new Error(`'${value}' is not a valid 'Flag' value`);
}
"
        );
    }

    #[test]
    fn complex_struct_gen() {
        use check::*;
        let test = Export {
            name: "Test".to_string(),
            r#struct: Struct {
                fields: vec![
                    StructField {
                        name: "builtin_scalar".to_string(),
                        r#type: Ptr::new((
                            "uint8".to_string(),
                            ResolvedType::Builtin(Builtin::Uint8),
                        )),
                        array: false,
                    },
                    StructField {
                        name: "builtin_array".to_string(),
                        r#type: Ptr::new((
                            "uint8".to_string(),
                            ResolvedType::Builtin(Builtin::Uint8),
                        )),
                        array: true,
                    },
                    StructField {
                        name: "string_scalar".to_string(),
                        r#type: Ptr::new((
                            "string".to_string(),
                            ResolvedType::Builtin(Builtin::String),
                        )),
                        array: false,
                    },
                    StructField {
                        name: "string_array".to_string(),
                        r#type: Ptr::new((
                            "string".to_string(),
                            ResolvedType::Builtin(Builtin::String),
                        )),
                        array: true,
                    },
                    StructField {
                        name: "enum_scalar".to_string(),
                        r#type: Ptr::new((
                            "Flag".to_string(),
                            ResolvedType::Enum(Enum {
                                repr: EnumRepr::U8,
                                variants: vec![],
                            }),
                        )),
                        array: false,
                    },
                    StructField {
                        name: "enum_array".to_string(),
                        r#type: Ptr::new((
                            "Flag".to_string(),
                            ResolvedType::Enum(Enum {
                                repr: EnumRepr::U8,
                                variants: vec![],
                            }),
                        )),
                        array: true,
                    },
                    StructField {
                        name: "struct_scalar".to_string(),
                        r#type: Ptr::new((
                            "Position".to_string(),
                            ResolvedType::Struct(Struct { fields: vec![] }),
                        )),
                        array: false,
                    },
                    StructField {
                        name: "struct_array".to_string(),
                        r#type: Ptr::new((
                            "Position".to_string(),
                            ResolvedType::Struct(Struct { fields: vec![] }),
                        )),
                        array: true,
                    },
                ],
            },
        };
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_def("Test".to_string(), &test.r#struct);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
export interface Test {
    builtin_scalar: number,
    builtin_array: number[],
    string_scalar: string,
    string_array: string[],
    enum_scalar: Flag,
    enum_array: Flag[],
    struct_scalar: Position,
    struct_array: Position[],
}
"
        );
    }

    #[test]
    fn nested_soa_write_gen() {
        use check::*;
        let test_a = Struct {
            fields: vec![
                StructField {
                    name: "first".to_string(),
                    r#type: Ptr::new(("uint8".to_string(), ResolvedType::Builtin(Builtin::Uint8))),
                    array: true,
                },
                StructField {
                    name: "second".to_string(),
                    r#type: Ptr::new(("uint8".to_string(), ResolvedType::Builtin(Builtin::Uint8))),
                    array: true,
                },
            ],
        };
        let test_b = Export {
            name: "TestB".to_string(),
            r#struct: Struct {
                fields: vec![StructField {
                    name: "test_a".to_string(),
                    r#type: Ptr::new(("TestA".to_string(), ResolvedType::Struct(test_a))),
                    array: true,
                }],
            },
        };
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_write_impl("TestB".to_string(), &test_b);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
export function write(writer: Writer, input: TestB) {
    writer.write_uint32(input.test_a.length);
    for (let input_test_a_item of input.test_a) {
        writer.write_uint32(input_test_a_item.first.length);
        for (let input_test_a_item_first_item of input_test_a_item.first) {
            writer.write_uint8(input_test_a_item_first_item);
        }
        writer.write_uint32(input_test_a_item.second.length);
        for (let input_test_a_item_second_item of input_test_a_item.second) {
            writer.write_uint8(input_test_a_item_second_item);
        }
    }
}
"
        );
    }

    #[test]
    fn nested_soa_read_gen() {
        use check::*;
        let test_a = Struct {
            fields: vec![
                StructField {
                    name: "first".to_string(),
                    r#type: Ptr::new(("uint8".to_string(), ResolvedType::Builtin(Builtin::Uint8))),
                    array: true,
                },
                StructField {
                    name: "second".to_string(),
                    r#type: Ptr::new(("uint8".to_string(), ResolvedType::Builtin(Builtin::Uint8))),
                    array: true,
                },
            ],
        };
        let test_b = Export {
            name: "TestB".to_string(),
            r#struct: Struct {
                fields: vec![StructField {
                    name: "test_a".to_string(),
                    r#type: Ptr::new(("TestA".to_string(), ResolvedType::Struct(test_a))),
                    array: true,
                }],
            },
        };
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_read_impl("TestB".to_string(), &test_b);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
export function read(reader: Reader, output: TestB) {
    let output_test_a_len = reader.read_uint32();
    output.test_a = new Array(output_test_a_len);
    for (let output_test_a_index = 0; output_test_a_index < output_test_a_len; ++output_test_a_index) {
        let output_test_a_item: any = {};
        let output_test_a_item_first_len = reader.read_uint32();
        output_test_a_item.first = new Array(output_test_a_item_first_len);
        for (let output_test_a_item_first_index = 0; output_test_a_item_first_index < output_test_a_item_first_len; ++output_test_a_item_first_index) {
            output_test_a_item.first[output_test_a_item_first_index] = reader.read_uint8();
        }
        let output_test_a_item_second_len = reader.read_uint32();
        output_test_a_item.second = new Array(output_test_a_item_second_len);
        for (let output_test_a_item_second_index = 0; output_test_a_item_second_index < output_test_a_item_second_len; ++output_test_a_item_second_index) {
            output_test_a_item.second[output_test_a_item_second_index] = reader.read_uint8();
        }
        output.test_a[output_test_a_index] = output_test_a_item;
    }
}
"
        );
    }

    #[test]
    fn complex_struct_write_gen() {
        use check::*;
        let flag = Enum {
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
        let test = Export {
            name: "Test".to_string(),
            r#struct: Struct {
                fields: vec![
                    StructField {
                        name: "builtin_scalar".to_string(),
                        r#type: Ptr::new((
                            "uint8".to_string(),
                            ResolvedType::Builtin(Builtin::Uint8),
                        )),
                        array: false,
                    },
                    StructField {
                        name: "builtin_array".to_string(),
                        r#type: Ptr::new((
                            "uint8".to_string(),
                            ResolvedType::Builtin(Builtin::Uint8),
                        )),
                        array: true,
                    },
                    StructField {
                        name: "string_scalar".to_string(),
                        r#type: Ptr::new((
                            "string".to_string(),
                            ResolvedType::Builtin(Builtin::String),
                        )),
                        array: false,
                    },
                    StructField {
                        name: "string_array".to_string(),
                        r#type: Ptr::new((
                            "string".to_string(),
                            ResolvedType::Builtin(Builtin::String),
                        )),
                        array: true,
                    },
                    StructField {
                        name: "enum_scalar".to_string(),
                        r#type: Ptr::new(("Flag".to_string(), ResolvedType::Enum(flag.clone()))),
                        array: false,
                    },
                    StructField {
                        name: "enum_array".to_string(),
                        r#type: Ptr::new(("Flag".to_string(), ResolvedType::Enum(flag))),
                        array: true,
                    },
                    StructField {
                        name: "struct_scalar".to_string(),
                        r#type: Ptr::new((
                            "Position".to_string(),
                            ResolvedType::Struct(position.clone()),
                        )),
                        array: false,
                    },
                    StructField {
                        name: "struct_array".to_string(),
                        r#type: Ptr::new(("Position".to_string(), ResolvedType::Struct(position))),
                        array: true,
                    },
                ],
            },
        };
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_write_impl("Test".to_string(), &test);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
export function write(writer: Writer, input: Test) {
    writer.write_uint8(input.builtin_scalar);
    writer.write_uint32(input.builtin_array.length);
    for (let input_builtin_array_item of input.builtin_array) {
        writer.write_uint8(input_builtin_array_item);
    }
    writer.write_uint32(input.string_scalar.length);
    writer.write_string(input.string_scalar);
    writer.write_uint32(input.string_array.length);
    for (let input_string_array_item of input.string_array) {
        writer.write_uint32(input_string_array_item.length);
        writer.write_string(input_string_array_item);
    }
    writer.write_uint8(input.enum_scalar as number);
    writer.write_uint32(input.enum_array.length);
    for (let input_enum_array_item of input.enum_array) {
        writer.write_uint8(input_enum_array_item as number);
    }
    writer.write_float(input.struct_scalar.x);
    writer.write_float(input.struct_scalar.y);
    writer.write_uint32(input.struct_array.length);
    for (let input_struct_array_item of input.struct_array) {
        writer.write_float(input_struct_array_item.x);
        writer.write_float(input_struct_array_item.y);
    }
}
"
        );
    }

    #[test]
    fn complex_struct_read_gen() {
        use check::*;
        let flag = Enum {
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
        let test = Export {
            name: "Test".to_string(),
            r#struct: Struct {
                fields: vec![
                    StructField {
                        name: "builtin_scalar".to_string(),
                        r#type: Ptr::new((
                            "uint8".to_string(),
                            ResolvedType::Builtin(Builtin::Uint8),
                        )),
                        array: false,
                    },
                    StructField {
                        name: "builtin_array".to_string(),
                        r#type: Ptr::new((
                            "uint8".to_string(),
                            ResolvedType::Builtin(Builtin::Uint8),
                        )),
                        array: true,
                    },
                    StructField {
                        name: "string_scalar".to_string(),
                        r#type: Ptr::new((
                            "string".to_string(),
                            ResolvedType::Builtin(Builtin::String),
                        )),
                        array: false,
                    },
                    StructField {
                        name: "string_array".to_string(),
                        r#type: Ptr::new((
                            "string".to_string(),
                            ResolvedType::Builtin(Builtin::String),
                        )),
                        array: true,
                    },
                    StructField {
                        name: "enum_scalar".to_string(),
                        r#type: Ptr::new(("Flag".to_string(), ResolvedType::Enum(flag.clone()))),
                        array: false,
                    },
                    StructField {
                        name: "enum_array".to_string(),
                        r#type: Ptr::new(("Flag".to_string(), ResolvedType::Enum(flag))),
                        array: true,
                    },
                    StructField {
                        name: "struct_scalar".to_string(),
                        r#type: Ptr::new((
                            "Position".to_string(),
                            ResolvedType::Struct(position.clone()),
                        )),
                        array: false,
                    },
                    StructField {
                        name: "struct_array".to_string(),
                        r#type: Ptr::new(("Position".to_string(), ResolvedType::Struct(position))),
                        array: true,
                    },
                ],
            },
        };
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_read_impl("Test".to_string(), &test);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
export function read(reader: Reader, output: Test) {
    output.builtin_scalar = reader.read_uint8();
    let output_builtin_array_len = reader.read_uint32();
    output.builtin_array = new Array(output_builtin_array_len);
    for (let output_builtin_array_index = 0; output_builtin_array_index < output_builtin_array_len; ++output_builtin_array_index) {
        output.builtin_array[output_builtin_array_index] = reader.read_uint8();
    }
    let output_string_scalar_len = reader.read_uint32();
    output.string_scalar = reader.read_string(output_string_scalar_len);
    let output_string_array_len = reader.read_uint32();
    output.string_array = new Array(output_string_array_len);
    for (let output_string_array_index = 0; output_string_array_index < output_string_array_len; ++output_string_array_index) {
        let output_string_array_item_len = reader.read_uint32();
        output.string_array[output_string_array_index] = reader.read_string(output_string_array_item_len);
    }
    output.enum_scalar = Flag_try_from(reader.read_uint8());
    let output_enum_array_len = reader.read_uint32();
    output.enum_array = new Array(output_enum_array_len);
    for (let output_enum_array_index = 0; output_enum_array_index < output_enum_array_len; ++output_enum_array_index) {
        output.enum_array[output_enum_array_index] = Flag_try_from(reader.read_uint8());
    }
    output.struct_scalar.x = reader.read_float();
    output.struct_scalar.y = reader.read_float();
    let output_struct_array_len = reader.read_uint32();
    output.struct_array = new Array(output_struct_array_len);
    for (let output_struct_array_index = 0; output_struct_array_index < output_struct_array_len; ++output_struct_array_index) {
        let output_struct_array_item: any = {};
        output_struct_array_item.x = reader.read_float();
        output_struct_array_item.y = reader.read_float();
        output.struct_array[output_struct_array_index] = output_struct_array_item;
    }
}
"
        );
    }
}
