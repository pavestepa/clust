use super::*;

impl Engine<'_> {
    pub(super) fn operation(
        &mut self,
        operation: &Operation,
        locals: &[Slot],
        views: &Views,
        span: Span,
        result_type: &Type,
    ) -> Result<Value, Trap> {
        let value = |value: &Operand| operand(value, locals).map_err(|kind| Trap { kind, span });
        let values = |values: &[Operand]| values.iter().map(value).collect::<Result<Vec<_>, _>>();
        let result = match operation {
            Operation::Unary("weak_identity", Operand::Address(place)) => {
                let Value::Class(object) = resolve(place, views, locals)
                    .and_then(|place| place.read())
                    .map_err(|kind| Trap { kind, span })?
                else {
                    return Err(Trap {
                        kind: TrapKind::InvalidIr,
                        span,
                    });
                };
                self.weak.create(object)
            }
            Operation::Unary("weak_alive", token) => {
                Ok(Value::Bool(self.weak.get(&value(token)?, None).is_some()))
            }
            Operation::Binary("weak_access", token, callback) => {
                let token = value(token)?;
                let Value::Function(id) = value(callback)? else {
                    return Err(Trap {
                        kind: TrapKind::InvalidIr,
                        span,
                    });
                };
                let function = &self.program.functions[id];
                let target_type = &function.locals[function.parameters[0]].ty;
                if let Some((target, temporary)) = self.weak.access(
                    &token,
                    target_type,
                    &self.program.classes,
                    self.clock.clone(),
                ) {
                    let view = memory::class_reference(target.clone(), self.clock.clone());
                    let result = self.call(id, vec![Argument::Borrowed(view)], span);
                    if temporary {
                        self.weak.invalidate(&target);
                    }
                    let result = result?;
                    Ok(Value::Enum {
                        variant: 1,
                        fields: vec![result],
                    })
                } else {
                    Ok(Value::Enum {
                        variant: 0,
                        fields: vec![],
                    })
                }
            }
            Operation::StorageView { arguments, .. } => {
                let Operand::Address(storage) = &arguments[0] else {
                    return Err(Trap {
                        kind: TrapKind::InvalidIr,
                        span,
                    });
                };
                let storage = resolve(storage, views, locals)
                    .and_then(|p| p.read())
                    .map_err(|kind| Trap { kind, span })?;
                self.allocations
                    .storage_view(storage, value(&arguments[1])?)
            }
            Operation::StoreView(p) => resolve(p, views, locals)
                .and_then(|p| p.captured())
                .map(|p| Value::View(memory::ViewValue(p))),
            Operation::LoadView(p) => resolve(p, views, locals).and_then(|p| p.read()),
            Operation::Identity(left, right) => {
                let left = resolve(left, views, locals).and_then(|p| p.read());
                let right = resolve(right, views, locals).and_then(|p| p.read());
                match (left, right) {
                    (Ok(Value::Class(a)), Ok(Value::Class(b))) => {
                        Ok(Value::Bool(a.same_identity(&b)))
                    }
                    _ => Err(TrapKind::InvalidIr),
                }
            }
            Operation::AllocateClass(name) => Ok(Value::Class(memory::allocate_class(
                name,
                &self.program.classes[name],
                self.clock.clone(),
            ))),
            Operation::CompleteClass(v) => {
                let value = value(v)?;
                memory::complete_class(&value).map_err(|kind| Trap { kind, span })?;
                Ok(value)
            }
            Operation::Borrow(_) | Operation::CapturePlace(_) => Err(TrapKind::InvalidIr),
            Operation::Copy(place) => resolve(place, views, locals)
                .and_then(|p| p.read())
                .and_then(|v| memory::copy_value(&v, self.clock.clone())),
            Operation::Move(place) => resolve(place, views, locals).and_then(|p| p.read()),
            Operation::Use(v) => return value(v),
            Operation::Unary(op @ ("raw_expose" | "raw_expose_mut" | "raw_recover"), operand) => {
                let operand = value(operand)?;
                if *op == "raw_recover" {
                    self.allocations.recover(operand, result_type)
                } else {
                    self.allocations
                        .expose(operand, *op == "raw_expose_mut", result_type)
                }
            }
            Operation::Unary("storage_allocate", count) => {
                let Value::Integer(count) = value(count)? else {
                    return Err(Trap {
                        kind: TrapKind::InvalidIr,
                        span,
                    });
                };
                let Type::Pointer { pointee, .. } = result_type else {
                    return Err(Trap {
                        kind: TrapKind::InvalidIr,
                        span,
                    });
                };
                self.allocations
                    .allocate(pointee, count.bits() as u64, self.clock.clone())
            }
            Operation::Unary(
                op @ ("raw_initialize"
                | "storage_initialize"
                | "storage_replace"
                | "storage_take"
                | "raw_take"
                | "raw_drop"
                | "storage_drop"
                | "storage_drop_range"
                | "storage_detach"
                | "storage_adopt"
                | "storage_release"
                | "storage_check_range"
                | "storage_move_slots"
                | "storage_address"
                | "storage_address_mut"),
                args,
            ) => {
                if matches!(*op, "raw_drop" | "storage_drop" | "storage_drop_range") {
                    let Operand::Local(local) = args else {
                        return Err(Trap {
                            kind: TrapKind::InvalidIr,
                            span,
                        });
                    };
                    let Type::Tuple(fields) = locals[*local].ty() else {
                        return Err(Trap {
                            kind: TrapKind::InvalidIr,
                            span,
                        });
                    };
                    let Type::Pointer { pointee, .. } = &fields[0] else {
                        return Err(Trap {
                            kind: TrapKind::InvalidIr,
                            span,
                        });
                    };
                    let argument = value(args)?;
                    if *op == "storage_drop_range" {
                        let (pointer, start, count) = self
                            .allocations
                            .drop_range_arguments(argument)
                            .map_err(|kind| Trap { kind, span })?;
                        let mut pending = None;
                        for index in start..start + count {
                            let argument = Value::Aggregate(vec![
                                pointer.clone(),
                                Value::Integer(Integer::from_bits(
                                    IntegerType::Usize,
                                    index as u128,
                                )),
                            ]);
                            let payload = self
                                .allocations
                                .operation(
                                    if *op == "raw_drop" {
                                        "raw_take"
                                    } else {
                                        "storage_take"
                                    },
                                    argument,
                                )
                                .map_err(|kind| Trap { kind, span })?;
                            self.drop_value(payload, pointee, &mut pending, span)?;
                        }
                        return pending.map_or(Ok(Value::Unit), Err);
                    }
                    let payload = self
                        .allocations
                        .operation(
                            if *op == "raw_drop" {
                                "raw_take"
                            } else {
                                "storage_take"
                            },
                            argument,
                        )
                        .map_err(|kind| Trap { kind, span })?;
                    let mut pending = None;
                    self.drop_value(payload, pointee, &mut pending, span)?;
                    return pending.map_or(Ok(Value::Unit), Err);
                }
                self.allocations.operation(op, value(args)?)
            }
            Operation::Unary("raw_addr" | "raw_addr_mut", Operand::Address(p)) => {
                resolve(p, views, locals)
                    .and_then(memory::RawPointer::new)
                    .map(Value::Pointer)
            }
            Operation::Unary(operator, v) => match (*operator, value(v)?) {
                ("str_byte_len", Value::Str(value)) => {
                    Ok(Value::Integer(clust_semantics::Integer::from_bits(
                        clust_semantics::IntegerType::Usize,
                        value.len() as u128,
                    )))
                }
                ("write_byte", Value::Integer(value)) => {
                    crate::output::write_byte(value.bits() as u8)
                }
                ("io_print", value) => crate::output::write(value, false),
                ("io_println", value) => crate::output::write(value, true),
                ("assert", Value::Bool(true)) => Ok(Value::Unit),
                ("assert", Value::Bool(false)) => Err(TrapKind::Assertion),
                ("raw_read", Value::Pointer(p)) => p.read(),
                ("-", Value::Integer(v)) => v
                    .negate(self.checks)
                    .map(Value::Integer)
                    .map_err(TrapKind::Arithmetic),
                ("!", Value::Integer(v)) => Ok(Value::Integer(v.bit_not())),
                ("!", Value::Bool(v)) => Ok(Value::Bool(!v)),
                ("-", Value::F32(v)) => Ok(Value::F32(-v)),
                ("-", Value::F64(v)) => Ok(Value::F64(-v)),
                _ => Err(TrapKind::InvalidIr),
            },
            Operation::Binary(op, a, b) => binary(op, value(a)?, value(b)?, self.checks),
            Operation::Cast(v, ty) => cast(value(v)?, ty).ok_or(TrapKind::InvalidIr),
            Operation::Aggregate(v) => Ok(Value::Aggregate(values(v)?)),
            Operation::Enum(variant, v) => Ok(Value::Enum {
                variant: *variant,
                fields: values(v)?,
            }),
            Operation::TagPlace(place) => {
                match resolve(place, views, locals)
                    .and_then(|p| p.read())
                    .map_err(|kind| Trap { kind, span })?
                {
                    Value::Enum { variant, .. } => Ok(Value::Integer(Integer::from_bits(
                        IntegerType::U32,
                        variant as u128,
                    ))),
                    _ => Err(TrapKind::InvalidIr),
                }
            }
            Operation::Tag(v) => match value(v)? {
                Value::Enum { variant, .. } => Ok(Value::Integer(Integer::from_bits(
                    IntegerType::U32,
                    variant as u128,
                ))),
                _ => Err(TrapKind::InvalidIr),
            },
            Operation::Call(..)
            | Operation::CallIndirect(..)
            | Operation::CallView { .. }
            | Operation::CallStorage { .. } => {
                let (id, args) = match operation {
                    Operation::Call(id, args) => (*id, args),
                    Operation::CallStorage {
                        function,
                        arguments,
                        ..
                    }
                    | Operation::CallView {
                        function,
                        arguments,
                        ..
                    } => (*function, arguments),
                    Operation::CallIndirect(callee, args, _) => {
                        let Value::Function(id) = value(callee)? else {
                            return Err(Trap {
                                kind: TrapKind::InvalidIr,
                                span,
                            });
                        };
                        (id, args)
                    }
                    _ => unreachable!(),
                };
                let arguments = args
                    .iter()
                    .map(|arg| match arg {
                        Operand::Address(place) => resolve(place, views, locals)
                            .map(Argument::Borrowed)
                            .map_err(|kind| Trap { kind, span }),
                        other => value(other).map(Argument::Owned),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                return self.call(id, arguments, span);
            }
        };
        result.map_err(|kind| Trap { kind, span })
    }
}
