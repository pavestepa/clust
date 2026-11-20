use super::*;

pub(super) fn operand(value: &Operand, locals: &[Slot]) -> Result<Value, TrapKind> {
    match value {
        Operand::Address(_) => Err(TrapKind::InvalidIr),
        Operand::Constant(value) => Ok(value.clone()),
        Operand::Local(local) => locals
            .get(*local)
            .and_then(|slot| slot.borrow().clone())
            .ok_or(TrapKind::InvalidIr),
    }
}

pub(super) fn indices(place: &Place, locals: &[Slot]) -> Result<Vec<Projection>, TrapKind> {
    place
        .projections
        .iter()
        .map(|p| match p {
            Projection::Index(local) => {
                let Value::Integer(index) = operand(&Operand::Local(*local), locals)? else {
                    return Err(TrapKind::InvalidIr);
                };
                Ok(Projection::Field(
                    usize::try_from(index.bits()).map_err(|_| TrapKind::Bounds)?,
                ))
            }
            Projection::ConstantIndex(index) => Ok(Projection::Field(
                usize::try_from(*index).map_err(|_| TrapKind::Bounds)?,
            )),
            other => Ok(other.clone()),
        })
        .collect()
}

pub(super) fn resolve(
    place: &Place,
    views: &Views,
    locals: &[Slot],
) -> Result<Reference, TrapKind> {
    let result = if matches!(place.projections.first(), Some(Projection::Dereference)) {
        let mut origin = views
            .get(&place.local)
            .cloned()
            .ok_or(TrapKind::InvalidIr)?;
        origin.projections.extend(indices(
            &Place {
                local: place.local,
                projections: place.projections[1..].to_vec(),
            },
            locals,
        )?);
        origin
    } else {
        Reference {
            slot: locals
                .get(place.local)
                .cloned()
                .ok_or(TrapKind::InvalidIr)?,
            projections: indices(place, locals)?,
        }
    };
    result.normalized()
}
