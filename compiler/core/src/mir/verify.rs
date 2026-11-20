//! Validation at the public MIR/backend boundary. Run structural checks before
//! analyses, which deliberately assume valid indices. Recompute provenance
//! rather than trusting serialized/caller-supplied result contracts.
use super::*;
use clust_frontend::source::Diagnostic;
use std::collections::BTreeSet;

pub fn verify(program: &Program) -> Result<(), Diagnostic> {
    for f in &program.functions {
        let fail = |message| Diagnostic::new("E0401", message, f.span);
        if f.entry >= f.blocks.len()
            || f.parameters.iter().any(|p| *p >= f.locals.len())
            || f.parameters.iter().collect::<BTreeSet<_>>().len() != f.parameters.len()
            || f.drop_reads.iter().any(|p| *p >= f.locals.len())
        {
            return Err(fail("invalid MIR entry or local index"));
        }
        let plan = &f.cleanup;
        if let Some(adapter) = &f.rust_adapter {
            if !adapter.valid_path()
                || !h::RustAdapter::scalar(&f.result)
                || f.parameters.iter().any(|id| {
                    !h::RustAdapter::scalar(&f.locals[*id].ty) || f.locals[*id].view.is_some()
                })
            {
                return Err(fail("invalid Rust adapter signature or path"));
            }
        }
        if plan.parents.first() != Some(&None)
            || plan
                .parents
                .iter()
                .enumerate()
                .skip(1)
                .any(|(i, p)| p.is_none_or(|p| p >= i))
            || plan.owners.len() != f.locals.len()
            || plan.owners.iter().any(|s| *s >= plan.parents.len())
            || plan.construction.iter().any(|s| *s >= plan.parents.len())
            || plan.field_order.keys().any(|l| *l >= f.locals.len())
            || f.storage_writes.len() != program.functions.len()
        {
            return Err(fail("invalid MIR cleanup or effect table"));
        }
        for block in &f.blocks {
            let t = &block.terminator;
            if t.successors().iter().any(|b| *b >= f.blocks.len())
                || dataflow::uses(t).iter().any(|l| *l >= f.locals.len())
            {
                return Err(fail("invalid MIR successor or operand index"));
            }
            let mut places = dataflow::required_initialized(t);
            if let Terminator::Evaluate { destination, .. } = t {
                places.push(destination.clone());
            }
            if places.iter().any(|p| place_type(f, p).is_none()) {
                return Err(fail("invalid MIR place projection"));
            }
            if let Terminator::EndStorage { locals, .. } = t {
                if locals.iter().any(|l| *l >= f.locals.len()) {
                    return Err(fail("invalid MIR storage boundary"));
                }
            }
            if let Terminator::Evaluate {
                operation,
                destination,
                ..
            } = t
            {
                if destination.local >= f.locals.len()
                    || destination
                        .projections
                        .iter()
                        .any(|p| matches!(p, Projection::Index(l) if *l >= f.locals.len()))
                {
                    return Err(fail("invalid MIR destination"));
                }
                verify_raw(f, operation, destination).map_err(fail)?;
                // Two uses of the same owner in one aggregate/call would both
                // appear initialized on entry to this block. Check atomically.
                let consumed = dataflow::consumed(f, operation);
                for (i, a) in consumed.iter().enumerate() {
                    if consumed[i + 1..].iter().any(|b| loans::overlaps(a, b)) {
                        return Err(fail("MIR operation duplicates an ownership token"));
                    }
                }
                if let Operation::Copy(p) = operation {
                    if !place_type(f, p).is_some_and(|ty| ty.copyable()) {
                        return Err(fail("MIR Copy duplicates a non-Copy owner"));
                    }
                }
                if let Some((callee, args)) = storage_writes::call(operation) {
                    let Some(target) = program.functions.get(callee) else {
                        return Err(fail("invalid MIR call target"));
                    };
                    if target.parameters.len() != args.len() {
                        return Err(fail("invalid MIR call arity"));
                    }
                    for (parameter, arg) in target.parameters.iter().zip(args) {
                        let Some(parameter) = target.locals.get(*parameter) else {
                            return Err(fail("invalid MIR callee parameter"));
                        };
                        match (parameter.view, arg) {
                            (Some(access), Operand::Address(place)) => {
                                if access == loans::Access::Mutable && !writable(f, place) {
                                    return Err(fail(
                                        "MIR call upgrades readonly access to mutable",
                                    ));
                                }
                            }
                            (None, Operand::Address(_)) | (Some(_), _) => {
                                return Err(fail("MIR call changes ownership/borrow ABI"));
                            }
                            _ => {}
                        }
                    }
                }
                if let Some((args, contract)) = returns::call_view(operation) {
                    if destination.projections.is_empty()
                        && f.locals[destination.local].view != Some(contract.access)
                    {
                        return Err(fail("MIR result slot changes borrowed access"));
                    }
                    if contract.origins.is_empty()
                        || contract.origins.iter().any(|o| o.parameter >= args.len())
                    {
                        return Err(fail("invalid MIR borrowed result origins"));
                    }
                }
                if let Some((args, contract)) = returns::call_storage(operation) {
                    if contract
                        .iter()
                        .any(|c| c.origins.iter().any(|o| o.parameter >= args.len()))
                    {
                        return Err(fail("invalid MIR storage origins"));
                    }
                }
                if let Operation::CallIndirect(_, _, contract) = operation {
                    if contract
                        .targets
                        .iter()
                        .any(|t| *t >= program.functions.len())
                    {
                        return Err(fail("invalid MIR indirect call target"));
                    }
                }
            }
        }
    }
    let views = returns::infer(program)
        .map_err(|e| Diagnostic::new("E0401", e.message, program.functions[e.function].span))?;
    for (f, inferred) in program.functions.iter().zip(&views) {
        if &f.result_view != inferred {
            return Err(Diagnostic::new(
                "E0401",
                "MIR result provenance differs from its contract",
                f.span,
            ));
        }
    }
    let storage = returns::infer_storage(program, &views)
        .map_err(|e| Diagnostic::new("E0401", e.message, program.functions[e.function].span))?;
    let writes = storage_writes::infer(program)
        .map_err(|e| Diagnostic::new("E0401", e.message, program.functions[e.function].span))?;
    for f in &program.functions {
        if *f.storage_writes != writes {
            return Err(Diagnostic::new(
                "E0401",
                "MIR storage effects differ from inferred effects",
                f.span,
            ));
        }
    }
    for f in &program.functions {
        for block in &f.blocks {
            if let Terminator::Evaluate { operation, .. } = &block.terminator {
                let mismatch = match operation {
                    Operation::CallView {
                        function, contract, ..
                    } => views[*function].as_ref() != Some(contract),
                    Operation::CallStorage {
                        function, contract, ..
                    } => &storage[*function] != contract,
                    Operation::Call(function, _) => {
                        views[*function].is_some() || !storage[*function].is_empty()
                    }
                    Operation::CallIndirect(_, _, contract) => {
                        contract.targets.iter().any(|target| {
                            let view_mismatch = match (&views[*target], &contract.view) {
                                (None, None) => false,
                                (Some(actual), Some(declared)) => {
                                    !actual.origins.is_subset(&declared.origins)
                                        || (actual.access == loans::Access::Readonly
                                            && declared.access == loans::Access::Mutable)
                                }
                                _ => true,
                            };
                            view_mismatch
                                || storage[*target].iter().any(|actual| {
                                    !contract.storage.iter().any(|declared| {
                                        actual.path == declared.path
                                            && actual.origins.is_subset(&declared.origins)
                                    })
                                })
                                || writes[*target].iter().any(|actual| {
                                    !contract.writes.iter().any(|declared| {
                                        actual.target == declared.target
                                            && actual.origins.is_subset(&declared.origins)
                                    })
                                })
                        })
                    }
                    _ => false,
                };
                if mismatch {
                    return Err(Diagnostic::new(
                        "E0401",
                        "MIR call erases or changes result provenance",
                        block.span,
                    ));
                }
            }
        }
    }
    for f in &program.functions {
        cleanup_exits(f)?;
        if let Some((block, _)) = dataflow::place_initialization_errors(f).first() {
            return Err(Diagnostic::new(
                "E0401",
                "uninitialized MIR place",
                f.blocks[*block].span,
            ));
        }
        if let Some(error) = coli::loan_errors(f, "E0401").next() {
            return Err(error);
        }
    }
    Ok(())
}

mod cleanup_edges;
mod operands;
mod places;
mod raw_operations;
use self::cleanup_edges::cleanup_exits;
use self::cleanup_edges::writable;
use self::operands::operand_type;
pub(crate) use self::places::place_type;
use self::raw_operations::verify_raw;
