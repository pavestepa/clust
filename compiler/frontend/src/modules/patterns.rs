use super::*;

impl Rewrite<'_> {
    pub(super) fn pattern(&mut self, pattern: &mut Pattern) {
        match &mut pattern.kind {
            PatternKind::Binding { name, .. } => {
                self.scopes.last_mut().unwrap().insert(name.clone());
            }
            PatternKind::Tuple(fields) | PatternKind::Array(fields) => {
                for field in fields {
                    self.pattern(field);
                }
            }
            PatternKind::Variant { path, fields } => {
                self.path(path, pattern.span);
                match fields {
                    PatternFields::Unit => {}
                    PatternFields::Tuple(fields) => {
                        for field in fields {
                            self.pattern(field);
                        }
                    }
                    PatternFields::Named(fields, _) => {
                        for (_, field) in fields {
                            self.pattern(field);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
