use super::*;
#[derive(Clone, Default)]
pub(in crate::check) struct Substitution {
    pub types: BTreeMap<String, Type>,
    pub constants: BTreeMap<String, u64>,
    shadowed: Vec<std::collections::BTreeSet<String>>,
}
impl Substitution {
    pub fn ty(&mut self, ty: &mut a::Type) {
        if let a::TypeKind::Named { path, arguments } = &ty.kind {
            if path.len() == 1 && arguments.is_empty() {
                if let Some(value) = self.types.get(&path[0]) {
                    *ty = ast_type(value, ty.span);
                    return;
                }
            }
        }
        match &mut ty.kind {
            a::TypeKind::Named { arguments, .. } => self.arguments(arguments),
            a::TypeKind::Tuple(fields) => {
                for ty in fields {
                    self.ty(ty);
                }
            }
            a::TypeKind::Array { element, length } => {
                self.ty(element);
                self.expr(length);
            }
            a::TypeKind::Pointer { pointee, .. } => self.ty(pointee),
            a::TypeKind::Function {
                parameters, result, ..
            } => {
                for (_, ty) in parameters {
                    self.ty(ty);
                }
                self.ty(result);
            }
        }
    }
    fn arguments(&mut self, args: &mut [a::TypeArgument]) {
        for arg in args {
            if let a::TypeArgument::Type(a::Type {
                kind: a::TypeKind::Named { path, arguments },
                span,
            }) = arg
            {
                if path.len() == 1 && arguments.is_empty() {
                    if let Some(value) = self.constants.get(&path[0]) {
                        *arg = a::TypeArgument::Const(number(*value, *span));
                        continue;
                    }
                }
            }
            match arg {
                a::TypeArgument::Type(ty) => self.ty(ty),
                a::TypeArgument::Const(value) => self.expr(value),
            }
        }
    }
    pub fn function(&mut self, function: &mut a::Function) {
        self.shadowed.push(Default::default());
        for param in &mut function.parameters {
            if let Some(ty) = &mut param.ty {
                self.ty(ty);
            }
            self.pattern(&mut param.pattern);
        }
        self.ty(&mut function.result);
        if let Some(body) = &mut function.body {
            self.block(body);
        }
        self.shadowed.pop();
    }
    fn pattern(&mut self, pattern: &mut a::Pattern) {
        match &mut pattern.kind {
            a::PatternKind::Binding { name, .. } => {
                if let Some(scope) = self.shadowed.last_mut() {
                    scope.insert(name.clone());
                }
            }
            a::PatternKind::Tuple(fields) | a::PatternKind::Array(fields) => {
                for field in fields {
                    self.pattern(field);
                }
            }
            a::PatternKind::Variant { path, fields } => {
                if let Some(name) = path.first_mut() {
                    if let Some(
                        Type::Record { name: actual, .. } | Type::Enum { name: actual, .. },
                    ) = self.types.get(name)
                    {
                        *name = actual.clone();
                    }
                }
                match fields {
                    a::PatternFields::Tuple(fields) => {
                        for p in fields {
                            self.pattern(p);
                        }
                    }
                    a::PatternFields::Named(fields, _) => {
                        for (_, p) in fields {
                            self.pattern(p);
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }
}

mod constants;
mod expressions;
pub(crate) use self::constants::ast_type;
use self::constants::number;
