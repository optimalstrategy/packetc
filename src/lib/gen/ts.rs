use super::*;
use fstrings::format_args_f;
use std::collections::HashSet;

// TODO: use (proc?) macros or some template syntax to clean up all of this duplicated code.
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
            indentation: String::new(),
            out,
            stack: Vec::new(),
        }
    }

    #[inline]
    fn push_indent(&mut self) {
        self.indentation += "    ";
    }

    #[inline]
    fn pop_indent(&mut self) {
        self.indentation.truncate(if self.indentation.len() < 4 {
            0
        } else {
            self.indentation.len() - 4
        });
    }

    #[inline]
    fn push_fname<S: Into<String>>(&mut self, name: S) {
        self.stack.push(name.into());
    }

    #[inline]
    fn pop_fname(&mut self) {
        self.stack.pop();
    }

    #[inline]
    fn swap_stack(&mut self, other: &mut Vec<String>) {
        std::mem::swap(&mut self.stack, other);
    }
}

fn varname(stack: &[String], name: &str) -> String {
    format!("{}_{}", stack.join("_"), name)
}

fn bindname(stack: &[String]) -> String {
    stack.join("_")
}

fn fname(stack: &[String]) -> String {
    stack.join(".")
}

fn gen_write_impl_builtin_array(ctx: &mut ImplCtx, type_info: &check::Builtin, type_name: &str) {
    let fname = fname(&ctx.stack);
    append!(
        ctx.out,
        "{ctx.indentation}writer.write_uint32({fname}.length);\n"
    );
    let item_var = varname(&ctx.stack, "item");
    // TODO: use index-based for loop instead
    append!(
        ctx.out,
        "{ctx.indentation}for (let {item_var} of {fname}) {{\n"
    );
    let mut old_stack = Vec::new();
    ctx.swap_stack(&mut old_stack);
    ctx.push_fname(item_var.clone());
    ctx.push_indent();

    match type_info {
        check::Builtin::String => {
            append!(
                ctx.out,
                "{ctx.indentation}writer.write_uint32({item_var}.length);\n"
            );
            append!(
                ctx.out,
                "{ctx.indentation}writer.write_string({item_var});\n"
            );
        }
        _ => append!(
            ctx.out,
            "{ctx.indentation}writer.write_{type_name}({item_var});\n"
        ),
    }

    ctx.swap_stack(&mut old_stack);
    ctx.pop_indent();
    append!(ctx.out, "{ctx.indentation}}}\n");
}

fn gen_write_impl_builtin(ctx: &mut ImplCtx, type_info: &check::Builtin, type_name: &str) {
    let fname = fname(&ctx.stack);
    match type_info {
        check::Builtin::String => {
            append!(
                ctx.out,
                "{ctx.indentation}writer.write_uint32({fname}.length);\n"
            );
            append!(ctx.out, "{ctx.indentation}writer.write_string({fname});\n");
        }
        _ => append!(
            ctx.out,
            "{ctx.indentation}writer.write_{type_name}({fname});\n"
        ),
    }
}

fn gen_write_impl_enum_array(ctx: &mut ImplCtx, type_info: &check::Enum, _: &str) {
    let fname = fname(&ctx.stack);
    append!(
        ctx.out,
        "{ctx.indentation}writer.write_uint32({fname}.length);\n"
    );
    let item_var = varname(&ctx.stack, "item");
    // TODO: use index-based for loop instead
    append!(
        ctx.out,
        "{ctx.indentation}for (let {item_var} of {fname}) {{\n"
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
    let ifname = self::fname(&ctx.stack);
    append!(
        ctx.out,
        "{ctx.indentation}writer.write_{repr_name}({ifname} as number);\n"
    );

    ctx.swap_stack(&mut old_stack);
    ctx.pop_indent();
    append!(ctx.out, "{ctx.indentation}}}\n");
}

fn gen_write_impl_enum(ctx: &mut ImplCtx, type_info: &check::Enum, _: &str) {
    let repr_name = match &type_info.repr {
        check::EnumRepr::U8 => "uint8",
        check::EnumRepr::U16 => "uint16",
        check::EnumRepr::U32 => "uint32",
    };
    let fname = fname(&ctx.stack);
    append!(
        ctx.out,
        "{ctx.indentation}writer.write_{repr_name}({fname} as number);\n"
    );
}

fn gen_write_impl_struct_array(ctx: &mut ImplCtx, type_info: &check::Struct, _: &str) {
    let fname = fname(&ctx.stack);
    append!(
        ctx.out,
        "{ctx.indentation}writer.write_uint32({fname}.length);\n"
    );
    let item_var = varname(&ctx.stack, "item");
    // TODO: use index-based for loop instead
    append!(
        ctx.out,
        "{ctx.indentation}for (let {item_var} of {fname}) {{\n"
    );
    let mut old_stack = Vec::new();
    ctx.swap_stack(&mut old_stack);
    ctx.push_fname(item_var);
    ctx.push_indent();

    for field in &type_info.fields {
        ctx.push_fname(field.name);
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
    append!(ctx.out, "{ctx.indentation}}}\n");
}

fn gen_write_impl_struct(ctx: &mut ImplCtx, type_info: &check::Struct, _: &str) {
    for field in &type_info.fields {
        ctx.push_fname(field.name);
        let mut old_stack = if field.optional {
            let fname = fname(&ctx.stack);
            let bind_var = bindname(&ctx.stack);
            append!(ctx.out, "{ctx.indentation}let {bind_var} = {fname};\n");
            append!(ctx.out, "{ctx.indentation}switch ({bind_var}) {{\n");
            ctx.push_indent();
            append!(
                ctx.out,
                "{ctx.indentation}case undefined: case null: writer.write_uint8(0); break;\n"
            );
            append!(ctx.out, "{ctx.indentation}default: {{\n");
            ctx.push_indent();
            append!(ctx.out, "{ctx.indentation}writer.write_uint8(1);\n");

            let mut old_stack = Vec::new();
            ctx.swap_stack(&mut old_stack);
            ctx.push_fname(bind_var);

            Some(old_stack)
        } else {
            None
        };

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

        if old_stack.is_some() {
            ctx.swap_stack(old_stack.as_mut().unwrap());

            ctx.pop_indent();
            append!(ctx.out, "{ctx.indentation}}}\n");
            ctx.pop_indent();
            append!(ctx.out, "{ctx.indentation}}}\n");
        }
        ctx.pop_fname();
    }
}

impl<'a> WriteImpl<TypeScript> for check::Export<'a> {
    fn gen_write_impl(&self, _: &mut TypeScript, name: &str, out: &mut String) {
        let mut ctx = ImplCtx::new(out);
        ctx.push_fname("input");
        append!(
            ctx.out,
            "export function write(writer: Writer, input: {name}) {{\n"
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
        "{ctx.indentation}let {len_var} = reader.read_uint32();\n"
    );
    let out_var = fname.clone();
    append!(
        ctx.out,
        "{ctx.indentation}{fname} = new Array({len_var});\n"
    );
    let idx_var = varname(&ctx.stack, "index");
    append!(
        ctx.out,
        "{ctx.indentation}for (let {idx_var} = 0; {idx_var} < {len_var}; ++{idx_var}) {{\n"
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
                "{ctx.indentation}let {len_var} = reader.read_uint32();\n"
            );
            append!(
                ctx.out,
                "{ctx.indentation}{out_var}[{idx_var}] = reader.read_string({len_var});\n"
            );
        }
        _ => append!(
            ctx.out,
            "{ctx.indentation}{out_var}[{idx_var}] = reader.read_{type_name}();\n"
        ),
    }

    ctx.swap_stack(&mut old_stack);
    ctx.pop_indent();
    append!(ctx.out, "{ctx.indentation}}}\n");
}

fn gen_read_impl_builtin(
    ctx: &mut ImplCtx,
    type_info: &check::Builtin,
    type_name: &str,
    optional: bool,
) {
    if optional {
        append!(
            ctx.out,
            "{ctx.indentation}if (reader.read_uint8() > 0) {{\n"
        );
        ctx.push_indent();
    }
    match type_info {
        check::Builtin::String => {
            let len_var = varname(&ctx.stack, "len");
            append!(
                ctx.out,
                "{ctx.indentation}let {len_var} = reader.read_uint32();\n"
            );
            let fname = fname(&ctx.stack);
            append!(
                ctx.out,
                "{ctx.indentation}{fname} = reader.read_string({len_var});\n"
            );
        }
        _ => {
            let fname = fname(&ctx.stack);
            append!(
                ctx.out,
                "{ctx.indentation}{fname} = reader.read_{type_name}();\n"
            )
        }
    }
    if optional {
        ctx.pop_indent();
        append!(ctx.out, "{ctx.indentation}}}\n");
    }
}

fn gen_read_impl_enum_array(ctx: &mut ImplCtx, type_info: &check::Enum, type_name: &str) {
    let len_var = varname(&ctx.stack, "len");
    let fname = fname(&ctx.stack);
    append!(
        ctx.out,
        "{ctx.indentation}let {len_var} = reader.read_uint32();\n"
    );
    let out_var = fname.clone();
    append!(
        ctx.out,
        "{ctx.indentation}{fname} = new Array({len_var});\n"
    );
    let idx_var = varname(&ctx.stack, "index");
    append!(
        ctx.out,
        "{ctx.indentation}for (let {idx_var} = 0; {idx_var} < {len_var}; ++{idx_var}) {{\n"
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
        "{ctx.indentation}{out_var}[{idx_var}] = {type_name}_try_from(reader.read_{repr_name}());\n"
    );

    ctx.swap_stack(&mut old_stack);
    ctx.pop_indent();
    append!(ctx.out, "{ctx.indentation}}}\n");
}

fn gen_read_impl_enum(ctx: &mut ImplCtx, type_info: &check::Enum, type_name: &str, optional: bool) {
    if optional {
        append!(
            ctx.out,
            "{ctx.indentation}if (reader.read_uint8() > 0) {{\n"
        );
        ctx.push_indent();
    }

    let repr_name = match type_info.repr {
        check::EnumRepr::U8 => "uint8",
        check::EnumRepr::U16 => "uint16",
        check::EnumRepr::U32 => "uint32",
    };
    let fname = fname(&ctx.stack);
    append!(
        ctx.out,
        "{ctx.indentation}{fname} = {type_name}_try_from(reader.read_{repr_name}());\n"
    );

    if optional {
        ctx.pop_indent();
        append!(ctx.out, "{ctx.indentation}}}\n");
    }
}

fn gen_read_impl_struct_array(ctx: &mut ImplCtx, type_info: &check::Struct, _: &str) {
    let len_var = varname(&ctx.stack, "len");
    let fname = fname(&ctx.stack);
    append!(
        ctx.out,
        "{ctx.indentation}let {len_var} = reader.read_uint32();\n"
    );
    append!(
        ctx.out,
        "{ctx.indentation}{fname} = new Array({len_var});\n"
    );
    let idx_var = varname(&ctx.stack, "index");
    append!(
        ctx.out,
        "{ctx.indentation}for (let {idx_var} = 0; {idx_var} < {len_var}; ++{idx_var}) {{\n"
    );
    let item_var = varname(&ctx.stack, "item");
    let mut old_stack = Vec::new();
    ctx.swap_stack(&mut old_stack);
    ctx.push_fname(item_var.clone());
    ctx.push_indent();

    append!(ctx.out, "{ctx.indentation}let {item_var}: any = {{}};\n");
    for field in &type_info.fields {
        ctx.push_fname(field.name);
        let field_type = &*field.r#type.borrow();
        match &field_type.1 {
            check::ResolvedType::Builtin(field_type_info) if field.array => {
                gen_read_impl_builtin_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Builtin(field_type_info) => {
                gen_read_impl_builtin(ctx, &field_type_info, &field_type.0, field.optional)
            }
            check::ResolvedType::Enum(field_type_info) if field.array => {
                gen_read_impl_enum_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Enum(field_type_info) => {
                gen_read_impl_enum(ctx, &field_type_info, &field_type.0, field.optional)
            }
            check::ResolvedType::Struct(field_type_info) if field.array => {
                gen_read_impl_struct_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Struct(field_type_info) => {
                gen_read_impl_struct(ctx, &field_type_info, &field_type.0, field.optional)
            }
        }
        ctx.pop_fname();
    }

    ctx.swap_stack(&mut old_stack);
    let ifname = self::fname(&ctx.stack);
    append!(
        ctx.out,
        "{ctx.indentation}{ifname}[{idx_var}] = {item_var};\n"
    );
    ctx.pop_indent();
    append!(ctx.out, "{ctx.indentation}}}\n");
}

fn gen_read_impl_struct(
    ctx: &mut ImplCtx,
    type_info: &check::Struct,
    type_name: &str,
    optional: bool,
) {
    let (old_stack, fname, bind_var) = if optional {
        let fname = fname(&ctx.stack);
        let bind_var = bindname(&ctx.stack);
        append!(
            ctx.out,
            "{ctx.indentation}if (reader.read_uint8() > 0) {{\n"
        );
        ctx.push_indent();
        append!(
            ctx.out,
            "{ctx.indentation}let {bind_var} = {{}} as unknown as {type_name};\n"
        );

        let mut old_stack = Vec::new();
        ctx.swap_stack(&mut old_stack);
        ctx.push_fname(&bind_var);

        (Some(old_stack), fname, bind_var)
    } else {
        (None, String::new(), String::new())
    };

    for field in &type_info.fields {
        ctx.push_fname(field.name);
        let field_type = &*field.r#type.borrow();
        match &field_type.1 {
            check::ResolvedType::Builtin(field_type_info) if field.array => {
                gen_read_impl_builtin_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Builtin(field_type_info) => {
                gen_read_impl_builtin(ctx, &field_type_info, &field_type.0, field.optional)
            }
            check::ResolvedType::Enum(field_type_info) if field.array => {
                gen_read_impl_enum_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Enum(field_type_info) => {
                gen_read_impl_enum(ctx, &field_type_info, &field_type.0, field.optional)
            }
            check::ResolvedType::Struct(field_type_info) if field.array => {
                gen_read_impl_struct_array(ctx, &field_type_info, &field_type.0)
            }
            check::ResolvedType::Struct(field_type_info) => {
                gen_read_impl_struct(ctx, &field_type_info, &field_type.0, field.optional)
            }
        }
        ctx.pop_fname();
    }
    if let Some(mut old_stack) = old_stack {
        append!(ctx.out, "{ctx.indentation}{fname} = {bind_var};\n");

        ctx.swap_stack(&mut old_stack);
        ctx.pop_indent();
        append!(ctx.out, "{ctx.indentation}}}\n");
    }
}

impl<'a> ReadImpl<TypeScript> for check::Export<'a> {
    fn gen_read_impl(&self, _: &mut TypeScript, name: &str, out: &mut String) {
        let mut ctx = ImplCtx::new(out);
        ctx.push_fname("output");
        append!(
            ctx.out,
            "export function read(reader: Reader, output: {name}) {{\n"
        );
        ctx.push_indent();
        gen_read_impl_struct(&mut ctx, &self.r#struct, &name, false);
        ctx.pop_indent();
        append!(ctx.out, "}}\n");
    }
}

impl<'a> Definition<TypeScript> for check::Struct<'a> {
    fn gen_def(&self, _: &mut TypeScript, name: &str, out: &mut String) {
        append!(out, "export interface {name} {{\n");
        for field in self.fields.iter() {
            let type_info = &*field.r#type.borrow();
            let typename: &str = match &type_info.1 {
                check::ResolvedType::Builtin(b) => match b {
                    check::Builtin::String => "string",
                    _ => "number",
                },
                _ => &type_info.0,
            };
            let opt = if field.optional { "?" } else { "" };
            let arr = if field.array { "[]" } else { "" };
            append!(out, "    {field.name}{opt}: {typename}{arr},\n");
        }
        append!(out, "}}\n");
    }
}

fn gen_def_enum_tryfrom_impl<'a>(name: &str, ty: &check::Enum<'a>, out: &mut String) {
    let mut indent = String::new();
    append!(
        out,
        "{indent}function {name}_try_from(value: number): {name} {{\n"
    );
    indent += "    ";
    // this will not panic, because enums are never empty
    let (min, max) = (&ty.variants[0], &ty.variants[ty.variants.len() - 1]);
    append!(
        out,
        "{indent}if ({name}.{min.name} <= value && value <= {name}.{max.name}) {{ return value; }}\n"
    );
    append!(
        out,
        "{indent}else throw new Error(`'${{value}}' is not a valid '{name}' value`);\n"
    );
    indent.truncate(indent.len() - 4);
    append!(out, "}}\n");
}

impl<'a> Definition<TypeScript> for check::Enum<'a> {
    fn gen_def(&self, _: &mut TypeScript, name: &str, out: &mut String) {
        append!(out, "export const enum {name} {{\n");
        for variant in self.variants.iter() {
            append!(out, "    {variant.name} = 1 << {variant.value},\n");
        }
        append!(out, "}}\n");
        gen_def_enum_tryfrom_impl(name, &self, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

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
                    name: "x",
                    r#type: Ptr::new(("float", ResolvedType::Builtin(Builtin::Float))),
                    array: false,
                    optional: false,
                },
                StructField {
                    name: "y",
                    r#type: Ptr::new(("float", ResolvedType::Builtin(Builtin::Float))),
                    array: false,
                    optional: false,
                },
            ],
        };
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_def("Position", &position);
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
    fn struct_with_optional_gen() {
        use check::*;
        let test = Struct {
            fields: vec![
                StructField {
                    name: "a",
                    r#type: Ptr::new(("float", ResolvedType::Builtin(Builtin::Float))),
                    array: false,
                    optional: true,
                },
                StructField {
                    name: "b",
                    r#type: Ptr::new(("float", ResolvedType::Builtin(Builtin::Float))),
                    array: true,
                    optional: true,
                },
                StructField {
                    name: "c",
                    r#type: Ptr::new(("float", ResolvedType::Builtin(Builtin::Float))),
                    array: false,
                    optional: false,
                },
            ],
        };
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_def("Test", &test);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
export interface Test {
    a?: number,
    b?: number[],
    c: number,
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
                    name: "A",
                    value: 0,
                },
                EnumVariant {
                    name: "B",
                    value: 1,
                },
            ],
        };
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_def("Flag", &flag);
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
            name: "Test",
            r#struct: Struct {
                fields: vec![
                    StructField {
                        name: "builtin_scalar",
                        r#type: Ptr::new(("uint8", ResolvedType::Builtin(Builtin::Uint8))),
                        array: false,
                        optional: false,
                    },
                    StructField {
                        name: "builtin_array",
                        r#type: Ptr::new(("uint8", ResolvedType::Builtin(Builtin::Uint8))),
                        array: true,
                        optional: false,
                    },
                    StructField {
                        name: "string_scalar",
                        r#type: Ptr::new(("string", ResolvedType::Builtin(Builtin::String))),
                        array: false,
                        optional: false,
                    },
                    StructField {
                        name: "string_array",
                        r#type: Ptr::new(("string", ResolvedType::Builtin(Builtin::String))),
                        array: true,
                        optional: false,
                    },
                    StructField {
                        name: "enum_scalar",
                        r#type: Ptr::new((
                            "Flag",
                            ResolvedType::Enum(Enum {
                                repr: EnumRepr::U8,
                                variants: vec![],
                            }),
                        )),
                        array: false,
                        optional: false,
                    },
                    StructField {
                        name: "enum_array",
                        r#type: Ptr::new((
                            "Flag",
                            ResolvedType::Enum(Enum {
                                repr: EnumRepr::U8,
                                variants: vec![],
                            }),
                        )),
                        array: true,
                        optional: false,
                    },
                    StructField {
                        name: "struct_scalar",
                        r#type: Ptr::new((
                            "Position",
                            ResolvedType::Struct(Struct { fields: vec![] }),
                        )),
                        array: false,
                        optional: false,
                    },
                    StructField {
                        name: "struct_array",
                        r#type: Ptr::new((
                            "Position",
                            ResolvedType::Struct(Struct { fields: vec![] }),
                        )),
                        array: true,
                        optional: false,
                    },
                ],
            },
        };
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_def("Test", &test.r#struct);
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
    fn optional_write_gen() {
        use check::*;
        let test = Export {
            name: "Test",
            r#struct: Struct {
                fields: vec![
                    StructField {
                        name: "a",
                        r#type: Ptr::new(("uint8", ResolvedType::Builtin(Builtin::Uint8))),
                        array: false,
                        optional: true,
                    },
                    StructField {
                        name: "b",
                        r#type: Ptr::new(("uint8", ResolvedType::Builtin(Builtin::Uint8))),
                        array: true,
                        optional: true,
                    },
                    StructField {
                        name: "c",
                        r#type: Ptr::new(("uint8", ResolvedType::Builtin(Builtin::Uint8))),
                        array: false,
                        optional: false,
                    },
                ],
            },
        };
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_write_impl("Test", &test);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
export function write(writer: Writer, input: Test) {
    let input_a = input.a;
    switch (input_a) {
        case undefined: case null: writer.write_uint8(0); break;
        default: {
            writer.write_uint8(1);
            writer.write_uint8(input_a);
        }
    }
    let input_b = input.b;
    switch (input_b) {
        case undefined: case null: writer.write_uint8(0); break;
        default: {
            writer.write_uint8(1);
            writer.write_uint32(input_b.length);
            for (let input_b_item of input_b) {
                writer.write_uint8(input_b_item);
            }
        }
    }
    writer.write_uint8(input.c);
}
"
        );
    }

    #[test]
    fn optional_read_gen() {
        use check::*;
        let test = Export {
            name: "Test",
            r#struct: Struct {
                fields: vec![
                    StructField {
                        name: "a",
                        r#type: Ptr::new(("uint8", ResolvedType::Builtin(Builtin::Uint8))),
                        array: false,
                        optional: true,
                    },
                    StructField {
                        name: "b",
                        r#type: Ptr::new(("uint8", ResolvedType::Builtin(Builtin::Uint8))),
                        array: false,
                        optional: false,
                    },
                ],
            },
        };
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_read_impl("Test", &test);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
export function read(reader: Reader, output: Test) {
    if (reader.read_uint8() > 0) {
        output.a = reader.read_uint8();
    }
    output.b = reader.read_uint8();
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
                    name: "first",
                    r#type: Ptr::new(("uint8", ResolvedType::Builtin(Builtin::Uint8))),
                    array: true,
                    optional: false,
                },
                StructField {
                    name: "second",
                    r#type: Ptr::new(("uint8", ResolvedType::Builtin(Builtin::Uint8))),
                    array: true,
                    optional: false,
                },
            ],
        };
        let test_b = Export {
            name: "TestB",
            r#struct: Struct {
                fields: vec![StructField {
                    name: "test_a",
                    r#type: Ptr::new(("TestA", ResolvedType::Struct(test_a))),
                    array: true,
                    optional: false,
                }],
            },
        };
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_write_impl("TestB", &test_b);
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
                    name: "first",
                    r#type: Ptr::new(("uint8", ResolvedType::Builtin(Builtin::Uint8))),
                    array: true,
                    optional: false,
                },
                StructField {
                    name: "second",
                    r#type: Ptr::new(("uint8", ResolvedType::Builtin(Builtin::Uint8))),
                    array: true,
                    optional: false,
                },
            ],
        };
        let test_b = Export {
            name: "TestB",
            r#struct: Struct {
                fields: vec![StructField {
                    name: "test_a",
                    r#type: Ptr::new(("TestA", ResolvedType::Struct(test_a))),
                    array: true,
                    optional: false,
                }],
            },
        };
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_read_impl("TestB", &test_b);
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
                    name: "A",
                    value: 0,
                },
                EnumVariant {
                    name: "B",
                    value: 1,
                },
            ],
        };
        let position = Struct {
            fields: vec![
                StructField {
                    name: "x",
                    r#type: Ptr::new(("float", ResolvedType::Builtin(Builtin::Float))),
                    array: false,
                    optional: false,
                },
                StructField {
                    name: "y",
                    r#type: Ptr::new(("float", ResolvedType::Builtin(Builtin::Float))),
                    array: false,
                    optional: false,
                },
            ],
        };
        let test = Export {
            name: "Test",
            r#struct: Struct {
                fields: vec![
                    StructField {
                        name: "builtin_scalar",
                        r#type: Ptr::new(("uint8", ResolvedType::Builtin(Builtin::Uint8))),
                        array: false,
                        optional: false,
                    },
                    StructField {
                        name: "builtin_array",
                        r#type: Ptr::new(("uint8", ResolvedType::Builtin(Builtin::Uint8))),
                        array: true,
                        optional: false,
                    },
                    StructField {
                        name: "string_scalar",
                        r#type: Ptr::new(("string", ResolvedType::Builtin(Builtin::String))),
                        array: false,
                        optional: false,
                    },
                    StructField {
                        name: "string_array",
                        r#type: Ptr::new(("string", ResolvedType::Builtin(Builtin::String))),
                        array: true,
                        optional: false,
                    },
                    StructField {
                        name: "enum_scalar",
                        r#type: Ptr::new(("Flag", ResolvedType::Enum(flag.clone()))),
                        array: false,
                        optional: false,
                    },
                    StructField {
                        name: "enum_array",
                        r#type: Ptr::new(("Flag", ResolvedType::Enum(flag.clone()))),
                        array: true,
                        optional: false,
                    },
                    StructField {
                        name: "struct_scalar",
                        r#type: Ptr::new(("Position", ResolvedType::Struct(position.clone()))),
                        array: false,
                        optional: false,
                    },
                    StructField {
                        name: "struct_array",
                        r#type: Ptr::new(("Position", ResolvedType::Struct(position.clone()))),
                        array: true,
                        optional: false,
                    },
                    StructField {
                        name: "opt_scalar",
                        r#type: Ptr::new(("uint8", ResolvedType::Builtin(Builtin::Uint8))),
                        array: false,
                        optional: true,
                    },
                    StructField {
                        name: "opt_enum",
                        r#type: Ptr::new(("Flag", ResolvedType::Enum(flag.clone()))),
                        array: false,
                        optional: true,
                    },
                    StructField {
                        name: "opt_struct",
                        r#type: Ptr::new(("Position", ResolvedType::Struct(position.clone()))),
                        array: false,
                        optional: true,
                    },
                ],
            },
        };
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_write_impl("Test", &test);
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
    let input_opt_scalar = input.opt_scalar;
    switch (input_opt_scalar) {
        case undefined: case null: writer.write_uint8(0); break;
        default: {
            writer.write_uint8(1);
            writer.write_uint8(input_opt_scalar);
        }
    }
    let input_opt_enum = input.opt_enum;
    switch (input_opt_enum) {
        case undefined: case null: writer.write_uint8(0); break;
        default: {
            writer.write_uint8(1);
            writer.write_uint8(input_opt_enum as number);
        }
    }
    let input_opt_struct = input.opt_struct;
    switch (input_opt_struct) {
        case undefined: case null: writer.write_uint8(0); break;
        default: {
            writer.write_uint8(1);
            writer.write_float(input_opt_struct.x);
            writer.write_float(input_opt_struct.y);
        }
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
                    name: "A",
                    value: 0,
                },
                EnumVariant {
                    name: "B",
                    value: 1,
                },
            ],
        };
        let position = Struct {
            fields: vec![
                StructField {
                    name: "x",
                    r#type: Ptr::new(("float", ResolvedType::Builtin(Builtin::Float))),
                    array: false,
                    optional: false,
                },
                StructField {
                    name: "y",
                    r#type: Ptr::new(("float", ResolvedType::Builtin(Builtin::Float))),
                    array: false,
                    optional: false,
                },
            ],
        };
        let test = Export {
            name: "Test",
            r#struct: Struct {
                fields: vec![
                    StructField {
                        name: "builtin_scalar",
                        r#type: Ptr::new(("uint8", ResolvedType::Builtin(Builtin::Uint8))),
                        array: false,
                        optional: false,
                    },
                    StructField {
                        name: "builtin_array",
                        r#type: Ptr::new(("uint8", ResolvedType::Builtin(Builtin::Uint8))),
                        array: true,
                        optional: false,
                    },
                    StructField {
                        name: "string_scalar",
                        r#type: Ptr::new(("string", ResolvedType::Builtin(Builtin::String))),
                        array: false,
                        optional: false,
                    },
                    StructField {
                        name: "string_array",
                        r#type: Ptr::new(("string", ResolvedType::Builtin(Builtin::String))),
                        array: true,
                        optional: false,
                    },
                    StructField {
                        name: "enum_scalar",
                        r#type: Ptr::new(("Flag", ResolvedType::Enum(flag.clone()))),
                        array: false,
                        optional: false,
                    },
                    StructField {
                        name: "enum_array",
                        r#type: Ptr::new(("Flag", ResolvedType::Enum(flag.clone()))),
                        array: true,
                        optional: false,
                    },
                    StructField {
                        name: "struct_scalar",
                        r#type: Ptr::new(("Position", ResolvedType::Struct(position.clone()))),
                        array: false,
                        optional: false,
                    },
                    StructField {
                        name: "struct_array",
                        r#type: Ptr::new(("Position", ResolvedType::Struct(position.clone()))),
                        array: true,
                        optional: false,
                    },
                    StructField {
                        name: "opt_scalar",
                        r#type: Ptr::new(("uint8", ResolvedType::Builtin(Builtin::Uint8))),
                        array: false,
                        optional: true,
                    },
                    StructField {
                        name: "opt_enum",
                        r#type: Ptr::new(("Flag", ResolvedType::Enum(flag.clone()))),
                        array: false,
                        optional: true,
                    },
                    StructField {
                        name: "opt_struct",
                        r#type: Ptr::new(("Position", ResolvedType::Struct(position.clone()))),
                        array: false,
                        optional: true,
                    },
                ],
            },
        };
        let mut gen = Generator::<TypeScript>::new();
        gen.push_line();
        gen.push_read_impl("Test", &test);
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
    if (reader.read_uint8() > 0) {
        output.opt_scalar = reader.read_uint8();
    }
    if (reader.read_uint8() > 0) {
        output.opt_enum = Flag_try_from(reader.read_uint8());
    }
    if (reader.read_uint8() > 0) {
        let output_opt_struct = {} as unknown as Position;
        output_opt_struct.x = reader.read_float();
        output_opt_struct.y = reader.read_float();
        output.opt_struct = output_opt_struct;
    }
}
"
        );
    }
}
