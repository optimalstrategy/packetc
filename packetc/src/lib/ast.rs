#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    Uint8,
    Uint16,
    Uint32,
    Int8,
    Int16,
    Int32,
    Float,
    String,
    Flag { variants: Vec<String> },
    Array { r#type: Box<Type> },
    Tuple { elements: Vec<Node> },
}

pub type Node = (String, Type);
pub type AST = Vec<Node>;
