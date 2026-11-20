use super::*;

impl Bindings {
    pub(super) fn hide_pattern(&mut self, pattern: &a::Pattern) {
        match &pattern.kind {
            a::PatternKind::Binding { name, .. } => {
                self.constants.remove(name);
            }
            a::PatternKind::Tuple(fields) | a::PatternKind::Array(fields) => {
                for field in fields {
                    self.hide_pattern(field);
                }
            }
            a::PatternKind::Variant { fields, .. } => match fields {
                a::PatternFields::Tuple(fields) => {
                    for field in fields {
                        self.hide_pattern(field);
                    }
                }
                a::PatternFields::Named(fields, _) => {
                    for (_, field) in fields {
                        self.hide_pattern(field);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl Bindings {
    pub(super) fn body(&self, method: &mut a::Function) {
        let mut local = self.clone();
        for parameter in &method.parameters {
            local.hide_pattern(&parameter.pattern);
        }
        if let Some(body) = &mut method.body {
            local.block(body);
        }
    }
}

impl Bindings {
    pub(super) fn signature(&self, method: &mut a::Function) {
        for parameter in &mut method.parameters {
            if let Some(ty) = &mut parameter.ty {
                self.ty(ty);
            }
        }
        self.ty(&mut method.result);
        for generic in &mut method.generics {
            match &mut generic.kind {
                a::GenericKind::Type {
                    constraints,
                    default,
                } => {
                    for ty in constraints {
                        self.ty(ty);
                    }
                    if let Some(ty) = default {
                        self.ty(ty);
                    }
                }
                a::GenericKind::Const { ty, default } => {
                    self.ty(ty);
                    if let Some(value) = default {
                        self.expr(value);
                    }
                }
            }
        }
    }
}
