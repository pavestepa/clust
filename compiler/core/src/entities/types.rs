use super::*;
pub type FunctionId = usize;
pub type LocalId = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    /// Static readonly descriptor stored as an aggregate component.
    StoredView(Box<Type>),
    /// Non-owning address. Validity and aliasing are obligations of unsafe code.
    Pointer {
        mutable: bool,
        pointee: Box<Type>,
    },
    Unit,
    Bool,
    Char,
    Integer(IntegerType),
    F32,
    F64,
    Str,
    Function {
        unsafe_: bool,
        parameters: Vec<(bool, Type)>,
        result: Box<Type>,
        contract: Option<Box<FunctionContract>>,
    },
    Tuple(Vec<Type>),
    /// Identity-bearing handle. Payload definitions live in a nominal registry,
    /// so recursive classes do not recursively expand the Type value.
    Class {
        name: String,
        copy: bool,
        storage: Option<Vec<RecordField>>,
    },
    Record {
        copy: bool,
        name: String,
        fields: Vec<RecordField>,
    },
    Array(Box<Type>, u64),
    Enum {
        copy: bool,
        name: String,
        variants: Vec<Variant>,
    },
}
/// Compile-time ABI and provenance of a plain code pointer. No environment is stored.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct FunctionContract {
    pub sources: Vec<String>,
    pub modes: Vec<Option<crate::mir::loans::Access>>,
    pub view: Option<crate::mir::returns::Contract>,
    pub storage: crate::mir::returns::StorageContract,
    pub writes: crate::mir::storage_writes::Contract,
}
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CallContract {
    pub targets: Vec<FunctionId>,
    pub view: Option<crate::mir::returns::Contract>,
    pub storage: crate::mir::returns::StorageContract,
    pub writes: crate::mir::storage_writes::Contract,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<Type>,
    pub shape: VariantShape,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VariantShape {
    Unit,
    Tuple,
    Named(Vec<String>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordField {
    pub name: String,
    pub ty: Type,
    pub private: bool,
    pub source: clust_frontend::source::SourceId,
}
