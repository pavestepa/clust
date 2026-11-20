use super::*;

pub(super) fn operand_type(f: &Function, operand: &Operand) -> Option<Type> {
    match operand {
        Operand::Local(l) => f.locals.get(*l).map(|l| l.ty.clone()),
        Operand::Constant(Value::Integer(i)) => Some(Type::Integer(i.ty())),
        Operand::Constant(Value::Bool(_)) => Some(Type::Bool),
        Operand::Constant(Value::Char(_)) => Some(Type::Char),
        Operand::Constant(Value::F32(_)) => Some(Type::F32),
        Operand::Constant(Value::F64(_)) => Some(Type::F64),
        _ => None,
    }
}
