use super::*;

impl Checker<'_> {
    pub(super) fn require_raw_unsafe(&self, span: Span) -> Result<()> {
        if self.unsafe_depth == 0 {
            Err(error(
                span,
                "E0500",
                "raw operation requires an explicit unsafe block",
            ))
        } else {
            Ok(())
        }
    }
}

impl Checker<'_> {
    pub(super) fn mutable_place(&self, place: &h::Expr) -> Result<()> {
        if self.pending_result_place(place) {
            return Ok(());
        }
        match &place.kind {
            h::ExprKind::Unary("storage_view_mut", _) => Ok(()),
            h::ExprKind::Local(id) if self.locals[*id].mutable => Ok(()),
            h::ExprKind::View(id)
                if self.locals[*id].view == Some(crate::mir::loans::Access::Mutable) =>
            {
                Ok(())
            }
            h::ExprKind::CallIndirect(
                _,
                _,
                h::CallContract {
                    view: Some(contract),
                    ..
                },
            )
            | h::ExprKind::BorrowedCall { contract, .. }
                if contract.access == crate::mir::loans::Access::Mutable =>
            {
                Ok(())
            }
            h::ExprKind::Index(base, _)
            | h::ExprKind::Field(base, _)
            | h::ExprKind::Payload(base, _) => self.mutable_place(base),
            _ => {
                let mut diagnostic =
                    error(place.span, "E0317", "assignment requires a mutable place");
                if let h::ExprKind::View(id) = place.kind {
                    diagnostic = diagnostic.with_note(
                        self.locals[id].span,
                        "this view only grants readonly access",
                    );
                } else if matches!(place.kind, h::ExprKind::BorrowedCall { .. }) {
                    diagnostic =
                        diagnostic.with_note(place.span, "this call returns a readonly view");
                }
                Err(diagnostic)
            }
        }
    }
}
