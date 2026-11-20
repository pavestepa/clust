use super::*;
#[derive(Clone, Debug, PartialEq)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}
#[derive(Clone, Debug, PartialEq)]
pub enum TypeKind {
    Named {
        path: Vec<String>,
        arguments: Vec<TypeArgument>,
    },
    Tuple(Vec<Type>),
    Array {
        element: Box<Type>,
        length: Box<Expr>,
    },
    Pointer {
        mutable: bool,
        pointee: Box<Type>,
    },
    Function {
        unsafe_: bool,
        parameters: Vec<(bool, Type)>,
        result: Box<Type>,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub enum TypeArgument {
    Type(Type),
    Const(Expr),
}
