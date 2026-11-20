use super::*;

/// Execute one recipe. `pending` retains the first panic while remaining fields
/// are cleaned. A second panic ends execution immediately; native glue aborts.
pub fn execute<M: Memory>(
    plan: &Plan,
    memory: &mut M,
    pending: &mut Option<M::Panic>,
) -> Result<(), Failure<M::Panic>> {
    execute_at(plan, memory, pending, &mut Vec::new())
}

pub(super) fn execute_at<M: Memory>(
    plan: &Plan,
    memory: &mut M,
    pending: &mut Option<M::Panic>,
    path: &mut Vec<Part>,
) -> Result<(), Failure<M::Panic>> {
    if !plan.needs_drop() {
        return Ok(());
    }
    let state = memory.initialization(path);
    match state {
        Initialization::Vacant => return Ok(()),
        Initialization::Finalizing => return Err(Failure::InvalidLifecycle),
        _ => {}
    }
    // Snapshot the tag before beginning cleanup. No tag read occurs for a vacant
    // enum, even if its backing bytes have never held a valid discriminant.
    let variant = if matches!(plan, Plan::Enum { .. }) {
        memory.variant(path)
    } else {
        None
    };
    if matches!(
        plan,
        Plan::Aggregate {
            destructor: Some(_),
            ..
        }
    ) && state == Initialization::Partial
    {
        return Err(Failure::InvalidLifecycle);
    }
    memory.begin(path);
    match plan {
        Plan::Class(name) => memory.class(name, path, pending)?,
        Plan::None => unreachable!(),
        Plan::Aggregate { destructor, fields } => {
            if let Some(entry) = destructor.filter(|_| state == Initialization::Live) {
                if let Err(second) = memory.destructor(entry, path) {
                    if let Some(first) = pending.take() {
                        return Err(Failure::DoublePanic { first, second });
                    }
                    *pending = Some(second);
                }
            }
            let mut ordered: Vec<_> = fields.iter().collect();
            if state == Initialization::Constructing {
                ordered.sort_by_key(|(index, _)| {
                    path.push(Part::Field(*index));
                    let order = memory.initialization_order(path);
                    path.pop();
                    std::cmp::Reverse(order)
                });
            }
            for (index, field) in ordered {
                path.push(Part::Field(*index));
                execute_at(field, memory, pending, path)?;
                path.pop();
            }
        }
        Plan::Array { element, count } => {
            for index in 0..*count {
                path.push(Part::Element(index));
                execute_at(element, memory, pending, path)?;
                path.pop();
            }
        }
        Plan::Enum { variants } => {
            let Some(variant) = variant else {
                if state != Initialization::Constructing {
                    return Err(Failure::InvalidVariant);
                }
                memory.finish(path);
                return Ok(());
            };
            let active = variants.get(variant).ok_or(Failure::InvalidVariant)?;
            path.push(Part::Variant(variant));
            execute_at(active, memory, pending, path)?;
            path.pop();
        }
    }
    memory.finish(path);
    Ok(())
}
