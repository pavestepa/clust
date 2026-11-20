use super::*;

pub(super) fn resolve(place: &Place, bindings: &Bindings, origins: &[Origin]) -> Origin {
    let mut result = Origin {
        places: BTreeSet::from([Place::local(place.local)]),
        ..Origin::default()
    };
    for projection in &place.projections {
        if *projection != Projection::Dereference {
            result.places = result
                .places
                .into_iter()
                .map(|mut p| {
                    p.projections.push(projection.clone());
                    p
                })
                .collect();
            continue;
        }
        let mut next = Origin::default();
        let mut ancestors: Option<BTreeSet<BlockId>> = None;
        for source in &result.places {
            for loan in bindings
                .iter()
                .filter(|(p, _)| {
                    p.projections.len() == source.projections.len() && compatible_prefix(source, p)
                })
                .flat_map(|(_, loans)| loans)
            {
                let mut through = origins[*loan].ancestors.clone();
                through.insert(*loan);
                through.extend(&result.ancestors);
                ancestors = Some(match ancestors {
                    Some(old) => old.intersection(&through).copied().collect(),
                    None => through,
                });
                next.parents.insert(*loan);
                next.parents.extend(&origins[*loan].parents);
                next.parents.extend(&result.parents);
                next.places.extend(origins[*loan].places.iter().cloned());
            }
        }
        next.ancestors = ancestors.unwrap_or_default();
        result = next;
    }
    result
}
