use super::*;

pub(super) fn children(pattern: &a::Pattern) -> Vec<&a::Pattern> {
    match &pattern.kind {
        a::PatternKind::Tuple(v) | a::PatternKind::Array(v) => v.iter().collect(),
        a::PatternKind::Variant { fields, .. } => match fields {
            a::PatternFields::Tuple(v) => v.iter().collect(),
            a::PatternFields::Named(v, _) => v.iter().map(|(_, p)| p).collect(),
            _ => vec![],
        },
        _ => vec![],
    }
}

pub(in super::super) fn has_ref(pattern: &a::Pattern) -> bool {
    matches!(pattern.kind, a::PatternKind::Binding { by_ref: true, .. })
        || children(pattern).iter().any(|p| has_ref(p))
}

pub(super) fn mutable_ref(pattern: &a::Pattern) -> bool {
    matches!(
        pattern.kind,
        a::PatternKind::Binding {
            by_ref: true,
            mutable: true,
            ..
        }
    ) || children(pattern).iter().any(|p| mutable_ref(p))
}

pub(super) fn only_refs(pattern: &a::Pattern) -> bool {
    !matches!(pattern.kind, a::PatternKind::Binding { by_ref: false, .. })
        && children(pattern).iter().all(|p| only_refs(p))
}

pub(in super::super) fn inferred_pattern(pattern: &a::Pattern, mutable: bool) -> a::Pattern {
    let mut result = pattern.clone();
    match &mut result.kind {
        a::PatternKind::Binding {
            by_ref,
            mutable: access,
            ..
        } => {
            *by_ref = true;
            *access |= mutable;
        }
        a::PatternKind::Tuple(fields) | a::PatternKind::Array(fields) => {
            for field in fields {
                *field = inferred_pattern(field, mutable);
            }
        }
        a::PatternKind::Variant { fields, .. } => match fields {
            a::PatternFields::Tuple(fields) => {
                for field in fields {
                    *field = inferred_pattern(field, mutable);
                }
            }
            a::PatternFields::Named(fields, _) => {
                for (_, field) in fields {
                    *field = inferred_pattern(field, mutable);
                }
            }
            _ => (),
        },
        _ => (),
    }
    result
}

impl Checker<'_> {
    pub(in super::super) fn freeze_place(
        &mut self,
        mut value: h::Expr,
        prelude: &mut Vec<h::Statement>,
    ) -> h::Expr {
        match &mut value.kind {
            h::ExprKind::Index(base, index) => {
                **base = self.freeze_place((**base).clone(), prelude);
                let id = self.locals.len();
                self.locals.push(h::Local {
                    name: "<pattern index>".into(),
                    ty: index.ty.clone(),
                    mutable: false,
                    view: None,
                    span: index.span,
                });
                prelude.push(h::Statement::Initialize(id, (**index).clone()));
                index.kind = h::ExprKind::Local(id);
            }
            h::ExprKind::Field(base, _) | h::ExprKind::Payload(base, _) => {
                **base = self.freeze_place((**base).clone(), prelude)
            }
            _ => (),
        }
        value
    }
}
