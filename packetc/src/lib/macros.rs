#[macro_export]
macro_rules! append {
    ($dst:expr, $($arg:tt)*) => (write!($dst, $($arg)*).unwrap());
    ($dst:expr, $arg:expr) => (write!($dst, "{}", $arg).unwrap());
}

#[macro_export]
macro_rules! gen_builtin {
    ($dst:expr, string, $indentation:ident, $varname:ident) => {
        append!(
            $dst,
            "{}writer.write_slice(&{}.as_bytes());\n",
            indent(*$indentation),
            $varname
        )
    };
    ($dst:expr, $builtin:ident, $indentation:ident, $varname:ident) => {
        append!(
            $dst,
            "{}writer.write_{}({});\n",
            indent(*$indentation),
            stringify!($builtin),
            $varname
        )
    };
}

#[macro_export]
macro_rules! gen_enum {
    ($dst:expr, $builtin:ident, $as:ty, $indentation:ident, $varname:ident) => {
        append!(
            $dst,
            "{}writer.write_{}({} as {});\n",
            indent(*$indentation),
            stringify!($builtin),
            $varname,
            stringify!($as)
        )
    };
}

#[macro_export]
macro_rules! gen_array_start {
    ($dst:expr, $id:ident, $fname_stack:ident, $indentation:ident, $varname:ident) => {
        append!(
            $dst,
            "{}for n_{} in self.{}{}iter() {{\n",
            indent(*$indentation),
            *$id,
            $fname_stack.join("."),
            if $fname_stack.len() > 1 { "." } else { "" }
        );
        *$indentation += 1;
    };
}

#[macro_export]
macro_rules! gen_array_end {
    ($dst:expr, $indentation:ident) => {
        *$indentation -= 1;
        append!($dst, "{}}}\n", indent(*$indentation));
    };
}

#[macro_export]
macro_rules! gen_rust_prelude {
    ($out:expr, $array:ident, $name:ident, $id:ident, $fname_stack:ident, $indentation:ident, $varname:ident) => {{
        *$id += 1;
        // TODO: check sizes
        // TODO: DRY
        $fname_stack.push($name.to_string());
        if $array {
            gen_array_start!($out, $id, $fname_stack, $indentation, $varname);
        }
        if $array {
            format!("n_{}", *$id)
        } else {
            format!("self.{}", $fname_stack.join("."))
        }
    }};
}

#[macro_export]
macro_rules! gen_rust_epilogue {
    ($out:expr, $array:ident, $fname_stack:ident, $indentation:ident) => {{
        if $array {
            gen_array_end!($out, $indentation);
        }
        $fname_stack.pop();
    }};
}
