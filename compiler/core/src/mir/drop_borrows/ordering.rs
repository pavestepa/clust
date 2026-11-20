use super::*;

pub(in super::super) fn orders(f: &Function, provenance: &borrow_check::Provenance) -> Orders {
    let flow = reachability::analyze(f);
    let mut incoming = vec![None::<State>; f.blocks.len()];
    let mut initial = State::default();
    for p in &f.parameters {
        if f.locals[*p].view.is_none() {
            initial.initialize(*p);
        }
    }
    incoming[f.entry] = Some(initial);
    let mut queue = VecDeque::from([f.entry]);
    while let Some(id) = queue.pop_front() {
        let input = incoming[id].clone().unwrap();
        let term = &f.blocks[id].terminator;
        for next in &flow.successors[id] {
            let mut out = input.clone();
            match term {
                Terminator::Evaluate {
                    operation,
                    destination,
                    next: success,
                    ..
                } => {
                    if *next == *success
                        || storage_writes::call(operation).is_some()
                        || matches!(operation, Operation::CallIndirect(..))
                        || stages_on_unwind(operation)
                    {
                        for source in dataflow::consumed(f, operation) {
                            let targets = provenance.places(id, &source);
                            if targets.len() == 1 {
                                for target in targets {
                                    if target.projections.is_empty() {
                                        out.forget(target.local);
                                    }
                                }
                            }
                        }
                    }
                    if *next != *success && stages_on_unwind(operation) {
                        let targets = provenance.places(id, destination);
                        if targets.len() == 1 {
                            for target in targets {
                                if target.projections.is_empty() {
                                    out.forget(target.local)
                                }
                            }
                        }
                    }
                    if *next == *success {
                        let targets = provenance.places(id, destination);
                        if targets.len() == 1 {
                            for target in targets {
                                if target.projections.is_empty()
                                    && f.locals[target.local].view.is_none()
                                {
                                    out.initialize(target.local);
                                }
                            }
                        } else {
                            // A may-alias write may reinitialize any target. Keep
                            // both possible orders, including the unchanged path.
                            for target in targets {
                                if target.projections.is_empty() {
                                    let mut changed = out.clone();
                                    changed.initialize(target.local);
                                    out.join(&changed);
                                }
                            }
                        }
                    }
                    let mut written = BTreeSet::new();
                    if *next == *success {
                        for target in provenance.places(id, destination) {
                            if !target.projections.is_empty() {
                                written.insert(target.local);
                            }
                        }
                    }
                    if let Some((arguments, writes)) = storage_writes::effects(f, operation) {
                        for write in writes {
                            if let Some(Operand::Address(argument)) =
                                arguments.get(write.target.parameter)
                            {
                                let mut slot = argument.clone();
                                slot.projections.extend_from_slice(&write.target.path);
                                for target in provenance.places(id, &slot) {
                                    written.insert(target.local);
                                }
                            }
                        }
                    }
                    for holder in written {
                        for origins in provenance.components(*next, &Place::local(holder)).values()
                        {
                            for target in origins {
                                if out.before.contains(&(holder, target.local)) {
                                    out.bad.insert((holder, target.local));
                                }
                            }
                        }
                    }
                }
                Terminator::EndStorage { locals, .. } => {
                    for local in locals {
                        out.forget(*local)
                    }
                }
                _ => {}
            }
            let changed = match &mut incoming[*next] {
                Some(old) => old.join(&out),
                slot @ None => {
                    *slot = Some(out);
                    true
                }
            };
            if changed {
                queue.push_back(*next);
            }
        }
    }
    Orders { incoming }
}
