use super::*;

impl Builder {
    pub(super) fn aggregate_fields(&mut self, values: &[Expr]) -> Vec<Operand> {
        let parent = *self.cleanup_scopes.last().unwrap();
        let construction = self.function.cleanup.construction_scope(parent);
        let mut fields = Vec::with_capacity(values.len());
        for value in values {
            let expression = self.function.cleanup.scope(construction);
            self.cleanup_scopes.push(expression);
            let operand = self.expr(value);
            self.cleanup_scopes.pop();
            if let Operand::Local(local) = &operand {
                self.function.cleanup.assign(*local, construction);
            }
            fields.push(operand);
        }
        fields
    }
}

impl Builder {
    pub(super) fn check_place(&mut self, place: &Place, span: Span) {
        if self.current.is_none() {
            return;
        }
        self.function.blocks[self.current.unwrap()].span = span;
        let next = self.block_id(span);
        self.end(Terminator::CheckPlace {
            place: place.clone(),
            next,
            unwind: self.unwind,
        });
        self.current = Some(next);
    }
}
