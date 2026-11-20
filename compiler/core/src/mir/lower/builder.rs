use super::*;

impl Builder {
    pub(super) fn block_id(&mut self, span: Span) -> BlockId {
        let id = self.function.blocks.len();
        self.function.blocks.push(Block {
            terminator: Terminator::Unreachable,
            span,
        });
        id
    }
}

impl Builder {
    pub(super) fn end(&mut self, terminator: Terminator) {
        if let Some(id) = self.current.take() {
            self.function.blocks[id].terminator = terminator;
        }
    }
}

impl Builder {
    pub(super) fn temp(&mut self, ty: &Type, span: Span) -> LocalId {
        let id = self.function.locals.len();
        self.function
            .cleanup
            .add_local(*self.cleanup_scopes.last().unwrap());
        self.function.locals.push(h::Local {
            view: None,
            name: "<temporary>".into(),
            ty: ty.clone(),
            mutable: true,
            span,
        });
        id
    }
}

impl Builder {
    pub(super) fn evaluate_into(&mut self, operation: Operation, destination: Place, span: Span) {
        if self.current.is_none() {
            return;
        }
        self.function.blocks[self.current.unwrap()].span = span;
        let next = self.block_id(span);
        self.end(Terminator::Evaluate {
            operation,
            destination,
            next,
            unwind: self.unwind,
        });
        self.current = Some(next);
    }
}

impl Builder {
    pub(super) fn evaluate(&mut self, operation: Operation, ty: &Type, span: Span) -> Operand {
        let id = self.temp(ty, span);
        self.evaluate_into(operation, Place::local(id), span);
        Operand::Local(id)
    }
}
