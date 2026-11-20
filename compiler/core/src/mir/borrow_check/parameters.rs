// A parameter's descriptor components refer to external targets even when the
// wrapper itself is passed by value. The symbolic Dereference separates the
// target lifetime from the lifetime of the wrapper slot.
use super::*;

pub(super) fn parameter_loans(function: &Function) -> Vec<(Place, Place, Access)> {
    fn components(ty: &Type, slot: Place, out: &mut Vec<(Place, Place, Access)>) {
        match ty {
            Type::StoredView(target) => {
                let mut origin = slot.clone();
                origin.projections.push(Projection::Dereference);
                out.push((slot, origin.clone(), Access::Readonly));
                components(target, origin, out);
            }
            Type::Record { fields, .. }
            | Type::Class {
                storage: Some(fields),
                ..
            } => {
                for (i, field) in fields.iter().enumerate() {
                    let mut place = slot.clone();
                    place.projections.push(Projection::Field(i));
                    components(&field.ty, place, out);
                }
            }
            Type::Tuple(fields) => {
                for (i, ty) in fields.iter().enumerate() {
                    let mut place = slot.clone();
                    place.projections.push(Projection::Field(i));
                    components(ty, place, out);
                }
            }
            Type::Array(ty, n) if ty.contains_views() => {
                for i in 0..*n {
                    let mut place = slot.clone();
                    place.projections.push(Projection::ConstantIndex(i));
                    components(ty, place, out);
                }
            }
            Type::Enum { variants, .. } => {
                for (i, variant) in variants.iter().enumerate() {
                    let mut place = slot.clone();
                    place.projections.push(Projection::Variant(i));
                    components(&Type::Tuple(variant.fields.clone()), place, out);
                }
            }
            _ => (),
        }
    }
    let mut loans = Vec::new();
    for p in &function.parameters {
        if let Some(access) = function.locals[*p].view {
            loans.push((Place::local(*p), Place::local(*p), access));
        }
        components(&function.locals[*p].ty, Place::local(*p), &mut loans);
    }
    loans
}

pub(super) fn merge(target: &mut Bindings, source: &Bindings) -> bool {
    let mut changed = false;
    for (local, loans) in source {
        let entry = target.entry(local.clone()).or_default();
        let before = entry.len();
        entry.extend(loans);
        changed |= entry.len() != before;
    }
    changed
}

impl Provenance {
    pub(crate) fn places(&self, block: BlockId, place: &Place) -> BTreeSet<Place> {
        self.incoming[block]
            .as_ref()
            .map_or_else(BTreeSet::new, |bindings| {
                resolve(place, bindings, &self.origins).places
            })
    }
}
