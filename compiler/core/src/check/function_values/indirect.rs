use super::*;

impl Checker<'_> {
    pub(in super::super) fn indirect_call(
        &mut self,
        callee: &a::Expr,
        types: &[a::TypeArgument],
        arguments: &[a::Expr],
        span: Span,
    ) -> Result<Option<h::Expr>> {
        if let a::ExprKind::Name(name) = &callee.kind {
            let implicit_field = self.lookup("this").is_some_and(|id| {
                matches!(&self.locals[id].ty, Type::Record { fields, .. } if fields.iter().any(|field|field.name==*name))
            });
            if self.lookup(name).is_none() && !implicit_field {
                return Ok(None);
            }
        } else if let Some((name, _)) = methods::resolved_static_call(callee, self.nominals) {
            let shadowed = matches!(&callee.kind,a::ExprKind::Member {value,..} if matches!(&value.kind,a::ExprKind::Name(owner) if self.lookup(owner).is_some()));
            if !shadowed
                && (self.signatures.contains_key(&name)
                    || self.mono.borrow().templates.contains_key(&name))
            {
                return Ok(None);
            }
        }
        if !types.is_empty() {
            return Err(error(
                span,
                "E0390",
                "function pointer calls cannot supply generic arguments",
            ));
        }
        let source_callee = callee;
        let callee = self.expr(callee, None, true)?;
        if matches!(callee.ty, Type::Class { .. }) {
            let invoke = a::Expr {
                kind: a::ExprKind::Member {
                    value: Box::new(source_callee.clone()),
                    name: "call".into(),
                },
                span,
            };
            return self
                .instance_call(&invoke, &[], arguments, None, span)?
                .map(Some)
                .ok_or_else(|| error(span, "E0405", "class value has no callable method"));
        }
        let Type::Function {
            unsafe_,
            parameters,
            result,
            contract,
        } = &callee.ty
        else {
            return Err(error(
                span,
                "E0390",
                "called value is not a function pointer",
            ));
        };
        if *unsafe_ && self.unsafe_depth == 0 {
            return Err(error(
                span,
                "E0500",
                "unsafe function pointer call requires an explicit unsafe block",
            ));
        }
        if parameters.len() != arguments.len() {
            return Err(error(span, "E0313", "wrong number of arguments"));
        }
        let mut values = Vec::new();
        for (argument, (_, ty)) in arguments.iter().zip(parameters) {
            let value = self.expr(argument, Some(ty), true)?;
            if !self.request_storage_mode(&value, ty)?
                && storage::union(ty, &value.ty, value.span)? != value.ty
            {
                self.same_storage_mode(ty, &value.ty, value.span)?;
            }
            values.push(value);
        }
        let mut call_contract = h::CallContract::default();
        let mut result = (**result).clone();
        // Temporaries kept alive for borrowed parameters of the selected entry.
        let mut temporaries = (Vec::new(), Vec::new());
        if let Some(contract) = contract {
            let mut selected_type = None;
            let original_values = values.clone();
            for key in &contract.sources {
                let signature = {
                    let mono = self.mono.borrow();
                    self.signatures
                        .values()
                        .chain(mono.instances.values())
                        .chain(mono.entries.values())
                        .find(|s| {
                            mono.bodies
                                .get(&s.id)
                                .is_some_and(|d| entries::source_key(d, s) == *key)
                        })
                        .cloned()
                }
                .ok_or_else(|| {
                    error(
                        span,
                        PENDING_CALLABLE,
                        "function pointer source is not yet instantiated",
                    )
                })?;
                let call = self.call_entry_modes(
                    signature,
                    arguments,
                    original_values.clone(),
                    span,
                    Some(&contract.modes),
                )?;
                let pointer = self.pointer_from_call(call.clone(), span)?;
                let h::ExprKind::Value(Value::Function(id)) = pointer.kind else {
                    unreachable!()
                };
                let (prelude, storage, call) = split_temporaries(call);
                temporaries = (prelude, storage);
                call_contract.targets.push(id);
                if let Some(old) = selected_type {
                    selected_type = Some(storage::union(&old, &pointer.ty, span)?)
                } else {
                    selected_type = Some(pointer.ty)
                }
                values = match call.kind {
                    h::ExprKind::Call(_, args)
                    | h::ExprKind::BorrowedCall {
                        arguments: args, ..
                    }
                    | h::ExprKind::StorageCall {
                        arguments: args, ..
                    } => args,
                    _ => unreachable!(),
                };
                result = call.ty;
            }
            if let Some(required) = selected_type {
                let Type::Function {
                    contract: Some(meta),
                    ..
                } = &required
                else {
                    unreachable!()
                };
                call_contract.view = meta.view.clone();
                call_contract.storage = meta.storage.clone();
                call_contract.writes = meta.writes.clone();
                if !self.request_storage_mode(&callee, &required)? {
                    self.same_storage_mode(&callee.ty, &required, span)?;
                }
            }
        } else {
            // A pointer type without a known entry uses the default ABI: non-Copy
            // parameters are borrowed (NMM §11), Copy values are passed by value.
            for (value, (mutable, ty)) in values.iter_mut().zip(parameters) {
                if !ty.copyable() {
                    let access = if *mutable {
                        crate::mir::loans::Access::Mutable
                    } else {
                        crate::mir::loans::Access::Readonly
                    };
                    self.borrow_argument(value, access, &mut temporaries.0, &mut temporaries.1)?;
                }
                storage::same_mode(ty, &value.ty, value.span)?;
            }
        }
        let call = h::Expr {
            kind: h::ExprKind::CallIndirect(Box::new(callee), values, call_contract),
            ty: result,
            span,
        };
        let (mut prelude, storage) = temporaries;
        if prelude.is_empty() {
            return Ok(Some(call));
        }
        prelude.push(h::Statement::Evaluate(call.clone()));
        Ok(Some(h::Expr {
            ty: call.ty.clone(),
            kind: h::ExprKind::Block(h::Block {
                construction: vec![],
                storage,
                statements: prelude,
                result: call.ty,
                diverges: false,
                span,
            }),
            span,
        }))
    }
}

/// A call whose borrowed arguments needed temporaries is wrapped in a block
/// `{ tmp = value; call }`; return the prelude, its storage and the bare call.
pub(super) fn split_temporaries(call: h::Expr) -> (Vec<h::Statement>, Vec<usize>, h::Expr) {
    match call.kind {
        h::ExprKind::Block(mut block) if block.construction.is_empty() => {
            match block.statements.pop() {
                Some(h::Statement::Evaluate(inner)) => (block.statements, block.storage, inner),
                Some(last) => {
                    block.statements.push(last);
                    (
                        vec![],
                        vec![],
                        h::Expr {
                            kind: h::ExprKind::Block(block),
                            ..call
                        },
                    )
                }
                None => (
                    vec![],
                    vec![],
                    h::Expr {
                        kind: h::ExprKind::Block(block),
                        ..call
                    },
                ),
            }
        }
        _ => (vec![], vec![], call),
    }
}
