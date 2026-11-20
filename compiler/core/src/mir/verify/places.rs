use super::*;

/// Physical place type, including stored descriptors. A local view has an
/// implicit pointer layer; projections must explicitly cross it.
pub(crate) fn place_type(f: &Function, place: &Place) -> Option<Type> {
    let local = f.locals.get(place.local)?;
    let mut ty = local.ty.clone();
    for (index, projection) in place.projections.iter().enumerate() {
        ty = match (projection, ty) {
            (Projection::Dereference, ty) if index == 0 && local.view.is_some() => ty,
            (Projection::Dereference, Type::StoredView(target)) => *target,
            (Projection::Field(i), Type::Tuple(fields)) => fields.get(*i)?.clone(),
            (Projection::Field(i), Type::Record { fields, .. }) => fields.get(*i)?.ty.clone(),
            (Projection::Field(i), ty @ Type::Class { .. }) => {
                f.classes.get(&ty.class_key())?.fields.get(*i)?.ty.clone()
            }
            (Projection::Variant(i), Type::Enum { variants, .. }) => {
                Type::Tuple(variants.get(*i)?.fields.clone())
            }
            (Projection::ConstantIndex(_), Type::Array(element, _)) => *element,
            (Projection::Index(i), Type::Array(element, _))
                if matches!(f.locals.get(*i)?.ty, Type::Integer(_)) =>
            {
                *element
            }
            _ => return None,
        };
    }
    Some(ty)
}
