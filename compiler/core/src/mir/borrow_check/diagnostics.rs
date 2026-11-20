use super::*;

/// Attach the target, creation site and a reachable later use to a loan error.
/// Source positions survive imports and specialization; MIR local IDs do not
/// appear in user-facing diagnostics.
pub fn diagnostic(
    function: &Function,
    error: Error,
    code: &'static str,
) -> clust_frontend::source::Diagnostic {
    let mut result = clust_frontend::source::Diagnostic::new(
        code,
        error.message,
        function.blocks[error.block].span,
    );
    let Some(loan) = error.loan else {
        return result;
    };
    let provenance = provenance(function);
    for origin in &provenance.origins[loan].places {
        let local = &function.locals[origin.local];
        result = result.with_note(
            local.span,
            format!("borrowed target '{}' originates here", local.name),
        );
    }
    if let Some(block) = provenance.sites[loan].and_then(|site| function.blocks.get(site)) {
        result = result.with_note(block.span, "this view is created here");
    }
    let mut queue = VecDeque::from(function.blocks[error.block].terminator.successors());
    let mut seen = BTreeSet::from([error.block]);
    while let Some(id) = queue.pop_front() {
        if !seen.insert(id) {
            continue;
        }
        if let Some(bindings) = &provenance.incoming[id] {
            if dataflow::uses(&function.blocks[id].terminator)
                .iter()
                .any(|local| {
                    bindings
                        .iter()
                        .filter(|(place, _)| place.local == *local)
                        .flat_map(|(_, loans)| loans)
                        .any(|source| {
                            *source == loan || provenance.origins[*source].parents.contains(&loan)
                        })
                })
            {
                result = result.with_note(
                    function.blocks[id].span,
                    "this later use keeps the view live",
                );
                break;
            }
        }
        queue.extend(function.blocks[id].terminator.successors());
    }
    result
}

pub(super) fn covers(parent: &Place, child: &Place) -> bool {
    parent.local == child.local
        && parent.projections.len() <= child.projections.len()
        && parent
            .projections
            .iter()
            .zip(&child.projections)
            .all(|(a, b)| a == b)
}

pub(super) fn compatible_prefix(parent: &Place, child: &Place) -> bool {
    parent.local == child.local
        && parent.projections.len() <= child.projections.len()
        && parent
            .projections
            .iter()
            .zip(&child.projections)
            .all(|(a, b)| {
                a == b
                    || matches!(
                        (a, b),
                        (Projection::Index(_), Projection::ConstantIndex(_))
                            | (Projection::ConstantIndex(_), Projection::Index(_))
                            | (Projection::Index(_), Projection::Index(_))
                    )
            })
}
