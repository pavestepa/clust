use super::*;

pub(super) fn projection_byte_offset(ty: &Type, path: &[Projection]) -> Option<u64> {
    let Some((head, tail)) = path.split_first() else {
        return Some(0);
    };
    let (offset, child) = match (ty, head) {
        (Type::Array(element, _), Projection::Field(index)) => (
            crate::layout::layout(element)
                .ok()?
                .size()
                .checked_mul(*index as u64)?,
            (**element).clone(),
        ),
        (Type::Record { fields, .. }, Projection::Field(index)) => {
            let layout = clust_semantics::AggregateLayout::new(
                &fields
                    .iter()
                    .map(|f| crate::layout::layout(&f.ty))
                    .collect::<Result<Vec<_>, _>>()
                    .ok()?,
            )
            .ok()?;
            (
                *layout.field_offsets.get(*index)?,
                fields.get(*index)?.ty.clone(),
            )
        }
        (Type::Tuple(fields), Projection::Field(index)) => {
            let layout = clust_semantics::AggregateLayout::new(
                &fields
                    .iter()
                    .map(crate::layout::layout)
                    .collect::<Result<Vec<_>, _>>()
                    .ok()?,
            )
            .ok()?;
            (
                *layout.field_offsets.get(*index)?,
                fields.get(*index)?.clone(),
            )
        }
        (Type::Enum { variants, .. }, Projection::Variant(index)) => {
            let layout = clust_semantics::EnumLayout::new(
                &variants
                    .iter()
                    .map(|v| crate::layout::layout(&Type::Tuple(v.fields.clone())))
                    .collect::<Result<Vec<_>, _>>()
                    .ok()?,
            )
            .ok()?;
            (
                layout.payload_offset,
                Type::Tuple(variants.get(*index)?.fields.clone()),
            )
        }
        _ => return None,
    };
    offset.checked_add(projection_byte_offset(&child, tail)?)
}
