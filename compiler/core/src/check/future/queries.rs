use super::*;

impl Checker<'_> {
    pub(in super::super) fn call_view_needed(&self, value: &a::Expr) -> bool {
        if self.ownership_view_needed(value) {
            return true;
        }
        let Some(target) = as_place(value) else {
            return false;
        };
        // Arguments can be followed by more reads in the same expression,
        // unlike a binding's complete initializer. Keep this suffix conservative.
        self.continuations.last().is_some_and(|(body, index)| {
            let end = body.statements[*index].span.end;
            self.value_uses.get(&target.name).is_some_and(|uses| {
                uses.iter().any(|s| {
                    s.source == value.span.source && s.start >= value.span.end && s.start < end
                })
            })
        })
    }
}

impl Checker<'_> {
    pub(in super::super) fn ownership_view_needed(&self, value: &a::Expr) -> bool {
        let Some(target) = as_place(value) else {
            return false;
        };
        if self
            .lookup(&target.name)
            .is_some_and(|id| self.locals[id].view.is_some())
        {
            return true;
        }
        let mut flow = Flow {
            live: true,
            used: false,
            stopped: false,
        };
        for (depth, (body, index)) in self.continuations.iter().rev().enumerate() {
            // Reach the enclosing expression only after the inner continuation.
            // A definite replacement in that continuation kills the old value,
            // including before the next iteration of an enclosing loop.
            if depth > 0 && flow.live && !flow.stopped {
                let end = body.statements[*index].span.end;
                if self.value_uses.get(&target.name).is_some_and(|uses| {
                    uses.iter().any(|s| {
                        s.source == value.span.source && s.start >= value.span.end && s.start < end
                    })
                }) {
                    return true;
                }
            }
            // A return's value never continues into later statements in this
            // block or its enclosing blocks. Uses on those paths must not turn
            // a returned owner into a view of a dying parameter.
            if matches!(body.statements[*index].kind, a::StatementKind::Return(_)) {
                break;
            }
            flow.statements(&body.statements[index + 1..], &target);
        }
        flow.used || self.loops > 0 && flow.live && !flow.stopped
    }
}
