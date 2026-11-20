use super::*;

pub(crate) fn provenance(function: &Function) -> Provenance {
    let flow = reachability::analyze(function);
    let count = function.blocks.len();
    let mut incoming = vec![None::<Bindings>; count];
    let mut initial = Bindings::new();
    let mut origins = vec![Origin::default(); count];
    for (slot, origin, _) in parameter_loans(function) {
        let id = origins.len();
        initial.insert(slot, BTreeSet::from([id]));
        origins.push(Origin {
            places: BTreeSet::from([origin]),
            ..Origin::default()
        });
    }
    let mut sites = (0..count).map(Some).collect::<Vec<_>>();
    sites.resize(origins.len(), None);
    let mut call_loans = BTreeMap::<BlockId, Vec<BlockId>>::new();
    for (block, body) in function.blocks.iter().enumerate() {
        if let Terminator::Evaluate { operation, .. } = &body.terminator {
            if let Some((_, contract)) = returns::call_storage(operation) {
                let ids = (0..contract.len())
                    .map(|_| {
                        let id = origins.len();
                        origins.push(Origin::default());
                        sites.push(Some(block));
                        id
                    })
                    .collect();
                call_loans.insert(block, ids);
            }
        }
    }
    let mut write_loans = BTreeMap::<BlockId, Vec<BlockId>>::new();
    for (block, body) in function.blocks.iter().enumerate() {
        if let Terminator::Evaluate { operation, .. } = &body.terminator {
            if let Some((_, writes)) = storage_writes::effects(function, operation) {
                let ids = writes
                    .iter()
                    .map(|_| {
                        let id = origins.len();
                        origins.push(Origin::default());
                        sites.push(Some(block));
                        id
                    })
                    .collect();
                write_loans.insert(block, ids);
            }
        }
    }
    incoming[function.entry] = Some(initial);
    // Revisit all reachable blocks when a loan's resolved provenance expands.
    loop {
        let mut changed_origins = false;
        let mut queue = VecDeque::from([function.entry]);
        let mut visited = BTreeSet::new();
        while let Some(id) = queue.pop_front() {
            let input = incoming[id].clone().unwrap_or_default();
            let mut success = input.clone();
            let mut panic = input.clone();
            let term = &function.blocks[id].terminator;
            if let Terminator::Evaluate {
                operation,
                destination,
                ..
            } = term
            {
                if let Operation::Borrow(source)
                | Operation::CapturePlace(source)
                | Operation::StoreView(source) = operation
                {
                    let resolved = resolve(source, &input, &origins);
                    if !origins[id].parents.contains(&id) && origins[id] != resolved {
                        origins[id] = resolved;
                        changed_origins = true;
                    }
                    success.insert(destination.clone(), BTreeSet::from([id]));
                } else if let Operation::LoadView(source) = operation {
                    let mut loaded = Bindings::new();
                    transfer(source, destination, &input, &origins, &mut loaded);
                    let loans = loaded.values().flatten().copied().collect();
                    success.insert(destination.clone(), loans);
                } else if let Some((arguments, contract)) = returns::call_storage(operation) {
                    for target in resolve(destination, &input, &origins).places {
                        success.retain(|p, _| !covers(&target, p) || parameter_slot(function, p));
                        for (component, loan) in contract.iter().zip(&call_loans[&id]) {
                            let resolved =
                                call_origin(arguments, &component.origins, &input, &origins);
                            if !origins[*loan].parents.contains(loan) && origins[*loan] != resolved
                            {
                                origins[*loan] = resolved;
                                changed_origins = true;
                            }
                            let mut slot = target.clone();
                            slot.projections.extend_from_slice(&component.path);
                            success.insert(slot, BTreeSet::from([*loan]));
                        }
                    }
                } else if let Some((arguments, contract)) = returns::call_view(operation) {
                    let resolved = call_origin(arguments, &contract.origins, &input, &origins);
                    if !origins[id].parents.contains(&id) && origins[id] != resolved {
                        origins[id] = resolved;
                        changed_origins = true;
                    }
                    success.insert(destination.clone(), BTreeSet::from([id]));
                } else {
                    // Replacing a slot removes only its stored descriptors. Moves
                    // and copies transfer the same loans, never the target owner.
                    let targets = resolve(destination, &input, &origins).places;
                    for target in &targets {
                        success.retain(|p, _| !covers(target, p) || parameter_slot(function, p));
                    }
                    for target in &targets {
                        match operation {
                            Operation::Use(Operand::Local(local))
                            | Operation::CompleteClass(Operand::Local(local)) => transfer(
                                &Place::local(*local),
                                target,
                                &input,
                                &origins,
                                &mut success,
                            ),
                            Operation::Copy(source) | Operation::Move(source) => {
                                transfer(source, target, &input, &origins, &mut success)
                            }
                            Operation::Aggregate(values) | Operation::Enum(_, values) => {
                                for (i, value) in values.iter().enumerate() {
                                    if let Operand::Local(local) = value {
                                        let mut field = target.clone();
                                        if let Operation::Enum(variant, _) = operation {
                                            field.projections.push(Projection::Variant(*variant));
                                        }
                                        field.projections.push(
                                            if matches!(
                                                function.locals[destination.local].ty,
                                                Type::Array(..)
                                            ) {
                                                Projection::ConstantIndex(i as u64)
                                            } else {
                                                Projection::Field(i)
                                            },
                                        );
                                        transfer(
                                            &Place::local(*local),
                                            &field,
                                            &input,
                                            &origins,
                                            &mut success,
                                        );
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
            if let Terminator::Evaluate { operation, .. } = term {
                if let Some((arguments, writes)) = storage_writes::effects(function, operation) {
                    for (write, loan) in writes.iter().zip(&write_loans[&id]) {
                        let resolved = call_origin(arguments, &write.origins, &input, &origins);
                        if !origins[*loan].parents.contains(loan) && origins[*loan] != resolved {
                            origins[*loan] = resolved;
                            changed_origins = true;
                        }
                        let Some(Operand::Address(argument)) =
                            arguments.get(write.target.parameter)
                        else {
                            continue;
                        };
                        let mut slot = argument.clone();
                        slot.projections.extend_from_slice(&write.target.path);
                        for target in resolve(&slot, &input, &origins).places {
                            // A may-write contract preserves old dependencies on
                            // both edges. Calls may panic after publishing a view.
                            success.entry(target.clone()).or_default().insert(*loan);
                            panic.entry(target).or_default().insert(*loan);
                        }
                    }
                }
            }
            if let Terminator::Evaluate { operation, .. } = term {
                for source in dataflow::consumed(function, operation) {
                    let sources = resolve(&source, &input, &origins).places;
                    if sources.len() != 1 {
                        continue;
                    }
                    for source in sources {
                        success.retain(|p, _| {
                            !covers(&source, p)
                                || parameter_slot(function, p)
                                || p.projections.contains(&Projection::Dereference)
                        });
                        if storage_writes::call(operation).is_some()
                            || matches!(operation, Operation::CallIndirect(..))
                            || drop_borrows::stages_on_unwind(operation)
                        {
                            panic.retain(|p, _| {
                                !covers(&source, p)
                                    || parameter_slot(function, p)
                                    || p.projections.contains(&Projection::Dereference)
                            });
                        }
                    }
                }
            }
            if let Terminator::EndStorage { locals, .. } = term {
                for local in locals {
                    success.retain(|p, _| p.local != *local);
                }
            }
            visited.insert(id);
            for next in &flow.successors[id] {
                let next = *next;
                let output = match term {
                    Terminator::Evaluate { next: ok, .. } if *ok == next => &success,
                    Terminator::EndStorage { .. } => &success,
                    _ => &panic,
                };
                let changed = match &mut incoming[next] {
                    Some(state) => merge(state, output),
                    slot @ None => {
                        *slot = Some(output.clone());
                        true
                    }
                };
                if changed || !visited.contains(&next) {
                    queue.push_back(next);
                }
            }
        }
        if !changed_origins {
            break;
        }
    }
    Provenance {
        incoming,
        origins,
        sites,
    }
}
