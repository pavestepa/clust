use super::*;

/// May-live owner roots must reach a storage boundary or the return operand on
/// every exit, including unwind. Partial moves keep their enclosing root live.
pub(super) fn cleanup_exits(f: &Function) -> Result<(), Diagnostic> {
    use std::collections::VecDeque;
    let owns = |l: usize| f.locals[l].view.is_none() && f.locals[l].ty.owns_storage();
    let mut incoming: Vec<Option<BTreeSet<usize>>> = vec![None; f.blocks.len()];
    incoming[f.entry] = Some(f.parameters.iter().copied().filter(|l| owns(*l)).collect());
    let mut queue = VecDeque::from([f.entry]);
    while let Some(id) = queue.pop_front() {
        let mut live = incoming[id].clone().unwrap();
        let term = &f.blocks[id].terminator;
        if let Terminator::EndStorage { locals, .. } = term {
            for l in locals {
                live.remove(l);
            }
        }
        if let Terminator::Return(Operand::Local(l)) = term {
            live.remove(l);
        }
        if matches!(term, Terminator::Return(_) | Terminator::ResumePanic) && !live.is_empty() {
            return Err(Diagnostic::new(
                "E0401",
                "MIR exit leaks an ownership token without cleanup",
                f.blocks[id].span,
            ));
        }
        for successor in term.successors() {
            let mut next_live = live.clone();
            if let Terminator::Evaluate {
                operation,
                destination,
                next,
                ..
            } = term
            {
                let call = matches!(
                    operation,
                    Operation::Call(..)
                        | Operation::CallIndirect(..)
                        | Operation::CallView { .. }
                        | Operation::CallStorage { .. }
                );
                if successor == *next || call {
                    for p in dataflow::consumed(f, operation) {
                        if p.projections.is_empty() {
                            next_live.remove(&p.local);
                        }
                    }
                }
                if successor == *next && owns(destination.local) {
                    next_live.insert(destination.local);
                }
            }
            match &mut incoming[successor] {
                Some(old) => {
                    let len = old.len();
                    old.extend(next_live);
                    if old.len() != len {
                        queue.push_back(successor);
                    }
                }
                slot @ None => {
                    *slot = Some(next_live);
                    queue.push_back(successor);
                }
            }
        }
    }
    Ok(())
}

pub(super) fn writable(f: &Function, p: &Place) -> bool {
    let l = &f.locals[p.local];
    let root = if p.projections.first() == Some(&Projection::Dereference) {
        l.view == Some(loans::Access::Mutable)
    } else {
        l.mutable
    };
    root && p.projections.iter().enumerate().all(|(i, projection)| {
        *projection != Projection::Dereference
            || i == 0
            || !matches!(
                place_type(
                    f,
                    &Place {
                        local: p.local,
                        projections: p.projections[..i].to_vec()
                    }
                ),
                Some(Type::StoredView(_))
            )
    })
}
