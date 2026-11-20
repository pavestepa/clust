mod contracts;
mod signatures;
use super::*;

pub(super) fn check_pass(
    module: &a::Module,
    contracts: &ReturnContracts,
    consuming: &Consuming,
) -> std::result::Result<Pass, Vec<Diagnostic>> {
    let consuming_before = consuming.borrow().clone();
    let class_module = classes::prepare(module).map_err(|e| vec![e])?;
    let lifecycle_module = lifecycle::prepare(&class_module).map_err(|e| vec![e])?;
    let prepared = methods::collect(&lifecycle_module);
    let module = &prepared;
    let private_functions: BTreeMap<_, _> = module
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            a::ItemKind::Function(function) if item.private => {
                Some((function.name.clone(), function.span))
            }
            _ => None,
        })
        .collect();
    let constants = crate::constants::evaluate(module).map_err(|e| vec![e])?;
    let nominals = records::collect(module, &constants).map_err(|e| vec![e])?;
    let signatures::Declarations {
        signatures,
        declarations,
        templates,
    } = signatures::collect(module, &constants, &nominals, consuming)?;
    let mut errors = Vec::new();
    let mono = std::cell::RefCell::new(generics::Monomorphization {
        templates,
        result_contracts: contracts.clone(),
        next_id: declarations.len(),
        consuming: consuming.clone(),
        ..Default::default()
    });
    default_methods::enqueue_audits(&nominals, &constants, &mono).map_err(|e| vec![e])?;
    let mut work: std::collections::VecDeque<_> = declarations
        .into_iter()
        .map(|function| (signatures[&function.name].clone(), function))
        .collect();
    for (signature, declaration) in &work {
        mono.borrow_mut()
            .bodies
            .insert(signature.id, declaration.clone());
    }
    let mut functions = BTreeMap::new();
    loop {
        let before = mono.borrow().next_id;
        let mut deferred = Vec::new();
        while let Some((signature, declaration)) = work.pop_front() {
            let checked = check_declaration(
                &declaration,
                &signature,
                &signatures,
                &constants,
                &nominals,
                &mono,
                &private_functions,
            );
            match checked {
                Ok(function) => {
                    functions.insert(signature.id, function);
                }
                Err(e) if e.code == function_values::PENDING_CALLABLE => {
                    deferred.push((signature, declaration, e));
                }
                Err(e) => errors.push(e),
            }
            work.extend(mono.borrow_mut().pending.drain(..));
        }
        if deferred.is_empty() {
            break;
        }
        // A returned pointer can name a generic entry which its producer has
        // not instantiated yet in this contract pass. Check queued producers
        // first, then retry callers; require progress to avoid an infinite retry.
        if mono.borrow().next_id == before {
            errors.extend(deferred.into_iter().map(|(_, _, e)| e));
            break;
        }
        work.extend(deferred.into_iter().map(|(s, d, _)| (s, d)));
    }
    // Newly inferred consuming parameters change entry modes: errors of this
    // pass may be artifacts of the provisional borrowed entries.
    if *consuming.borrow() != consuming_before {
        return Ok(Pass::Retry(contracts.clone()));
    }
    errors.extend(mono.borrow().ownership_errors.clone());
    let functions = functions.into_values().collect::<Vec<_>>();
    if !mono.borrow().result_requests.is_empty() {
        let mut next = contracts.clone();
        let mono_ref = mono.borrow();
        for (key, ty) in &mono_ref.result_requests {
            let signature = signatures
                .values()
                .chain(mono_ref.instances.values())
                .chain(mono_ref.entries.values())
                .find(|s| {
                    mono_ref
                        .bodies
                        .get(&s.id)
                        .is_some_and(|d| entries::result_key(d, s) == *key)
                });
            let Some(signature) = signature else { continue };
            let info = next.entry(key.clone()).or_insert_with(|| ReturnInfo {
                complete: false,
                parameters: signature.parameters.clone(),
                modes: signature.modes.clone(),
                ty: signature.result.clone(),
                view: None,
                storage: Vec::new(),
                writes: Vec::new(),
            });
            info.ty = storage::union(&info.ty, ty, mono_ref.bodies[&signature.id].span)
                .map_err(|e| vec![e])?;
            info.complete = false;
        }
        if &next != contracts {
            return Ok(Pass::Retry(next));
        }
    }
    if errors.is_empty() {
        let mut audit_work: std::collections::VecDeque<_> =
            mono.borrow_mut().audits.drain(..).collect();
        let audit_mono = std::cell::RefCell::new(generics::Monomorphization {
            templates: mono.borrow().templates.clone(),
            result_contracts: contracts.clone(),
            consuming: consuming.clone(),
            bodies: mono.borrow().bodies.clone(),
            next_id: mono.borrow().next_id,
            auditing: true,
            ..Default::default()
        });
        loop {
            let before = audit_mono.borrow().next_id;
            let mut deferred = Vec::new();
            while let Some((signature, declaration)) = audit_work.pop_front() {
                match check_declaration(
                    &declaration,
                    &signature,
                    &signatures,
                    &constants,
                    &nominals,
                    &audit_mono,
                    &private_functions,
                ) {
                    Ok(_) => {}
                    Err(diagnostic) if diagnostic.code == function_values::PENDING_CALLABLE => {
                        deferred.push((signature, declaration, diagnostic));
                    }
                    Err(diagnostic) => {
                        errors.push(error(
                            diagnostic.span,
                            "E0373",
                            format!(
                                "generic body is not valid under its declared constraints: {}",
                                diagnostic.message
                            ),
                        ));
                        break;
                    }
                }
                audit_work.extend(audit_mono.borrow_mut().pending.drain(..));
            }
            if !errors.is_empty() || deferred.is_empty() {
                break;
            }
            if audit_mono.borrow().next_id == before {
                errors.extend(deferred.into_iter().map(|(_, _, e)| e));
                break;
            }
            audit_work.extend(deferred.into_iter().map(|(s, d, _)| (s, d)));
        }
    }
    let main = signatures.get("main").map(|s| s.id);
    if let Some(signature) = signatures.get("main") {
        let declaration = module
            .items
            .iter()
            .find(|item| matches!(&item.kind,a::ItemKind::Function(f) if f.name=="main"))
            .unwrap();
        if matches!(&declaration.kind, a::ItemKind::Function(f) if f.unsafe_)
            || !signature.parameters.is_empty()
            || !matches!(signature.result, Type::Unit | Type::Integer(I::I32))
        {
            errors.push(error(
                declaration.span,
                "E0303",
                "main must be safe, have no parameters and return () or i32",
            ));
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    for function in &functions {
        for local in &function.locals {
            nominals.register_storage(&local.ty)
        }
        nominals.register_storage(&function.result);
    }
    let program = h::Program {
        classes: nominals.classes(),
        functions,
        main,
    };
    contracts::verify(program, contracts, &mono)
}
