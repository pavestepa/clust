use super::*;
#[derive(Clone, Debug)]
pub struct Block {
    /// Staged named fields, in declaration order, owned by this construction.
    pub construction: Vec<LocalId>,
    pub storage: Vec<LocalId>,
    pub statements: Vec<Statement>,
    pub result: Type,
    pub diverges: bool,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub enum Statement {
    Initialize(LocalId, Expr),
    Evaluate(Expr),
    Return(Option<Expr>),
    Break,
    Continue,
}
#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Type,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub enum ExprKind {
    StoreView(Box<Expr>),
    StoredView(Box<Expr>),
    StorageCall {
        function: FunctionId,
        arguments: Vec<Expr>,
        contract: crate::mir::returns::StorageContract,
    },
    BorrowedCall {
        function: FunctionId,
        arguments: Vec<Expr>,
        contract: crate::mir::returns::Contract,
    },
    Identity(Box<Expr>, Box<Expr>),
    /// Transfer an already evaluated internal owner; never a language Copy.
    Take(Box<Expr>),
    New {
        constructor: FunctionId,
        arguments: Vec<Expr>,
    },
    ArgumentView {
        place: Box<Expr>,
        access: crate::mir::loans::Access,
    },
    View(LocalId),
    Borrow {
        local: LocalId,
        place: Box<Expr>,
    },
    Block(Block),
    Enum {
        variant: usize,
        fields: Vec<Expr>,
    },
    Tag(Box<Expr>),
    Payload(Box<Expr>, usize),
    Match(Vec<MatchArm>),
    Value(Value),
    Local(LocalId),
    Tuple(Vec<Expr>),
    Array(Vec<Expr>),
    Unary(&'static str, Box<Expr>),
    Binary(&'static str, Box<Expr>, Box<Expr>),
    Cast(Box<Expr>),
    Call(FunctionId, Vec<Expr>),
    CallIndirect(Box<Expr>, Vec<Expr>, CallContract),
    Assign {
        place: Box<Expr>,
        operator: &'static str,
        value: Box<Expr>,
    },
    Index(Box<Expr>, Box<Expr>),
    Field(Box<Expr>, usize),
    If {
        condition: Box<Expr>,
        yes: Block,
        no: Option<Block>,
    },
    While {
        condition: Box<Expr>,
        body: Block,
    },
}

#[derive(Clone, Debug)]
pub struct MatchArm {
    pub guard_bindings: Vec<Statement>,
    pub condition: Expr,
    pub bindings: Vec<Statement>,
    pub guard: Option<Expr>,
    pub body: Block,
}
