use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassDefinition {
    pub fields: Vec<RecordField>,
    pub copy: bool,
}
pub type Classes = std::collections::BTreeMap<String, ClassDefinition>;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    View(crate::reference_memory::ViewValue),
    Pointer(crate::reference_memory::RawPointer),
    Class(crate::reference_memory::ClassValue),
    Unit,
    Bool(bool),
    Char(char),
    Integer(Integer),
    F32(f32),
    F64(f64),
    Str(String),
    Function(FunctionId),
    Aggregate(Vec<Value>),
    Enum { variant: usize, fields: Vec<Value> },
}
