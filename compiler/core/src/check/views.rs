use super::*;
use crate::mir::loans::Access;
/// Explicit views of local inline values. Borrowing is represented in HIR/MIR;
/// no copy of the source aggregate is inserted.
impl Checker<'_> {
    // Provisional call places are admitted only while result contracts are
    // unresolved. The stable pass must establish a real borrowed result.
    pub(super) fn pending_result_place(&self, value: &h::Expr) -> bool {
        match &value.kind {
            h::ExprKind::Call(id, _) => self.mono.borrow().pending_results.contains(id),
            h::ExprKind::CallIndirect(_, _, contract) => contract
                .targets
                .iter()
                .any(|id| self.mono.borrow().pending_results.contains(id)),
            h::ExprKind::Field(base, _)
            | h::ExprKind::Index(base, _)
            | h::ExprKind::Payload(base, _) => self.pending_result_place(base),
            _ => false,
        }
    }
    pub(super) fn view_binding(
        &mut self,
        pattern: &a::Pattern,
        source: &a::Expr,
        annotation: Option<&a::Type>,
    ) -> Result<h::Statement> {
        let expected = annotation
            .map(|t| resolve_type_with(t, self.constants, self.nominals))
            .transpose()?;
        let value = self.expr(source, expected.as_ref(), true)?;
        if !place(&value) && !self.pending_result_place(&value) {
            return Err(error(
                source.span,
                "E0382",
                "explicit ref requires an existing local place",
            ));
        }
        if !only_refs(pattern) {
            return Err(error(
                pattern.span,
                "E0900",
                "mixed consuming/ref patterns require ownership extraction lowering",
            ));
        }
        let access = if mutable_ref(pattern) {
            Access::Mutable
        } else {
            Access::Readonly
        };
        if access == Access::Mutable {
            self.mutable_place(&value)?;
        }
        let id = self.locals.len();
        self.locals.push(h::Local {
            name: "<pattern view>".into(),
            ty: value.ty.clone(),
            mutable: false,
            view: Some(access),
            span: source.span,
        });
        self.storage_sources.insert(id, vec![value.clone()]);
        let reference = h::Expr {
            kind: h::ExprKind::View(id),
            ty: value.ty.clone(),
            span: source.span,
        };
        let mut bindings = vec![h::Statement::Evaluate(h::Expr {
            kind: h::ExprKind::Borrow {
                local: id,
                place: Box::new(value),
            },
            ty: Type::Unit,
            span: source.span,
        })];
        self.scopes.push(BTreeMap::new());
        let (_, irrefutable, _) = self.pattern(pattern, &reference, &mut bindings)?;
        if !irrefutable {
            return Err(error(
                pattern.span,
                "E0344",
                "binding requires an irrefutable pattern",
            ));
        }
        let names = self.scopes.pop().unwrap();
        self.scopes.last_mut().unwrap().extend(names);
        Ok(h::Statement::Evaluate(h::Expr {
            kind: h::ExprKind::Block(h::Block {
                construction: vec![],
                storage: vec![],
                statements: bindings,
                result: Type::Unit,
                diverges: false,
                span: pattern.span,
            }),
            ty: Type::Unit,
            span: pattern.span,
        }))
    }
}
pub(super) fn place(value: &h::Expr) -> bool {
    match &value.kind {
        h::ExprKind::Unary("storage_view" | "storage_view_mut", _) => true,
        h::ExprKind::Local(_)
        | h::ExprKind::View(_)
        | h::ExprKind::StoredView(_)
        | h::ExprKind::BorrowedCall { .. }
        | h::ExprKind::CallIndirect(_, _, h::CallContract { view: Some(_), .. }) => true,
        h::ExprKind::Index(base, _)
        | h::ExprKind::Field(base, _)
        | h::ExprKind::Payload(base, _) => place(base),
        _ => false,
    }
}
pub(super) fn view_root(value: &h::Expr) -> bool {
    match &value.kind {
        h::ExprKind::Unary("storage_view" | "storage_view_mut", _) => true,
        h::ExprKind::View(_)
        | h::ExprKind::StoredView(_)
        | h::ExprKind::BorrowedCall { .. }
        | h::ExprKind::CallIndirect(_, _, h::CallContract { view: Some(_), .. }) => true,
        h::ExprKind::Index(base, _)
        | h::ExprKind::Field(base, _)
        | h::ExprKind::Payload(base, _) => view_root(base),
        _ => false,
    }
}

mod patterns;
pub(super) use self::patterns::has_ref;
pub(super) use self::patterns::inferred_pattern;
use self::patterns::mutable_ref;
use self::patterns::only_refs;
