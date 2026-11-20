use super::*;

impl Checker<'_> {
    pub(super) fn pointer_entry(
        &mut self,
        signature: Signature,
        expected: Option<&Type>,
        span: Span,
    ) -> Result<h::Expr> {
        let wanted = match expected {
            Some(Type::Function {
                parameters,
                contract: Some(contract),
                ..
            }) => Some((parameters, contract)),
            _ => None,
        };
        // The entry's inferred storage modes (for example a borrowed parameter
        // whose body stores views) are part of its physical parameter types.
        let inferred = {
            let mono = self.mono.borrow();
            mono.bodies
                .get(&signature.id)
                .and_then(|d| {
                    mono.result_contracts
                        .get(&entries::result_key(d, &signature))
                })
                .filter(|info| info.complete)
                .map(|info| info.parameters.clone())
        };
        let parameters = wanted.map_or_else(
            || inferred.unwrap_or_else(|| signature.parameters.clone()),
            |(p, _)| p.iter().map(|(_, ty)| ty.clone()).collect(),
        );
        let modes = wanted.map_or_else(|| signature.modes.clone(), |(_, c)| c.modes.clone());
        let start = self.locals.len();
        self.scopes.push(BTreeMap::new());
        self.unsafe_depth += 1;
        let result = (|| {
            let mut source = Vec::new();
            let mut args = Vec::new();
            for (i, ty) in parameters.into_iter().enumerate() {
                let name = format!("<pointer argument {i}>");
                let id = self.local(name.clone(), ty.clone(), true, span)?;
                self.callable_parameters.insert(id);
                source.push(a::Expr {
                    kind: a::ExprKind::Name(name),
                    span,
                });
                args.push(h::Expr {
                    kind: h::ExprKind::Local(id),
                    ty,
                    span,
                });
            }
            let call = self.call_entry_modes(signature, &source, args, span, Some(&modes))?;
            let mut pointer = self.pointer_from_call(call, span)?;
            if let (
                Some((_, wanted)),
                Type::Function {
                    contract: Some(actual),
                    ..
                },
                h::ExprKind::Value(Value::Function(id)),
            ) = (wanted, &pointer.ty, &pointer.kind)
            {
                if wanted.view.is_some()
                    && actual.view.is_none()
                    && !self.mono.borrow().pending_results.contains(id)
                {
                    return Err(error(
                        span,
                        "E0396",
                        "incompatible owned/borrowed function result ABI",
                    ));
                }
            }
            if let Some(expected) = expected {
                pointer.ty = storage::union(&pointer.ty, expected, span)?;
            }
            Ok(pointer)
        })();
        self.unsafe_depth -= 1;
        self.scopes.pop();
        self.locals.truncate(start);
        self.callable_parameters.retain(|id| *id < start);
        result
    }
}

impl Checker<'_> {
    pub(super) fn pointer_from_call(&self, call: h::Expr, span: Span) -> Result<h::Expr> {
        let (_, _, call) = split_temporaries(call);
        let id = match &call.kind {
            h::ExprKind::Call(id, _) => *id,
            h::ExprKind::BorrowedCall { function, .. }
            | h::ExprKind::StorageCall { function, .. } => *function,
            _ => {
                return Err(error(
                    span,
                    "E0390",
                    "function pointer requires a concrete callable entry",
                ))
            }
        };
        let mono = self.mono.borrow();
        let signature = self
            .signatures
            .values()
            .chain(mono.instances.values())
            .chain(mono.entries.values())
            .find(|s| s.id == id)
            .ok_or_else(|| error(span, "E0390", "function pointer signature is unavailable"))?;
        let d = &mono.bodies[&id];
        let info = mono
            .result_contracts
            .get(&entries::result_key(d, signature));
        let parameters = info
            .map_or(&signature.parameters, |info| &info.parameters)
            .iter()
            .enumerate()
            .map(|(i, ty)| (d.parameters[i].mutable, ty.clone()))
            .collect();
        let contract = h::FunctionContract {
            sources: vec![entries::source_key(d, signature)],
            modes: info.map_or_else(|| signature.modes.clone(), |i| i.modes.clone()),
            view: info.and_then(|i| i.view.clone()),
            storage: info.map_or_else(Vec::new, |i| i.storage.clone()),
            writes: info.map_or_else(Vec::new, |i| i.writes.clone()),
        };
        Ok(h::Expr {
            kind: h::ExprKind::Value(Value::Function(id)),
            ty: Type::Function {
                unsafe_: d.unsafe_,
                parameters,
                result: Box::new(call.ty),
                contract: Some(Box::new(contract)),
            },
            span,
        })
    }
}
