use super::*;

impl Engine<'_> {
    pub(super) fn call(
        &mut self,
        id: usize,
        args: Vec<Argument>,
        span: Span,
    ) -> Result<Value, Trap> {
        if self.depth >= 32 {
            return Err(Trap {
                kind: TrapKind::CallDepth,
                span,
            });
        }
        let Some(function) = self.program.functions.get(id) else {
            return Err(Trap {
                kind: TrapKind::InvalidIr,
                span,
            });
        };
        if function.rust_adapter.is_some() {
            return Err(Trap {
                kind: TrapKind::ForeignFunction,
                span,
            });
        }
        if args.len() != function.parameters.len() {
            return Err(Trap {
                kind: TrapKind::InvalidIr,
                span,
            });
        }
        let mut locals: Vec<Slot> = function
            .locals
            .iter()
            .map(|local| memory::typed_slot(local.ty.clone(), self.clock.clone()))
            .collect();
        let mut views = Views::new();
        for (id, value) in function.parameters.iter().zip(args) {
            match value {
                Argument::Owned(value) => {
                    *locals[*id].borrow_mut() = Some(value);
                    locals[*id]
                        .initialize()
                        .map_err(|kind| Trap { kind, span })?;
                }
                Argument::Borrowed(place) => {
                    views.insert(*id, place);
                }
            }
        }
        self.depth += 1;
        let result = self.run(function, &mut locals, &mut views);
        self.depth -= 1;
        for local in &locals {
            *local.borrow_mut() = None;
        }
        result
    }
}
