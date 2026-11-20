// The pointer binding of a borrowed parameter survives replacement of its
// pointee. Its component bindings describe external storage in the same map.
use super::*;

pub(super) fn transfer(
    source: &Place,
    target: &Place,
    input: &Bindings,
    origins: &[Origin],
    output: &mut Bindings,
) {
    for source in resolve(source, input, origins).places {
        for (place, loans) in input {
            if compatible_prefix(&source, place) {
                let mut field = target.clone();
                field
                    .projections
                    .extend_from_slice(&place.projections[source.projections.len()..]);
                output.entry(field).or_default().extend(loans);
            }
        }
    }
}

pub(super) fn call_origin(
    arguments: &[Operand],
    paths: &BTreeSet<returns::Origin>,
    bindings: &Bindings,
    origins: &[Origin],
) -> Origin {
    let mut result = Origin::default();
    let mut ancestors: Option<BTreeSet<BlockId>> = None;
    for path in paths {
        let mut argument = match arguments.get(path.parameter) {
            Some(Operand::Address(p)) => p.clone(),
            Some(Operand::Local(p)) => Place::local(*p),
            _ => continue,
        };
        argument.projections.extend_from_slice(&path.path);
        let source = resolve(&argument, bindings, origins);
        ancestors = Some(match ancestors {
            Some(old) => old.intersection(&source.ancestors).copied().collect(),
            None => source.ancestors.clone(),
        });
        result.parents.extend(source.parents);
        result.places.extend(source.places);
    }
    result.ancestors = ancestors.unwrap_or_default();
    result
}

pub(super) fn parameter_slot(function: &Function, place: &Place) -> bool {
    place.projections.is_empty()
        && function.parameters.contains(&place.local)
        && function.locals[place.local].view.is_some()
}
