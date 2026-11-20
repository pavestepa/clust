use super::*;
#[derive(Clone, Debug)]
pub struct Program {
    pub classes: Classes,
    pub functions: Vec<Function>,
    pub main: Option<FunctionId>,
}
#[derive(Clone, Debug)]
pub struct Function {
    pub rust_adapter: Option<RustAdapter>,
    pub storage_writes: crate::mir::storage_writes::Contract,
    pub result_view: Option<crate::mir::returns::Contract>,
    pub name: String,
    pub parameters: Vec<LocalId>,
    pub locals: Vec<Local>,
    pub result: Type,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct RustAdapter {
    pub path: String,
    pub unsafe_: bool,
    pub safety: Option<String>,
}
impl RustAdapter {
    pub fn valid_path(&self) -> bool {
        !self.path.is_empty()
            && self.path.split('.').all(|part| {
                let mut chars = part.chars();
                chars
                    .next()
                    .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
                    && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            })
    }
    pub fn scalar(ty: &Type) -> bool {
        matches!(
            ty,
            Type::Unit
                | Type::Bool
                | Type::Char
                | Type::Integer(_)
                | Type::F32
                | Type::F64
                | Type::Str
        )
    }
}
#[derive(Clone, Debug)]
pub struct Local {
    pub view: Option<crate::mir::loans::Access>,
    pub name: String,
    pub ty: Type,
    pub mutable: bool,
    pub span: Span,
}
