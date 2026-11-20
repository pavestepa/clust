use super::*;

pub(super) fn arg(arg: &a::TypeArgument, visit: &mut impl FnMut(&a::Type)) {
    match arg {
        a::TypeArgument::Type(value) => ty(value, visit),
        a::TypeArgument::Const(value) => expression(value, visit),
    }
}

pub(super) fn ty(value: &a::Type, visit: &mut impl FnMut(&a::Type)) {
    visit(value);
    match &value.kind {
        a::TypeKind::Named { arguments, .. } => {
            for value in arguments {
                arg(value, visit);
            }
        }
        a::TypeKind::Tuple(types) => {
            for value in types {
                ty(value, visit);
            }
        }
        a::TypeKind::Array { element, length } => {
            ty(element, visit);
            expression(length, visit);
        }
        a::TypeKind::Pointer { pointee, .. } => ty(pointee, visit),
        a::TypeKind::Function {
            parameters, result, ..
        } => {
            for (_, value) in parameters {
                ty(value, visit);
            }
            ty(result, visit);
        }
    }
}
