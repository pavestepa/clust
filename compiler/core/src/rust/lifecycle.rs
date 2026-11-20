//! Explicit resource state and typed drop glue. Payloads never implement Rust
//! Drop: Clust's checked moves and cleanup boundaries control their lifetime.
use super::*;
impl Emitter {
    pub(super) fn resource(&self, ty: &Type) -> bool {
        self.destructors.plan(ty) != mir::drop_plan::Plan::None
    }
    pub(super) fn type_id(&self, ty: &Type) -> usize {
        self.types.iter().position(|t| t == ty).unwrap()
    }
    pub(super) fn view(&self, place: &Place, f: &Function) -> String {
        let (data, ty) = self.place(place, f);
        if !self.resource(&ty) {
            return data;
        }
        if matches!(ty, Type::Class { .. }) {
            return format!("V{} {{data:({data}).read()}}", self.type_id(&ty));
        }
        let (state, stamp) = self.state_place(place, f);
        format!(
            "V{} {{data:{data},state:{state},stamp:{stamp}}}",
            self.type_id(&ty)
        )
    }
    pub(super) fn state_place(&self, place: &Place, f: &Function) -> (String, String) {
        let mut ty = f.locals[place.local].ty.clone();
        let mut state = format!("s{}", place.local);
        let mut stamp = format!("std::ptr::addr_of_mut!(stamp{})", place.local);
        for (index, projection) in place.projections.iter().enumerate() {
            match projection {
                Projection::Dereference
                    if matches!(ty, Type::Class { .. }) && !self.captured_slot(f, place.local) =>
                {
                    // A class view stores heap identity, never the owner's slot flags.
                    state = "std::ptr::null_mut()".into();
                    stamp = "std::ptr::null_mut()".into();
                }
                Projection::Dereference => {
                    state = format!("l{}.read().state", place.local);
                    stamp = format!("l{}.read().stamp", place.local);
                }
                Projection::Field(i) => {
                    let class_field = matches!(ty, Type::Class { .. });
                    stamp = if class_field {
                        let prefix = Place {
                            local: place.local,
                            projections: place.projections[..index].to_vec(),
                        };
                        let data = self.place(&prefix, f).0;
                        format!("std::ptr::addr_of_mut!((*({data}).read()).order[{i}])")
                    } else {
                        "std::ptr::null_mut()".into()
                    };
                    state = if matches!(ty, Type::Class { .. }) {
                        let prefix = Place {
                            local: place.local,
                            projections: place.projections[..index].to_vec(),
                        };
                        let data = self.place(&prefix, f).0;
                        format!("std::ptr::addr_of_mut!((*({data}).read()).state.f{i})")
                    } else {
                        format!("std::ptr::addr_of_mut!((*({state})).f{i})")
                    };
                    ty = match ty {
                        Type::Class { .. } => self.classes[&ty.class_key()].fields[*i].ty.clone(),
                        Type::Record { fields, .. } => fields[*i].ty.clone(),
                        Type::Tuple(fields) => fields[*i].clone(),
                        _ => unreachable!(),
                    };
                }
                Projection::Variant(i) => {
                    state = format!("std::ptr::addr_of_mut!((*({state})).v{i})");
                    let Type::Enum { variants, .. } = ty else {
                        unreachable!()
                    };
                    ty = Type::Tuple(variants[*i].fields.clone());
                    stamp = "std::ptr::null_mut()".into();
                }
                Projection::Index(_) | Projection::ConstantIndex(_) => {
                    let index = match projection {
                        Projection::Index(i) => format!("l{i}.read() as usize"),
                        Projection::ConstantIndex(i) => format!("{i}usize"),
                        _ => unreachable!(),
                    };
                    state = format!(
                        "std::ptr::addr_of_mut!((*({state})).elements).cast::<S{}>().add({index})",
                        match &ty {
                            Type::Array(t, _) => self.type_id(t),
                            _ => unreachable!(),
                        }
                    );
                    let Type::Array(element, _) = ty else {
                        unreachable!()
                    };
                    ty = *element;
                    stamp = "std::ptr::null_mut()".into();
                }
            }
        }
        (state, stamp)
    }
    pub(super) fn forget(&self, place: &Place, f: &Function) -> String {
        let ty = self.place(place, f).1;
        if !self.resource(&ty) {
            return String::new();
        }
        let (state, _) = self.state_place(place, f);
        format!("(*({state})).live=false;\n")
    }
}

mod resources;
mod runtime_types;
mod slots;
