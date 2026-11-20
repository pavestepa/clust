use super::*;

impl Rewrite<'_> {
    pub(super) fn ty(&mut self, ty: &mut Type) {
        match &mut ty.kind {
            TypeKind::Named { path, arguments } => {
                self.path(path, ty.span);
                self.arguments(arguments);
            }
            TypeKind::Tuple(fields) => {
                for field in fields {
                    self.ty(field);
                }
            }
            TypeKind::Array { element, length } => {
                self.ty(element);
                self.expr(length);
            }
            TypeKind::Pointer { pointee, .. } => self.ty(pointee),
            TypeKind::Function {
                parameters, result, ..
            } => {
                for (_, ty) in parameters {
                    self.ty(ty);
                }
                self.ty(result);
            }
        }
    }
}

impl Rewrite<'_> {
    pub(super) fn arguments(&mut self, arguments: &mut [TypeArgument]) {
        for argument in arguments {
            match argument {
                TypeArgument::Type(ty) => self.ty(ty),
                TypeArgument::Const(value) => self.expr(value),
            }
        }
    }
}
