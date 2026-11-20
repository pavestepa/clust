use super::*;

pub fn check(function: &Function) -> Vec<Error> {
    if function
        .locals
        .iter()
        .all(|local| local.view.is_none() && !local.ty.contains_views())
    {
        return Vec::new();
    }
    let count = function.blocks.len();
    let provenance = provenance(function);
    let orders =
        (!function.drop_reads.is_empty()).then(|| drop_borrows::orders(function, &provenance));
    let Provenance {
        incoming,
        origins,
        sites,
    } = provenance;
    let parameter_loans = parameter_loans(function);
    let live = dataflow::liveness(function);
    let mut errors = Vec::new();
    for (id, block) in function.blocks.iter().enumerate() {
        if origins
            .iter()
            .enumerate()
            .any(|(loan, origin)| sites[loan] == Some(id) && origin.parents.contains(&loan))
        {
            errors.push(Error {
                loan: None,
                block: id,
                message: "cyclic reborrowing across loop iterations is not yet supported",
            });
        }
        let Some(bindings) = &incoming[id] else {
            continue;
        };
        let mut holders = BTreeMap::<BlockId, BTreeSet<LocalId>>::new();
        for local in &live.live_in[id] {
            for loan in bindings
                .iter()
                .filter(|(place, _)| place.local == *local)
                .flat_map(|(_, loans)| loans)
            {
                holders.entry(*loan).or_default().insert(*local);
            }
        }
        loop {
            let old = holders.clone();
            for (loan, readers) in &old {
                for origin in &origins[*loan].places {
                    for (_, loans) in bindings
                        .iter()
                        .filter(|(stored, _)| compatible_prefix(origin, stored))
                    {
                        for loan in loans {
                            holders.entry(*loan).or_default().extend(readers);
                        }
                    }
                }
            }
            if holders == old {
                break;
            }
        }
        let active = holders.keys().copied().collect::<BTreeSet<_>>();
        if let Terminator::EndStorage { locals, .. } = &block.terminator {
            for loan in &active {
                if origins[*loan].places.iter().any(|p| {
                    locals.contains(&p.local)
                        && !orders.as_ref().is_some_and(|orders| {
                            holders[loan].iter().all(|holder| {
                                locals.contains(holder)
                                    && function.drop_reads.contains(holder)
                                    && orders.cleans_before(function, id, *holder, p.local)
                            })
                        })
                        && !(function.parameters.contains(&p.local)
                            && (function.locals[p.local].view.is_some()
                                || p.projections.contains(&Projection::Dereference)))
                }) {
                    errors.push(Error {
                        loan: Some(*loan),
                        block: id,
                        message: "view outlives its local target",
                    });
                }
            }
        }
        let mut access = |place: &Place, kind: Access, mutation: bool| {
            let resolved = resolve(place, bindings, &origins);
            if resolved.places.is_empty() {
                errors.push(Error {
                    loan: None,
                    block: id,
                    message: "view has no initialized origin",
                });
                return;
            }
            if mutation {
                let mutable = if matches!(place.projections.first(), Some(Projection::Dereference))
                {
                    function.locals[place.local].view == Some(Access::Mutable)
                } else {
                    function.locals[place.local].mutable
                };
                if !mutable {
                    errors.push(Error {
                        loan: None,
                        block: id,
                        message: "mutable access requires a mutable origin",
                    });
                }
            }
            for loan in &active {
                if resolved.ancestors.contains(loan) {
                    continue;
                }
                let capability = if *loan >= count {
                    Some(
                        parameter_loans
                            .get(*loan - count)
                            .map_or(Access::Readonly, |loan| loan.2),
                    )
                } else {
                    let Terminator::Evaluate {
                        destination,
                        operation,
                        ..
                    } = &function.blocks[*loan].terminator
                    else {
                        unreachable!()
                    };
                    if matches!(operation, Operation::CapturePlace(_)) {
                        continue;
                    }
                    if matches!(operation, Operation::StoreView(_)) {
                        Some(Access::Readonly)
                    } else {
                        function.locals[destination.local].view
                    }
                };
                if (kind == Access::Mutable || capability == Some(Access::Mutable))
                    && resolved
                        .places
                        .iter()
                        .any(|a| origins[*loan].places.iter().any(|b| overlaps(a, b)))
                {
                    errors.push(Error {
                        loan: Some(*loan),
                        block: id,
                        message: "access conflicts with a view that is used later",
                    });
                    break;
                }
            }
        };
        if let Terminator::Evaluate {
            operation,
            destination,
            ..
        } = &block.terminator
        {
            match operation {
                Operation::Identity(a, b) => {
                    access(a, Access::Readonly, false);
                    access(b, Access::Readonly, false);
                }
                Operation::StoreView(source) => access(source, Access::Readonly, false),
                Operation::LoadView(source) => access(source, Access::Readonly, false),
                Operation::Borrow(source) => access(
                    source,
                    function.locals[destination.local]
                        .view
                        .unwrap_or(Access::Readonly),
                    function.locals[destination.local].view == Some(Access::Mutable),
                ),
                Operation::Copy(source) | Operation::TagPlace(source) => {
                    access(source, Access::Readonly, false)
                }
                Operation::Move(source) => access(source, Access::Mutable, false),
                Operation::Unary("raw_addr", Operand::Address(source)) => {
                    access(source, Access::Readonly, false)
                }
                Operation::Unary("raw_addr_mut", Operand::Address(source)) => {
                    access(source, Access::Mutable, true)
                }
                _ => (),
            }
            if matches!(operation, Operation::Move(Place { projections, .. }) if projections.first() == Some(&Projection::Dereference))
            {
                errors.push(Error {
                    loan: None,
                    block: id,
                    message: "cannot move ownership out of a borrowed view",
                });
            }
            let owns_operand = |operand: &Operand| matches!(operand, Operand::Local(local) if function.locals[*local].view.is_some());
            let transfers_view = match operation {
                Operation::Use(value) | Operation::CompleteClass(value) => owns_operand(value),
                Operation::Aggregate(values)
                | Operation::Enum(_, values)
                | Operation::Call(_, values)
                | Operation::CallIndirect(_, values, _)
                | Operation::CallStorage {
                    arguments: values, ..
                }
                | Operation::CallView {
                    arguments: values, ..
                } => values.iter().any(owns_operand),
                _ => false,
            };
            if transfers_view {
                errors.push(Error {
                    loan: None,
                    block: id,
                    message: "cannot transfer a borrowed result into owning storage",
                });
            }
            // Root writes initialize/rebind a slot, not necessarily a mutable binding.
            // They still end the previous owner's lifetime and conflict with live loans.
            if !matches!(
                operation,
                Operation::Borrow(_)
                    | Operation::CapturePlace(_)
                    | Operation::CallView { .. }
                    | Operation::LoadView(_)
                    | Operation::StoreView(_)
            ) {
                let resolved = resolve(destination, bindings, &origins);
                for loan in &active {
                    if *loan < count
                        && matches!(
                            &function.blocks[*loan].terminator,
                            Terminator::Evaluate {
                                operation: Operation::CapturePlace(_),
                                ..
                            }
                        )
                    {
                        continue;
                    }
                    if !resolved.ancestors.contains(loan)
                        && resolved
                            .places
                            .iter()
                            .any(|a| origins[*loan].places.iter().any(|b| overlaps(a, b)))
                    {
                        errors.push(Error {
                            loan: Some(*loan),
                            block: id,
                            message: "replacement conflicts with a view that is used later",
                        });
                        break;
                    }
                }
                if matches!(
                    destination.projections.first(),
                    Some(Projection::Dereference)
                ) && function.locals[destination.local].view != Some(Access::Mutable)
                {
                    errors.push(Error {
                        loan: None,
                        block: id,
                        message: "cannot write through a readonly view",
                    });
                }
            }
        }
    }
    errors
}
