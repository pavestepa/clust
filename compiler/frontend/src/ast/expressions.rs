use super::*;
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}
#[derive(Clone, Debug, PartialEq)]
pub enum StatementKind {
    Binding {
        mutable: bool,
        pattern: Pattern,
        ty: Option<Type>,
        value: Expr,
    },
    Expression(Expr),
    Return(Option<Expr>),
    Break,
    Continue,
    Unsafe(Block),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Integer(String),
    Float(String),
    String(String),
    Char(char),
    Bool(bool),
}
#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Closure(Box<Function>),
    Name(String),
    Specialize {
        value: Box<Expr>,
        arguments: Vec<TypeArgument>,
    },
    Literal(Literal),
    Group(Box<Expr>),
    Tuple(Vec<Expr>),
    Array(Vec<Expr>),
    Construct {
        ty: Type,
        fields: Vec<(String, Expr)>,
    },
    New {
        ty: Type,
        arguments: Vec<Expr>,
    },
    Call {
        callee: Box<Expr>,
        types: Vec<TypeArgument>,
        arguments: Vec<Expr>,
    },
    Member {
        value: Box<Expr>,
        name: String,
    },
    Index {
        value: Box<Expr>,
        index: Box<Expr>,
    },
    Unary {
        operator: &'static str,
        value: Box<Expr>,
    },
    Binary {
        operator: &'static str,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Assign {
        operator: &'static str,
        place: Box<Expr>,
        value: Box<Expr>,
    },
    Cast {
        value: Box<Expr>,
        ty: Type,
    },
    If {
        condition: Box<Expr>,
        then_block: Block,
        else_branch: Option<Box<Expr>>,
    },
    While {
        condition: Box<Expr>,
        body: Block,
    },
    Branch(Block),
    Match {
        value: Box<Expr>,
        arms: Vec<Arm>,
    },
}
