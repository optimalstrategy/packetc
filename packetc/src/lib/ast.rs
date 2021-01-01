/// Unresolved is an "unchecked" type, which may be an array type
///
/// (identifier, is_array)
#[derive(Clone, PartialEq, Debug)]
pub struct Unresolved(pub String, pub bool);
/// Enum is just a list of its variants, which are plain strings
#[derive(Clone, PartialEq, Debug)]
pub struct Enum(pub Vec<String>);
/// Struct is a list of pairs of `identifier:type`, where `type` may be an array
#[derive(Clone, PartialEq, Debug)]
pub struct Struct(pub Vec<(String, Unresolved)>);
#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    Unresolved(Unresolved),
    Enum(Enum),
    /// so we re-use `Base`
    Struct(Struct),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Node {
    Decl(String, Type),
    Export(String),
}
pub type AST = Vec<Node>;
