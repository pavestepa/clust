use super::*;

impl Engine<'_> {
    pub(super) fn run(
        &mut self,
        function: &Function,
        locals: &mut [Slot],
        views: &mut Views,
    ) -> Result<Value, Trap> {
        let mut pc = function.entry;
        let mut pending = None;
        let mut initialization = super::super::initialization::State::default();
        if function.cleanup.owners.len() != function.locals.len() {
            return Err(Trap {
                kind: TrapKind::InvalidIr,
                span: function.span,
            });
        }
        let mut cleanup = super::super::cleanup::State::new(&function.cleanup);
        for local in &function.parameters {
            initialization.initialize(Place::local(*local));
            if function.locals[*local].view.is_none() {
                cleanup.initialize(*local);
            }
        }
        loop {
            let Some(block) = function.blocks.get(pc) else {
                return Err(Trap {
                    kind: TrapKind::InvalidIr,
                    span: function.span,
                });
            };
            let span = block.span;
            if self.fuel == 0 {
                return Err(Trap {
                    kind: TrapKind::StepLimit,
                    span,
                });
            }
            self.fuel -= 1;
            if super::super::dataflow::required_initialized(&block.terminator)
                .iter()
                .any(|place| {
                    initialization.status(place)
                        != super::super::initialization::Status::Initialized
                })
            {
                return Err(Trap {
                    kind: TrapKind::InvalidIr,
                    span,
                });
            }
            match &block.terminator {
                Terminator::EndStorage {
                    locals: ended,
                    next,
                } => {
                    let was_panicking = pending.is_some();
                    cleanup.end(ended);
                    for (_, mut selected) in function.cleanup.groups(ended) {
                        selected.sort_by_key(|id| std::cmp::Reverse(locals[*id].initialized_at()));
                        for local in selected {
                            if function.locals[local].view.is_none() {
                                self.drop_place(
                                    Reference {
                                        slot: locals[local].clone(),
                                        projections: vec![],
                                    },
                                    &function.locals[local].ty,
                                    &mut pending,
                                    span,
                                )?;
                            }
                            *locals[local].borrow_mut() = None;
                            locals[local].state().borrow_mut().finish(&[]);
                            views.remove(&local);
                            initialization.set(
                                Place::local(local),
                                super::super::initialization::Status::Uninitialized,
                            );
                        }
                    }
                    pc = if !was_panicking && pending.is_some() {
                        drop::unwind_entry(function).ok_or(Trap {
                            kind: TrapKind::InvalidIr,
                            span,
                        })?
                    } else {
                        *next
                    };
                }
                Terminator::Evaluate {
                    operation,
                    destination,
                    next,
                    unwind,
                } => {
                    if let Operation::Borrow(source) | Operation::CapturePlace(source) = operation {
                        let resolved = resolve(source, views, locals).and_then(|p| {
                            p.read()?;
                            if matches!(operation, Operation::Borrow(_)) {
                                p.captured()
                            } else {
                                Ok(p)
                            }
                        });
                        match resolved {
                            Ok(place) => {
                                views.insert(destination.local, place);
                                initialization.initialize(destination.clone());
                                pc = *next;
                            }
                            Err(kind) => {
                                pending = Some(Trap { kind, span });
                                pc = *unwind;
                            }
                        }
                        continue;
                    }
                    let target_type = drop::place_type(destination, function);
                    let target =
                        resolve(destination, views, locals).map_err(|kind| Trap { kind, span })?;
                    let is_call = matches!(
                        operation,
                        Operation::Call(..)
                            | Operation::CallIndirect(..)
                            | Operation::CallView { .. }
                            | Operation::CallStorage { .. }
                    );
                    let consumed = super::super::dataflow::consumed(function, operation);
                    if is_call {
                        for source in &consumed {
                            resolve(source, views, locals)
                                .and_then(|p| p.forget())
                                .map_err(|kind| Trap { kind, span })?;
                            cleanup.forget(source.local);
                            initialization.move_out(source.clone()).map_err(|_| Trap {
                                kind: TrapKind::InvalidIr,
                                span,
                            })?;
                        }
                    }
                    let entered_panicking = self.active_panic;
                    match self.operation(
                        operation,
                        locals,
                        views,
                        span,
                        &super::super::verify::place_type(function, destination)
                            .unwrap_or(Type::Unit),
                    ) {
                        Ok(value) => {
                            if !is_call {
                                for source in consumed {
                                    resolve(&source, views, locals)
                                        .and_then(|p| p.forget())
                                        .map_err(|kind| Trap { kind, span })?;
                                    if source.projections.is_empty() {
                                        cleanup.forget(source.local);
                                    }
                                    initialization.move_out(source).map_err(|_| Trap {
                                        kind: TrapKind::InvalidIr,
                                        span,
                                    })?;
                                }
                            }
                            if destination.projections.is_empty()
                                && function.locals[destination.local].view.is_some()
                            {
                                let Value::View(value) = value else {
                                    return Err(Trap {
                                        kind: TrapKind::InvalidIr,
                                        span,
                                    });
                                };
                                views.insert(destination.local, value.0);
                                initialization.initialize(destination.clone());
                                pc = *next;
                                continue;
                            }
                            self.drop_place(target.clone(), &target_type, &mut pending, span)?;
                            if pending.is_some() {
                                self.drop_value(value, &target_type, &mut pending, span)?;
                                pc = *unwind;
                            } else {
                                target.write(value).map_err(|kind| Trap { kind, span })?;
                                initialization.initialize(destination.clone());
                                if destination.projections.is_empty()
                                    && function.locals[destination.local].view.is_none()
                                {
                                    cleanup.initialize(destination.local);
                                }
                                pc = *next;
                            }
                        }
                        Err(trap) => {
                            let raised_in_drop = matches!(
                                operation,
                                Operation::Unary(
                                    "raw_drop" | "storage_drop" | "storage_drop_range",
                                    _
                                )
                            ) && !entered_panicking
                                && self.active_panic;
                            let trap = if is_call || raised_in_drop {
                                trap
                            } else {
                                self.new_panic(trap)
                            };
                            if trap.kind == TrapKind::DoublePanic {
                                return Err(trap);
                            }
                            pending = Some(trap);
                            pc = *unwind;
                        }
                    }
                }
                Terminator::CheckPlace {
                    place,
                    next,
                    unwind,
                } => match resolve(place, views, locals).and_then(|p| p.read()) {
                    Ok(_) => pc = *next,
                    Err(kind) => {
                        let trap = self.new_panic(Trap { kind, span });
                        if trap.kind == TrapKind::DoublePanic {
                            return Err(trap);
                        }
                        pending = Some(trap);
                        pc = *unwind;
                    }
                },
                Terminator::Goto(next) => pc = *next,
                Terminator::Branch { condition, yes, no } => match operand(condition, locals) {
                    Ok(Value::Bool(value)) => pc = if value { *yes } else { *no },
                    _ => {
                        return Err(Trap {
                            kind: TrapKind::InvalidIr,
                            span,
                        })
                    }
                },
                Terminator::Return(Operand::Address(place)) => {
                    return resolve(place, views, locals)
                        .and_then(|p| p.captured())
                        .map(|p| Value::View(memory::ViewValue(p)))
                        .map_err(|kind| Trap { kind, span });
                }
                Terminator::Return(value) => {
                    let result = operand(value, locals).map_err(|kind| Trap { kind, span })?;
                    if let Operand::Local(local) = value {
                        if function.locals[*local].ty.owns_storage() {
                            Reference {
                                slot: locals[*local].clone(),
                                projections: vec![],
                            }
                            .forget()
                            .map_err(|kind| Trap { kind, span })?;
                        }
                    }
                    return Ok(result);
                }
                Terminator::ResumePanic => {
                    return Err(pending.unwrap_or(Trap {
                        kind: TrapKind::InvalidIr,
                        span,
                    }))
                }
                Terminator::Unreachable => {
                    return Err(Trap {
                        kind: TrapKind::InvalidIr,
                        span,
                    })
                }
            }
        }
    }
}
