use super::*;

pub(super) fn check_declaration(
    declaration: &a::Function,
    signature: &Signature,
    signatures: &BTreeMap<String, Signature>,
    constants: &crate::constants::Constants,
    nominals: &records::Nominals,
    mono: &std::cell::RefCell<generics::Monomorphization>,
    private_functions: &BTreeMap<String, Span>,
) -> Result<h::Function> {
    let key = entries::result_key(declaration, signature);
    mono.borrow_mut()
        .entry_keys
        .insert(signature.id, key.clone());
    let initial = mono.borrow().clone();
    let mut hints = storage::Modes::new();
    if let Some(info) = initial.result_contracts.get(&key) {
        for (index, (parameter, ty)) in declaration
            .parameters
            .iter()
            .zip(&info.parameters)
            .enumerate()
        {
            hints.insert(
                storage::key(parameter.span),
                if info.modes[index].is_some() && signature.modes[index].is_none() {
                    Type::StoredView(Box::new(ty.clone()))
                } else {
                    ty.clone()
                },
            );
        }
    }
    for _ in 0..64 {
        let (result, requests) = check_declaration_modes(
            declaration,
            signature,
            signatures,
            constants,
            nominals,
            mono,
            private_functions,
            hints.clone(),
        );
        if requests.is_empty() {
            return result;
        }
        for (key, ty) in requests {
            hints.insert(key, ty);
        }
        // Provisional calls/destructor entries must not survive a representation
        // retry. They may have a different ABI from the stable local modes.
        let requests = mono.borrow().result_requests.clone();
        *mono.borrow_mut() = initial.clone();
        mono.borrow_mut().result_requests.extend(requests);
    }
    Err(error(
        declaration.span,
        "E0396",
        "local storage mode inference did not converge",
    ))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn check_declaration_modes(
    declaration: &a::Function,
    signature: &Signature,
    signatures: &BTreeMap<String, Signature>,
    constants: &crate::constants::Constants,
    nominals: &records::Nominals,
    mono: &std::cell::RefCell<generics::Monomorphization>,
    private_functions: &BTreeMap<String, Span>,
    hints: storage::Modes,
) -> (Result<h::Function>, storage::Modes) {
    let mut checker = Checker {
        unsafe_depth: 0,
        callable_parameters: Default::default(),
        member_owner: declaration
            .name
            .rsplit_once('.')
            .map(|(owner, _)| owner.to_owned()),
        storage_hints: hints,
        storage_requests: Default::default(),
        storage_roots: Default::default(),
        storage_sources: Default::default(),
        storage_parameters: Default::default(),
        signatures,
        mono,
        constants,
        nominals,
        private_functions,
        pattern_views: false,
        continuations: vec![],
        value_uses: declaration
            .body
            .as_ref()
            .map(uses::names)
            .unwrap_or_default(),
        scopes: vec![BTreeMap::new()],
        locals: Vec::new(),
        result: mono
            .borrow()
            .result_contracts
            .get(&entries::result_key(declaration, signature))
            .map_or_else(|| signature.result.clone(), |i| i.ty.clone()),
        actual_result: None,
        loops: 0,
    };
    let result = (|| {
        let mut parameters = Vec::new();
        let mut prelude = Vec::new();
        // A plain parameter name is a view local bound to the parameter itself.
        let mut parameter_names = BTreeMap::new();
        for (parameter_index, (parameter, ty)) in declaration
            .parameters
            .iter()
            .zip(&signature.parameters)
            .enumerate()
        {
            let ty = checker
                .storage_hints
                .get(&storage::key(parameter.span))
                .unwrap_or(ty)
                .clone();
            let (ty, mode) = match ty {
                Type::StoredView(target) => (
                    *target,
                    Some(if parameter.mutable {
                        crate::mir::loans::Access::Mutable
                    } else {
                        crate::mir::loans::Access::Readonly
                    }),
                ),
                ty => (ty, signature.modes[parameter_index]),
            };
            let id = checker.locals.len();
            checker.locals.push(h::Local {
                view: mode,
                name: "<parameter>".into(),
                ty: ty.clone(),
                mutable: false,
                span: parameter.span,
            });
            parameters.push(id);
            checker.storage_roots.insert(id);
            checker.storage_parameters.insert(id);
            let value = h::Expr {
                kind: if mode.is_some() {
                    h::ExprKind::View(id)
                } else {
                    h::ExprKind::Local(id)
                },
                ty: ty.clone(),
                span: parameter.span,
            };
            let first = checker.locals.len();
            let (_, irrefutable, _) = checker.pattern(
                &if mode.is_some() {
                    views::inferred_pattern(&parameter.pattern, parameter.mutable)
                } else {
                    parameter.pattern.clone()
                },
                &value,
                &mut prelude,
            )?;
            if !irrefutable {
                return Err(error(
                    parameter.span,
                    "E0344",
                    "parameter requires an irrefutable pattern",
                ));
            }
            if parameter.mutable {
                for local in &mut checker.locals[first..] {
                    local.mutable = true;
                }
            }
            if checker.locals.len() == first + 1 {
                parameter_names.insert(first, id);
            }
        }
        let source_body = declaration
            .body
            .as_ref()
            .ok_or_else(|| error(declaration.span, "E0300", "function body required"))?;
        let mut body = checker.block(source_body, None, false)?;
        body.storage = (0..checker.locals.len()).collect();
        prelude.append(&mut body.statements);
        body.statements = prelude;
        if signature.result != Type::Unit && !body.diverges {
            return Err(error(
                declaration.span,
                "E0302",
                "non-unit function must explicitly return on every path",
            ));
        }
        for ty in checker
            .locals
            .iter()
            .map(|l| l.ty.clone())
            .collect::<Vec<_>>()
        {
            checker.schedule_destructors(&ty, declaration.span)?;
        }
        checker.schedule_destructors(&signature.result, declaration.span)?;
        let result = checker
            .actual_result
            .clone()
            .unwrap_or_else(|| signature.result.clone());
        let function = h::Function {
            rust_adapter: None,
            storage_writes: Vec::new(),
            result_view: None,
            name: declaration.name.clone(),
            parameters,
            locals: std::mem::take(&mut checker.locals),
            result,
            body,
            span: declaration.span,
        };
        consume::record(&function, declaration, mono, &parameter_names)?;
        Ok(function)
    })();
    (result, checker.storage_requests)
}
