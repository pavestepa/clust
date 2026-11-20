use super::*;
#[derive(Clone, Debug, PartialEq)]
pub struct Arm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub value: Expr,
    pub span: Span,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
}
#[derive(Clone, Debug, PartialEq)]
pub enum PatternKind {
    Wildcard,
    Binding {
        name: String,
        by_ref: bool,
        mutable: bool,
    },
    Literal {
        negative: bool,
        value: Literal,
    },
    Tuple(Vec<Pattern>),
    Array(Vec<Pattern>),
    Rest,
    Variant {
        path: Vec<String>,
        fields: PatternFields,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub enum PatternFields {
    Unit,
    Tuple(Vec<Pattern>),
    Named(Vec<(String, Pattern)>, bool),
}
