//! Demand-driven static owned/borrowed entry specialization.
use super::*;
use crate::mir::loans::Access;
pub(super) fn result_key(declaration: &a::Function, signature: &Signature) -> String {
    format!(
        "{:?}:{:?}:{:?}:{:?}:{:?}",
        declaration.span,
        declaration.body,
        signature.parameters,
        signature.modes,
        signature.result.logical()
    )
}
pub(super) fn source_key(declaration: &a::Function, signature: &Signature) -> String {
    format!(
        "{:?}:{:?}:{:?}:{:?}",
        declaration.span,
        declaration.body,
        signature
            .parameters
            .iter()
            .map(Type::logical)
            .collect::<Vec<_>>(),
        signature.result.logical()
    )
}
impl Checker<'_> {
    pub(super) fn consuming_parameter(&self, declaration: &a::Function, index: usize) -> bool {
        let mono = self.mono.borrow();
        let table = mono.consuming.borrow();
        table
            .get(consume::base_name(&declaration.name))
            .is_some_and(|flags| flags.get(index) == Some(&true))
    }
    /// Pass `argument` as a view of its place; a value without a place gets a
    /// temporary that lives for the whole call.
    pub(super) fn borrow_argument(
        &mut self,
        argument: &mut h::Expr,
        access: Access,
        prelude: &mut Vec<h::Statement>,
        storage: &mut Vec<usize>,
    ) -> Result<()> {
        if !views::place(argument) {
            let id = self.locals.len();
            self.locals.push(h::Local {
                view: None,
                name: "<receiver temporary>".into(),
                ty: argument.ty.clone(),
                mutable: true,
                span: argument.span,
            });
            prelude.push(h::Statement::Initialize(id, argument.clone()));
            storage.push(id);
            argument.kind = h::ExprKind::Local(id);
        }
        if access == Access::Mutable {
            self.mutable_place(argument)?;
        }
        *argument = h::Expr {
            ty: argument.ty.clone(),
            span: argument.span,
            kind: h::ExprKind::ArgumentView {
                place: Box::new(argument.clone()),
                access,
            },
        };
        Ok(())
    }
    pub(super) fn contextual_result(
        &mut self,
        mut value: h::Expr,
        expected: Option<&Type>,
    ) -> Result<h::Expr> {
        if let Some(expected) = expected {
            let expected = match expected {
                Type::StoredView(target) => &**target,
                other => other,
            };
            same(expected, &value.ty, value.span)?;
            if expected != &value.ty && self.request_storage_mode(&value, expected)? {
                value.ty = storage::union(&value.ty, expected, value.span)?;
            }
        }
        Ok(value)
    }
    pub(super) fn request_result_mode(&mut self, value: &h::Expr, required: &Type) -> Result<bool> {
        if matches!(required, Type::StoredView(_)) {
            return Ok(false);
        }
        let targets = match &value.kind {
            h::ExprKind::Call(id, _)
            | h::ExprKind::BorrowedCall { function: id, .. }
            | h::ExprKind::StorageCall { function: id, .. } => vec![*id],
            h::ExprKind::CallIndirect(_, _, contract) => contract.targets.clone(),
            _ => return Ok(false),
        };
        let mut changed = false;
        for id in targets {
            let mut mono = self.mono.borrow_mut();
            let Some(signature) = self
                .signatures
                .values()
                .chain(mono.instances.values())
                .chain(mono.entries.values())
                .find(|s| s.id == id)
                .cloned()
            else {
                continue;
            };
            let key = result_key(&mono.bodies[&id], &signature);
            let known = mono
                .result_requests
                .get(&key)
                .or_else(|| mono.result_contracts.get(&key).map(|i| &i.ty))
                .unwrap_or(&value.ty);
            let wanted = storage::union(known, required, value.span)?;
            if known != &wanted {
                mono.result_requests.insert(key, wanted);
                changed = true;
            }
        }
        Ok(changed)
    }
    pub(super) fn call_entry(
        &mut self,
        signature: Signature,
        source: &[a::Expr],
        arguments: Vec<h::Expr>,
        span: Span,
    ) -> Result<h::Expr> {
        self.call_entry_modes(signature, source, arguments, span, None)
    }
}

mod specialization;
