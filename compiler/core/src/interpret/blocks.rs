use super::*;

impl Engine<'_> {
    pub(super) fn block(&mut self, block: &Block, frame: &mut Frame) -> Result<Value> {
        self.block_mode(block, frame, false)
    }
}

impl Engine<'_> {
    pub(super) fn block_mode(
        &mut self,
        block: &Block,
        frame: &mut Frame,
        borrowed: bool,
    ) -> Result<Value> {
        let result = self.block_body(block, frame, borrowed);
        self.finish_scope(
            result,
            &block.storage,
            !block.construction.is_empty(),
            &block.result,
            frame,
            block.span,
        )
    }
}

impl Engine<'_> {
    pub(super) fn block_body(
        &mut self,
        block: &Block,
        frame: &mut Frame,
        borrowed: bool,
    ) -> Result<Value> {
        let mut result = Value::Unit;
        for (index, statement) in block.statements.iter().enumerate() {
            self.tick(block.span)?;
            result = match statement {
                Statement::Initialize(id, value) => {
                    let value = self.expr(value, frame)?;
                    Place {
                        slot: frame.locals[*id].clone(),
                        projections: vec![],
                    }
                    .write(value)
                    .map_err(|kind| trap(kind, block.span))?;
                    Value::Unit
                }
                Statement::Evaluate(value) => {
                    let evaluated = self.expr_mode(
                        value,
                        frame,
                        borrowed && index + 1 == block.statements.len(),
                    )?;
                    if index + 1 == block.statements.len() && block.result != Type::Unit {
                        evaluated
                    } else {
                        let mut pending = None;
                        self.drop_value(evaluated, &value.ty, &mut pending, value.span)
                            .map_err(Signal::Trap)?;
                        if let Some(failure) = pending {
                            return Err(Signal::Trap(failure));
                        }
                        Value::Unit
                    }
                }
                Statement::Return(value) => {
                    let evaluated = if frame.returned_view {
                        let value = value
                            .as_ref()
                            .ok_or_else(|| trap(TrapKind::InvalidIr, block.span))?;
                        self.expr_mode(value, frame, true)?
                    } else {
                        value
                            .as_ref()
                            .map(|v| self.expr(v, frame))
                            .transpose()?
                            .unwrap_or(Value::Unit)
                    };
                    frame.return_staging = Some((
                        evaluated,
                        value.as_ref().map(|v| v.ty.clone()).unwrap_or(Type::Unit),
                    ));
                    return Err(Signal::Return);
                }
                Statement::Break => return Err(Signal::Break),
                Statement::Continue => return Err(Signal::Continue),
            };
        }
        if block.result == Type::Unit {
            Ok(Value::Unit)
        } else {
            Ok(result)
        }
    }
}
