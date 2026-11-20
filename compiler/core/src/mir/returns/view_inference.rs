use super::*;

pub fn infer(program: &Program) -> Result<Vec<Option<Contract>>, Error> {
    let mut summaries = vec![Fact::default(); program.functions.len()];
    // Every fact is an owned bit, a capability bit and a finite set of symbolic
    // parameter paths. Recursive calls are solved together, never assumed pure.
    for _ in 0..512 {
        let next = program
            .functions
            .iter()
            .map(|f| analyze(f, &summaries))
            .collect::<Vec<_>>();
        if next == summaries {
            return next
                .into_iter()
                .enumerate()
                .map(|(id, fact)| {
                    let f = &program.functions[id];
                    if fact.origins.is_empty() {
                        return Ok(None);
                    }
                    if fact.owned {
                        return Err(Error {
                            origin: None,
                            function: id,
                            message:
                                "one entry cannot return both an owned value and a borrowed view",
                        });
                    }
                    let mut origins = BTreeSet::new();
                    for place in fact.origins {
                        let parameter = place.local;
                        let Some(_) = f.parameters.get(parameter).filter(|p| {
                            f.locals[**p].view.is_some()
                                || place.projections.contains(&Projection::Dereference)
                        }) else {
                            let local = f
                                .parameters
                                .get(parameter)
                                .copied()
                                .unwrap_or_else(|| parameter - f.parameters.len());
                            return Err(Error {
                                origin: f.locals.get(local).map(|l| l.span),
                                function: id,
                                message:
                                    "returned view depends on an owner destroyed by this function",
                            });
                        };
                        origins.insert(Origin {
                            parameter,
                            path: place.projections,
                        });
                    }
                    Ok(Some(Contract {
                        access: if fact.readonly {
                            loans::Access::Readonly
                        } else {
                            loans::Access::Mutable
                        },
                        origins,
                    }))
                })
                .collect();
        }
        summaries = next;
    }
    Err(Error {
        origin: None,
        function: 0,
        message: "recursive result provenance exceeds the supported analysis limit",
    })
}

pub(super) fn analyze(f: &Function, summaries: &[Fact]) -> Fact {
    let provenance = borrow_check::provenance(f);
    let flow = reachability::analyze(f);
    let mut initial = BTreeMap::new();
    for p in &f.parameters {
        if f.locals[*p].view.is_none() {
            initial.insert(*p, Fact::owned());
        }
    }
    let mut incoming = vec![None::<BTreeMap<usize, Fact>>; f.blocks.len()];
    incoming[f.entry] = Some(initial);
    let mut queue = VecDeque::from([f.entry]);
    let mut result = Fact::default();
    while let Some(id) = queue.pop_front() {
        let state = incoming[id].clone().unwrap();
        let mut output = state.clone();
        let borrowed = |p: &Place, readonly: bool| {
            let mut fact = Fact {
                readonly,
                ..Fact::default()
            };
            for origin in provenance.places(id, p) {
                // An unresolved call result can still occupy a value temporary
                // in the first checker pass. Follow its inferred value origins.
                if let Some(value) = state
                    .get(&origin.local)
                    .filter(|v| !v.owned && !v.origins.is_empty())
                {
                    fact.merge(&value.clone().project(&origin.projections));
                } else {
                    fact.origins.insert(origin);
                }
            }
            fact.normalize();
            fact
        };
        let read = |p: &Place| {
            if p.projections.first() == Some(&Projection::Dereference) {
                borrowed(p, f.locals[p.local].view == Some(loans::Access::Readonly))
            } else {
                state
                    .get(&p.local)
                    .cloned()
                    .unwrap_or_default()
                    .project(&p.projections)
            }
        };
        let operand = |v: &Operand| match v {
            Operand::Local(l) => read(&Place::local(*l)),
            Operand::Constant(_) => Fact::owned(),
            Operand::Address(p) => borrowed(
                p,
                f.locals[p.local]
                    .view
                    .map_or(!f.locals[p.local].mutable, |a| a == loans::Access::Readonly),
            ),
        };
        match &f.blocks[id].terminator {
            Terminator::Return(value) => result.merge(&operand(value)),
            Terminator::Evaluate {
                operation,
                destination,
                ..
            } if destination.projections.is_empty() => {
                let fact = match operation {
                    Operation::StorageView {
                        arguments,
                        contract,
                    } => {
                        let mut result = operand(&arguments[0]);
                        result.readonly = contract.access == loans::Access::Readonly;
                        result
                    }
                    Operation::Move(p) => read(p),
                    Operation::Use(v) | Operation::CompleteClass(v) => operand(v),
                    Operation::Call(callee, args)
                    | Operation::CallStorage {
                        function: callee,
                        arguments: args,
                        ..
                    }
                    | Operation::CallView {
                        function: callee,
                        arguments: args,
                        ..
                    } => {
                        let summary = &summaries[*callee];
                        let mut value = Fact {
                            owned: summary.owned,
                            readonly: summary.readonly,
                            origins: BTreeSet::new(),
                        };
                        // Summary roots use parameter positions, independent of
                        // the callee's local numbering.
                        for origin in &summary.origins {
                            if let Some(arg) = args.get(origin.local) {
                                if origin.projections.contains(&Projection::Dereference) {
                                    let mut place = match arg {
                                        Operand::Local(p) => Place::local(*p),
                                        Operand::Address(p) => p.clone(),
                                        Operand::Constant(_) => continue,
                                    };
                                    place.projections.extend_from_slice(&origin.projections);
                                    value.merge(&borrowed(&place, true));
                                } else {
                                    value.merge(&operand(arg).project(&origin.projections));
                                }
                            }
                        }
                        value
                    }
                    Operation::CallIndirect(_, args, contract) => {
                        let mut value = Fact::default();
                        if contract.targets.is_empty() {
                            value = Fact::owned();
                        }
                        for callee in &contract.targets {
                            let summary = &summaries[*callee];
                            let mut next = Fact {
                                owned: summary.owned,
                                readonly: summary.readonly,
                                origins: BTreeSet::new(),
                            };
                            for origin in &summary.origins {
                                if let Some(arg) = args.get(origin.local) {
                                    if origin.projections.contains(&Projection::Dereference) {
                                        let mut place = match arg {
                                            Operand::Local(p) => Place::local(*p),
                                            Operand::Address(p) => p.clone(),
                                            Operand::Constant(_) => continue,
                                        };
                                        place.projections.extend_from_slice(&origin.projections);
                                        next.merge(&borrowed(&place, true));
                                    } else {
                                        next.merge(&operand(arg).project(&origin.projections));
                                    }
                                }
                            }
                            value.merge(&next);
                        }
                        value
                    }
                    Operation::Borrow(p) | Operation::CapturePlace(p) => borrowed(
                        p,
                        f.locals[destination.local].view == Some(loans::Access::Readonly),
                    ),
                    _ => Fact::owned(),
                };
                output.insert(destination.local, fact);
            }
            _ => {}
        }
        for next in &flow.successors[id] {
            let next = *next;
            // A failed operation does not establish a result value. The result
            // analysis follows successful paths; provenance separately models panic.
            if matches!(&f.blocks[id].terminator, Terminator::Evaluate {next: success, ..} if *success != next)
            {
                continue;
            }
            let mut joined = incoming[next].clone().unwrap_or_default();
            for (local, fact) in &output {
                joined.entry(*local).or_default().merge(fact);
            }
            if incoming[next].as_ref() != Some(&joined) {
                incoming[next] = Some(joined);
                queue.push_back(next);
            }
        }
    }
    // Normalize externally observable roots to parameter positions for calls;
    // local roots remain distinguishable beyond the parameter position range.
    result.origins = result
        .origins
        .into_iter()
        .map(|mut p| {
            if let Some(index) = p
                .projections
                .iter()
                .position(|p| matches!(p, Projection::Index(_)))
            {
                p.projections.truncate(index);
            }
            p.local = f
                .parameters
                .iter()
                .position(|id| *id == p.local)
                .unwrap_or(f.parameters.len() + p.local);
            p
        })
        .collect();
    result
}
