use super::*;

impl Flow {
    pub(super) fn address(&mut self, value: &a::Expr, target: &Place) {
        match &value.kind {
            a::ExprKind::Index { value, index } => {
                self.address(value, target);
                self.read(index, target)
            }
            a::ExprKind::Group(value) | a::ExprKind::Member { value, .. } => {
                self.address(value, target)
            }
            _ => (),
        }
    }
}

impl Flow {
    pub(super) fn block(&mut self, body: &a::Block, target: &Place) {
        self.statements(&body.statements, target)
    }
}

impl Flow {
    pub(super) fn statements(&mut self, statements: &[a::Statement], target: &Place) {
        for s in statements {
            if self.stopped {
                break;
            }
            match &s.kind {
                a::StatementKind::Binding { value, .. } | a::StatementKind::Expression(value) => {
                    self.read(value, target)
                }
                a::StatementKind::Return(value) => {
                    if let Some(value) = value {
                        self.read(value, target);
                    }
                    self.stopped = true;
                }
                a::StatementKind::Unsafe(body) => self.block(body, target),
                a::StatementKind::Break | a::StatementKind::Continue => {
                    self.used |= self.live;
                    self.stopped = true;
                }
            }
        }
    }
}
