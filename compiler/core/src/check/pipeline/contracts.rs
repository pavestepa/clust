use super::*;
pub(super) fn verify(
    mut program: h::Program,
    contracts: &ReturnContracts,
    mono: &std::cell::RefCell<generics::Monomorphization>,
) -> std::result::Result<Pass, Vec<Diagnostic>> {
    let mut typed = contracts.clone();
    for (id, function) in program.functions.iter_mut().enumerate() {
        let signature = Signature {
            id,
            modes: function
                .parameters
                .iter()
                .map(|p| function.locals[*p].view)
                .collect(),
            parameters: function
                .parameters
                .iter()
                .map(|p| function.locals[*p].ty.clone())
                .collect(),
            result: function.result.clone(),
        };
        let key = mono.borrow().entry_keys[&id].clone();
        let mut info = contracts
            .get(&key)
            .filter(|info| info.ty == function.result)
            .cloned()
            .unwrap_or_else(|| ReturnInfo {
                complete: false,
                parameters: signature.parameters.clone(),
                modes: signature.modes.clone(),
                ty: function.result.clone(),
                view: None,
                storage: Vec::new(),
                writes: Vec::new(),
            });
        info.parameters = signature.parameters.clone();
        info.modes = signature.modes.clone();
        function.storage_writes = info.writes.clone();
        typed.insert(key, info);
    }
    if typed.keys().any(|key| !contracts.contains_key(key))
        || typed.iter().any(|(key, info)| {
            contracts.get(key).is_some_and(|old| {
                old.ty != info.ty || old.parameters != info.parameters || old.modes != info.modes
            })
        })
    {
        return Ok(Pass::Retry(typed));
    }
    let mir = crate::mir::lower(&program);
    let inferred = crate::mir::returns::infer(&mir).map_err(|e| {
        let mut diagnostic = error(program.functions[e.function].span, "E0395", e.message);
        if let Some(origin) = e.origin {
            diagnostic = diagnostic.with_note(
                origin,
                "this owner ends its lifetime when the function exits",
            );
        }
        vec![diagnostic]
    })?;
    let storage = crate::mir::returns::infer_storage(&mir, &inferred).map_err(|e| {
        let mut diagnostic = error(program.functions[e.function].span, "E0395", e.message);
        if let Some(origin) = e.origin {
            diagnostic = diagnostic.with_note(
                origin,
                "this owner ends its lifetime when the function exits",
            );
        }
        vec![diagnostic]
    })?;
    let writes = crate::mir::storage_writes::infer(&mir).map_err(|e| {
        let mut diagnostic = error(program.functions[e.function].span, "E0396", e.message);
        if let Some(origin) = e.origin {
            diagnostic =
                diagnostic.with_note(origin, "this owner cannot outlive the caller's storage");
        }
        vec![diagnostic]
    })?;
    let mut next = contracts.clone();
    for (id, ((function, contract), storage)) in program
        .functions
        .iter_mut()
        .zip(inferred)
        .zip(storage)
        .enumerate()
    {
        let signature = Signature {
            id,
            modes: function
                .parameters
                .iter()
                .map(|p| function.locals[*p].view)
                .collect(),
            parameters: function
                .parameters
                .iter()
                .map(|p| function.locals[*p].ty.clone())
                .collect(),
            result: function.result.clone(),
        };
        next.insert(
            mono.borrow().entry_keys[&id].clone(),
            ReturnInfo {
                complete: true,
                parameters: signature.parameters.clone(),
                modes: signature.modes.clone(),
                ty: function.result.clone(),
                view: contract.clone(),
                storage,
                writes: writes[id].clone(),
            },
        );
        function.result_view = contract;
        function.storage_writes = writes[id].clone();
    }
    if &next != contracts {
        return Ok(Pass::Retry(next));
    }
    let mir = crate::mir::lower(&program);
    classes_init::validate(&mir).map_err(|e| vec![e])?;
    let errors = crate::mir::coli::validate(&mir);
    if errors.is_empty() {
        lifecycle::validate(&program).map_err(|e| vec![e])?;
        crate::mir::verify::verify(&mir).map_err(|e| vec![e])?;
        Ok(Pass::Ready(program))
    } else {
        Err(errors)
    }
}
