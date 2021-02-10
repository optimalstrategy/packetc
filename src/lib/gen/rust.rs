use super::*;
use fstrings::format_args_f;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Rust {
    imports: HashSet<String>,
}
impl Language for Rust {}

impl Common for Rust {
    fn gen_common(&self, out: &mut String) {
        append!(
            out,
            "#![allow(dead_code, non_camel_case_types, unused_imports, clippy::field_reassign_with_default)]\n"
        );
        append!(out, "use std::convert::TryFrom;\n");
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
        "{ctx.indentation}writer.write_uint32({fname}.len() as u32);\n"
    );
    let item_var = varname(&ctx.stack, "item");
    append!(
        ctx.out,
        "{ctx.indentation}for {item_var} in {fname}.iter() {{\n"
    );
    let mut old_stack = Vec::new();
    ctx.swap_stack(&mut old_stack);
    ctx.push_fname(item_var.clone());
    ctx.push_indent();

    match type_info {
        check::Builtin::String => {
            append!(
                ctx.out,
                "{ctx.indentation}writer.write_uint32({item_var}.len() as u32);\n"
            );
            append!(
                ctx.out,
                "{ctx.indentation}writer.write_string(&{item_var});\n"
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
                "{ctx.indentation}writer.write_uint32({fname}.len() as u32);\n"
            );
            append!(ctx.out, "{ctx.indentation}writer.write_string(&{fname});\n");
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
        "{ctx.indentation}writer.write_uint32({fname}.len() as u32);\n"
    );
    let item_var = varname(&ctx.stack, "item");
    append!(
        ctx.out,
        "{ctx.indentation}for {item_var} in {fname}.iter() {{\n"
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
        "{ctx.indentation}writer.write_{repr_name}({ifname} as {type_info.repr});\n"
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
        "{ctx.indentation}writer.write_{repr_name}({fname} as {type_info.repr});\n"
    );
}

fn gen_write_impl_struct_array(ctx: &mut ImplCtx, type_info: &check::Struct, _: &str) {
    let fname = fname(&ctx.stack);
    append!(
        ctx.out,
        "{ctx.indentation}writer.write_uint32({fname}.len() as u32);\n"
    );
    let item_var = varname(&ctx.stack, "item");
    append!(
        ctx.out,
        "{ctx.indentation}for {item_var} in {fname}.iter() {{\n"
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

// awful hack
// TODO: make write_impls the same as in gen::ts
fn is_resolvedtype_struct_variant(ty: &check::ResolvedType) -> bool {
    std::mem::discriminant(ty)
        == std::mem::discriminant(&check::ResolvedType::Struct(check::Struct {
            fields: Vec::new(),
        }))
}

fn gen_write_impl_struct(ctx: &mut ImplCtx, type_info: &check::Struct, _: &str) {
    for field in &type_info.fields {
        ctx.push_fname(field.name);
        let field_type = &*field.r#type.borrow();
        let mut old_stack = if field.optional {
            let fname = fname(&ctx.stack);
            let bind_var = bindname(&ctx.stack);
            let ref_prefix = if is_resolvedtype_struct_variant(&field_type.1) {
                "&"
            } else {
                ""
            };
            append!(ctx.out, "{ctx.indentation}match {ref_prefix}{fname} {{\n");
            ctx.push_indent();
            append!(
                ctx.out,
                "{ctx.indentation}None => writer.write_uint8(0u8),\n"
            );
            append!(ctx.out, "{ctx.indentation}Some({bind_var}) => {{\n");
            ctx.push_indent();
            append!(ctx.out, "{ctx.indentation}writer.write_uint8(1u8);\n");

            let mut old_stack = Vec::new();
            ctx.swap_stack(&mut old_stack);
            ctx.push_fname(bind_var);

            Some(old_stack)
        } else {
            None
        };

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

impl<'a> WriteImpl<Rust> for check::Export<'a> {
    fn gen_write_impl(&self, _: &mut Rust, name: &str, out: &mut String) {
        let mut ctx = ImplCtx::new(out);
        ctx.push_fname("input");
        append!(
            ctx.out,
            "pub fn write(writer: &mut packet::writer::Writer, input: &{name}) {{\n"
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
        "{ctx.indentation}let {len_var} = reader.read_uint32()? as usize;\n"
    );
    let out_var = fname.clone();
    append!(ctx.out, "{ctx.indentation}{fname}.reserve({len_var});\n");
    let item_var = varname(&ctx.stack, "item");
    append!(ctx.out, "{ctx.indentation}for _ in 0..{len_var} {{\n");
    let mut old_stack = Vec::new();
    ctx.swap_stack(&mut old_stack);
    ctx.push_fname(item_var);
    ctx.push_indent();

    match type_info {
        check::Builtin::String => {
            let len_var = varname(&ctx.stack, "len");
            append!(
                ctx.out,
                "{ctx.indentation}let {len_var} = reader.read_uint32()? as usize;\n"
            );
            append!(
                ctx.out,
                "{ctx.indentation}{out_var}.push(reader.read_string({len_var})?);\n"
            );
        }
        _ => append!(
            ctx.out,
            "{ctx.indentation}{out_var}.push(reader.read_{type_name}()?);\n"
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
        append!(ctx.out, "{ctx.indentation}if reader.read_uint8()? > 0 {{\n");
        ctx.push_indent();
    }

    match type_info {
        check::Builtin::String => {
            let len_var = varname(&ctx.stack, "len");
            let fname = fname(&ctx.stack);
            append!(
                ctx.out,
                "{ctx.indentation}let {len_var} = reader.read_uint32()? as usize;\n"
            );
            let opt_prefix = if optional { "Some(" } else { "" };
            let opt_suffix = if optional { ")" } else { "" };
            append!(
                ctx.out,
                "{ctx.indentation}{fname} = {opt_prefix}reader.read_string({len_var})?{opt_suffix};\n"
            );
        }
        _ => {
            let fname = fname(&ctx.stack);
            let opt_prefix = if optional { "Some(" } else { "" };
            let opt_suffix = if optional { ")" } else { "" };
            append!(
                ctx.out,
                "{ctx.indentation}{fname} = {opt_prefix}reader.read_{type_name}()?{opt_suffix};\n"
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
        "{ctx.indentation}let {len_var} = reader.read_uint32()? as usize;\n"
    );
    let out_var = fname.clone();
    append!(ctx.out, "{ctx.indentation}{fname}.reserve({len_var});\n");
    let item_var = varname(&ctx.stack, "item");
    append!(ctx.out, "{ctx.indentation}for _ in 0..{len_var} {{\n");
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
        "{ctx.indentation}{out_var}.push({type_name}::try_from(reader.read_{repr_name}()?)?);\n"
    );

    ctx.swap_stack(&mut old_stack);
    ctx.pop_indent();
    append!(ctx.out, "{ctx.indentation}}}\n");
}

fn gen_read_impl_enum(ctx: &mut ImplCtx, type_info: &check::Enum, type_name: &str, optional: bool) {
    if optional {
        append!(ctx.out, "{ctx.indentation}if reader.read_uint8()? > 0 {{\n");
        ctx.push_indent();
    }

    let repr_name = match type_info.repr {
        check::EnumRepr::U8 => "uint8",
        check::EnumRepr::U16 => "uint16",
        check::EnumRepr::U32 => "uint32",
    };
    let fname = fname(&ctx.stack);
    let opt_prefix = if optional { "Some(" } else { "" };
    let opt_suffix = if optional { ")" } else { "" };
    append!(
        ctx.out,
        "{ctx.indentation}{fname} = {opt_prefix}{type_name}::try_from(reader.read_{repr_name}()?)?{opt_suffix};\n"
    );

    if optional {
        ctx.pop_indent();
        append!(ctx.out, "{ctx.indentation}}}\n");
    }
}

fn gen_read_impl_struct_array(ctx: &mut ImplCtx, type_info: &check::Struct, type_name: &str) {
    let len_var = varname(&ctx.stack, "len");
    let fname = fname(&ctx.stack);
    append!(
        ctx.out,
        "{ctx.indentation}let {len_var} = reader.read_uint32()? as usize;\n"
    );
    append!(ctx.out, "{ctx.indentation}{fname}.reserve({len_var});\n");
    let item_var = varname(&ctx.stack, "item");
    append!(ctx.out, "{ctx.indentation}for _ in 0..{len_var} {{\n");
    let mut old_stack = Vec::new();
    ctx.swap_stack(&mut old_stack);
    ctx.push_fname(item_var.clone());
    ctx.push_indent();

    append!(
        ctx.out,
        "{ctx.indentation}let mut {item_var} = {type_name}::default();\n"
    );
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
    append!(ctx.out, "{ctx.indentation}{ifname}.push({item_var});\n");
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
        append!(ctx.out, "{ctx.indentation}if reader.read_uint8()? > 0 {{\n");
        ctx.push_indent();
        append!(
            ctx.out,
            "{ctx.indentation}let mut {bind_var} = {type_name}::default();\n"
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
        append!(ctx.out, "{ctx.indentation}{fname} = Some({bind_var});\n");

        ctx.swap_stack(&mut old_stack);
        ctx.pop_indent();
        append!(ctx.out, "{ctx.indentation}}}\n");
    }
}

impl<'a> ReadImpl<Rust> for check::Export<'a> {
    fn gen_read_impl(&self, _: &mut Rust, name: &str, out: &mut String) {
        let mut ctx = ImplCtx::new(out);
        ctx.push_fname("output");
        append!(
            ctx.out,
            "pub fn read(reader: &mut packet::reader::Reader, output: &mut {name}) -> Result<(), packet::Error> {{\n"
        );
        ctx.push_indent();
        gen_read_impl_struct(&mut ctx, &self.r#struct, &name, false);
        append!(ctx.out, "{ctx.indentation}Ok(())\n");
        ctx.pop_indent();
        append!(ctx.out, "}}\n");
    }
}

fn struct_field_typename(base: &str, array: bool, optional: bool) -> String {
    let mut out = String::new();

    if optional {
        out += "Option<"
    }
    if array {
        out += "Vec<"
    }
    out += base;
    if optional {
        out += ">"
    }
    if array {
        out += ">"
    }

    out
}

impl<'a> Definition<Rust> for check::Struct<'a> {
    fn gen_def(&self, _: &mut Rust, name: &str, out: &mut String) {
        append!(out, "#[derive(Clone, PartialEq, Debug, Default)]\n");
        append!(out, "pub struct {name} {{\n");
        for field in self.fields.iter() {
            let type_info = &*field.r#type.borrow();
            let mut typename: &str = &type_info.0;
            if let check::ResolvedType::Builtin(b) = &type_info.1 {
                typename = match b {
                    check::Builtin::Uint8 => "u8",
                    check::Builtin::Uint16 => "u16",
                    check::Builtin::Uint32 => "u32",
                    check::Builtin::Int8 => "i8",
                    check::Builtin::Int16 => "i16",
                    check::Builtin::Int32 => "i32",
                    check::Builtin::Float => "f32",
                    check::Builtin::String => "String",
                };
            }
            let sftyname = struct_field_typename(typename, field.array, field.optional);
            append!(out, "    pub {field.name}: {sftyname},\n");
        }
        append!(out, "}}\n");
    }
}

fn gen_def_enum_default_impl<'a>(name: &str, ty: &check::Enum<'a>, out: &mut String) {
    let mut indent = String::new();
    append!(out, "{indent}impl Default for {name} {{\n");
    indent += "    ";
    append!(out, "{indent}fn default() -> Self {{\n");
    indent += "    ";
    let fvname = ty.variants.first().unwrap().name;
    append!(out, "{indent}{name}::{fvname}\n");
    indent.truncate(indent.len() - 4);
    append!(out, "{indent}}}\n");
    append!(out, "}}\n");
}

fn gen_def_enum_tryfrom_impl<'a>(name: &str, ty: &check::Enum<'a>, out: &mut String) {
    let mut indent = String::new();
    match ty.repr {
        check::EnumRepr::U8 => append!(
            out,
            "{indent}impl std::convert::TryFrom<u8> for {name} {{\n"
        ),
        check::EnumRepr::U16 => append!(
            out,
            "{indent}impl std::convert::TryFrom<u16> for {name} {{\n"
        ),
        check::EnumRepr::U32 => append!(
            out,
            "{indent}impl std::convert::TryFrom<u32> for {name} {{\n"
        ),
    }
    indent += "    ";
    append!(out, "{indent}type Error = packet::Error;\n");
    match ty.repr {
        check::EnumRepr::U8 => append!(
            out,
            "{indent}fn try_from(value: u8) -> Result<Self, Self::Error> {{\n"
        ),
        check::EnumRepr::U16 => append!(
            out,
            "{indent}fn try_from(value: u16) -> Result<Self, Self::Error> {{\n"
        ),
        check::EnumRepr::U32 => append!(
            out,
            "{indent}fn try_from(value: u32) -> Result<Self, Self::Error> {{\n"
        ),
    }
    indent += "    ";
    append!(out, "{indent}match value {{\n");
    indent += "    ";
    for variant in &ty.variants {
        let value = 1 << variant.value;
        append!(out, "{indent}{value} => Ok({name}::{variant.name}),\n");
    }
    append!(
        out,
        "{indent}_ => Err(packet::Error::InvalidEnumValue(value as usize, \"{name}\"))\n"
    );
    indent.truncate(indent.len() - 4);
    append!(out, "{indent}}}\n");
    indent.truncate(indent.len() - 4);
    append!(out, "{indent}}}\n");
    append!(out, "}}\n");
}

impl<'a> Definition<Rust> for check::Enum<'a> {
    fn gen_def(&self, _: &mut Rust, name: &str, out: &mut String) {
        append!(out, "#[derive(Clone, Copy, PartialEq, Debug)]\n");
        let repr = &self.repr;
        append!(out, "#[repr({repr})]\n");
        append!(out, "pub enum {name} {{\n");
        for variant in self.variants.iter() {
            append!(out, "    {variant.name} = 1 << {variant.value},\n");
        }
        append!(out, "}}\n");
        gen_def_enum_default_impl(name, &self, out);
        gen_def_enum_tryfrom_impl(name, &self, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn commmon_gen() {
        let mut gen = Generator::<Rust>::new();
        gen.push_line();
        gen.push_common();
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
#![allow(dead_code, non_camel_case_types, unused_imports, clippy::field_reassign_with_default)]
use std::convert::TryFrom;
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
        let mut gen = Generator::<Rust>::new();
        gen.push_line();
        gen.push_def("Position", &position);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
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
        let mut gen = Generator::<Rust>::new();
        gen.push_line();
        gen.push_def("Test", &test);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Test {
    pub a: Option<f32>,
    pub b: Option<Vec<f32>>,
    pub c: f32,
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
        let mut gen = Generator::<Rust>::new();
        gen.push_line();
        gen.push_def("Flag", &flag);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum Flag {
    A = 1 << 0,
    B = 1 << 1,
}
impl Default for Flag {
    fn default() -> Self {
        Flag::A
    }
}
impl std::convert::TryFrom<u8> for Flag {
    type Error = packet::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Flag::A),
            2 => Ok(Flag::B),
            _ => Err(packet::Error::InvalidEnumValue(value as usize, \"Flag\"))
        }
    }
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
        let mut gen = Generator::<Rust>::new();
        gen.push_line();
        gen.push_def("Test", &test.r#struct);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Test {
    pub builtin_scalar: u8,
    pub builtin_array: Vec<u8>,
    pub string_scalar: String,
    pub string_array: Vec<String>,
    pub enum_scalar: Flag,
    pub enum_array: Vec<Flag>,
    pub struct_scalar: Position,
    pub struct_array: Vec<Position>,
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
        let mut gen = Generator::<Rust>::new();
        gen.push_line();
        gen.push_write_impl("Test", &test);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
pub fn write(writer: &mut packet::writer::Writer, input: &Test) {
    match input.a {
        None => writer.write_uint8(0u8),
        Some(input_a) => {
            writer.write_uint8(1u8);
            writer.write_uint8(input_a);
        }
    }
    match input.b {
        None => writer.write_uint8(0u8),
        Some(input_b) => {
            writer.write_uint8(1u8);
            writer.write_uint32(input_b.len() as u32);
            for input_b_item in input_b.iter() {
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
        let mut gen = Generator::<Rust>::new();
        gen.push_line();
        gen.push_read_impl("Test", &test);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
pub fn read(reader: &mut packet::reader::Reader, output: &mut Test) -> Result<(), packet::Error> {
    if reader.read_uint8()? > 0 {
        output.a = Some(reader.read_uint8()?);
    }
    output.b = reader.read_uint8()?;
    Ok(())
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
        let mut gen = Generator::<Rust>::new();
        gen.push_line();
        gen.push_write_impl("TestB", &test_b);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
pub fn write(writer: &mut packet::writer::Writer, input: &TestB) {
    writer.write_uint32(input.test_a.len() as u32);
    for input_test_a_item in input.test_a.iter() {
        writer.write_uint32(input_test_a_item.first.len() as u32);
        for input_test_a_item_first_item in input_test_a_item.first.iter() {
            writer.write_uint8(input_test_a_item_first_item);
        }
        writer.write_uint32(input_test_a_item.second.len() as u32);
        for input_test_a_item_second_item in input_test_a_item.second.iter() {
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
        let mut gen = Generator::<Rust>::new();
        gen.push_line();
        gen.push_read_impl("TestB", &test_b);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
pub fn read(reader: &mut packet::reader::Reader, output: &mut TestB) -> Result<(), packet::Error> {
    let output_test_a_len = reader.read_uint32()? as usize;
    output.test_a.reserve(output_test_a_len);
    for _ in 0..output_test_a_len {
        let mut output_test_a_item = TestA::default();
        let output_test_a_item_first_len = reader.read_uint32()? as usize;
        output_test_a_item.first.reserve(output_test_a_item_first_len);
        for _ in 0..output_test_a_item_first_len {
            output_test_a_item.first.push(reader.read_uint8()?);
        }
        let output_test_a_item_second_len = reader.read_uint32()? as usize;
        output_test_a_item.second.reserve(output_test_a_item_second_len);
        for _ in 0..output_test_a_item_second_len {
            output_test_a_item.second.push(reader.read_uint8()?);
        }
        output.test_a.push(output_test_a_item);
    }
    Ok(())
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
        let mut gen = Generator::<Rust>::new();
        gen.push_line();
        gen.push_write_impl("Test", &test);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
pub fn write(writer: &mut packet::writer::Writer, input: &Test) {
    writer.write_uint8(input.builtin_scalar);
    writer.write_uint32(input.builtin_array.len() as u32);
    for input_builtin_array_item in input.builtin_array.iter() {
        writer.write_uint8(input_builtin_array_item);
    }
    writer.write_uint32(input.string_scalar.len() as u32);
    writer.write_string(&input.string_scalar);
    writer.write_uint32(input.string_array.len() as u32);
    for input_string_array_item in input.string_array.iter() {
        writer.write_uint32(input_string_array_item.len() as u32);
        writer.write_string(&input_string_array_item);
    }
    writer.write_uint8(input.enum_scalar as u8);
    writer.write_uint32(input.enum_array.len() as u32);
    for input_enum_array_item in input.enum_array.iter() {
        writer.write_uint8(input_enum_array_item as u8);
    }
    writer.write_float(input.struct_scalar.x);
    writer.write_float(input.struct_scalar.y);
    writer.write_uint32(input.struct_array.len() as u32);
    for input_struct_array_item in input.struct_array.iter() {
        writer.write_float(input_struct_array_item.x);
        writer.write_float(input_struct_array_item.y);
    }
    match input.opt_scalar {
        None => writer.write_uint8(0u8),
        Some(input_opt_scalar) => {
            writer.write_uint8(1u8);
            writer.write_uint8(input_opt_scalar);
        }
    }
    match input.opt_enum {
        None => writer.write_uint8(0u8),
        Some(input_opt_enum) => {
            writer.write_uint8(1u8);
            writer.write_uint8(input_opt_enum as u8);
        }
    }
    match &input.opt_struct {
        None => writer.write_uint8(0u8),
        Some(input_opt_struct) => {
            writer.write_uint8(1u8);
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
        let mut gen = Generator::<Rust>::new();
        gen.push_line();
        gen.push_read_impl("Test", &test);
        let actual = gen.finish();
        assert_eq!(
            actual,
            "
pub fn read(reader: &mut packet::reader::Reader, output: &mut Test) -> Result<(), packet::Error> {
    output.builtin_scalar = reader.read_uint8()?;
    let output_builtin_array_len = reader.read_uint32()? as usize;
    output.builtin_array.reserve(output_builtin_array_len);
    for _ in 0..output_builtin_array_len {
        output.builtin_array.push(reader.read_uint8()?);
    }
    let output_string_scalar_len = reader.read_uint32()? as usize;
    output.string_scalar = reader.read_string(output_string_scalar_len)?;
    let output_string_array_len = reader.read_uint32()? as usize;
    output.string_array.reserve(output_string_array_len);
    for _ in 0..output_string_array_len {
        let output_string_array_item_len = reader.read_uint32()? as usize;
        output.string_array.push(reader.read_string(output_string_array_item_len)?);
    }
    output.enum_scalar = Flag::try_from(reader.read_uint8()?)?;
    let output_enum_array_len = reader.read_uint32()? as usize;
    output.enum_array.reserve(output_enum_array_len);
    for _ in 0..output_enum_array_len {
        output.enum_array.push(Flag::try_from(reader.read_uint8()?)?);
    }
    output.struct_scalar.x = reader.read_float()?;
    output.struct_scalar.y = reader.read_float()?;
    let output_struct_array_len = reader.read_uint32()? as usize;
    output.struct_array.reserve(output_struct_array_len);
    for _ in 0..output_struct_array_len {
        let mut output_struct_array_item = Position::default();
        output_struct_array_item.x = reader.read_float()?;
        output_struct_array_item.y = reader.read_float()?;
        output.struct_array.push(output_struct_array_item);
    }
    if reader.read_uint8()? > 0 {
        output.opt_scalar = Some(reader.read_uint8()?);
    }
    if reader.read_uint8()? > 0 {
        output.opt_enum = Some(Flag::try_from(reader.read_uint8()?)?);
    }
    if reader.read_uint8()? > 0 {
        let mut output_opt_struct = Position::default();
        output_opt_struct.x = reader.read_float()?;
        output_opt_struct.y = reader.read_float()?;
        output.opt_struct = Some(output_opt_struct);
    }
    Ok(())
}
"
        );
    }
}
