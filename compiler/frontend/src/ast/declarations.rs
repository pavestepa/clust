use super::*;
#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub kind: ItemKind,
    pub private: bool,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Attribute {
    Copy,
    Visibility(Vec<String>),
    ReprC,
    Link(String),
    Rust(String),
    Safety(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ItemKind {
    Function(Function),
    Aggregate {
        class: bool,
        name: String,
        generics: Vec<Generic>,
        interfaces: Vec<Type>,
        members: Vec<Member>,
    },
    Enum {
        name: String,
        generics: Vec<Generic>,
        variants: Vec<Variant>,
    },
    Interface {
        name: String,
        generics: Vec<Generic>,
        methods: Vec<Function>,
    },
    Const {
        name: String,
        ty: Type,
        value: Expr,
    },
    Module {
        name: String,
        body: Option<Module>,
    },
    Use(Vec<UseBinding>),
    Extern(Vec<Function>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct UseBinding {
    pub path: Vec<String>,
    pub alias: Option<String>,
    pub glob: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Generic {
    pub name: String,
    pub kind: GenericKind,
    pub span: Span,
}
#[derive(Clone, Debug, PartialEq)]
pub enum GenericKind {
    Type {
        constraints: Vec<Type>,
        default: Option<Type>,
    },
    Const {
        ty: Type,
        default: Option<Expr>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub generics: Vec<Generic>,
    pub parameters: Vec<Parameter>,
    pub result: Type,
    pub body: Option<Block>,
    pub c_abi: bool,
    pub unsafe_: bool,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    pub pattern: Pattern,
    pub mutable: bool,
    pub ty: Option<Type>,
    pub receiver: bool,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MethodDispatch {
    #[default]
    Ordinary,
    /// Public instance method whose override relationship is inferred.
    Inferred,
    Virtual,
    Override,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Member {
    Field {
        name: String,
        ty: Type,
        weak: bool,
        private: bool,
        span: Span,
    },
    Method {
        function: Function,
        private: bool,
        dispatch: MethodDispatch,
    },
    Constructor {
        parameters: Vec<Parameter>,
        body: Block,
        span: Span,
    },
    Destructor(Block),
    Const {
        name: String,
        ty: Type,
        value: Expr,
        span: Span,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Variant {
    pub name: String,
    pub fields: VariantFields,
    pub span: Span,
}
#[derive(Clone, Debug, PartialEq)]
pub enum VariantFields {
    Unit,
    Tuple(Vec<Type>),
    Named(Vec<(String, Type)>),
}
