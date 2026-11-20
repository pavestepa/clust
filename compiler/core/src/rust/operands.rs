use super::*;

impl Emitter {
    pub(super) fn operand(&self, value: &Operand) -> String {
        match value {
            Operand::Local(id) => format!("l{id}.read()"),
            Operand::Constant(v) => self.constant(v),
            Operand::Address(_) => unreachable!("address operand requires function context"),
        }
    }
}

impl Emitter {
    pub(super) fn operand_ty(&self, value: &Operand, f: &Function) -> Type {
        match value {
            Operand::Local(id) => f.locals[*id].ty.clone(),
            Operand::Address(p) => self.place(p, f).1,
            Operand::Constant(v) => match v {
                Value::Integer(v) => Type::Integer(v.ty()),
                Value::Bool(_) => Type::Bool,
                Value::F32(_) => Type::F32,
                Value::F64(_) => Type::F64,
                Value::Char(_) => Type::Char,
                Value::Str(_) => Type::Str,
                Value::Unit => Type::Unit,
                _ => unreachable!(),
            },
        }
    }
}

impl Emitter {
    pub(super) fn place(&self, place: &Place, f: &Function) -> (String, Type) {
        let mut ty = f.locals[place.local].ty.clone();
        let mut address = format!("l{}", place.local);
        for p in &place.projections {
            match p {
                Projection::Dereference => {
                    address =
                        if matches!(ty, Type::Class { .. }) && !self.captured_slot(f, place.local) {
                            format!("std::ptr::addr_of_mut!((*({address})).data)")
                        } else if self.resource(&ty) {
                            format!("({address}).read().data")
                        } else {
                            format!("({address}).read()")
                        }
                }
                Projection::Field(i) => {
                    if let Type::Class { .. } = &ty {
                        address = format!(
                            "std::ptr::addr_of_mut!((*({address}).read()).data).cast::<{}>()",
                            self.ty(&self.class_payload(&ty.class_key()))
                        );
                    }
                    let (field, field_ty) = match &ty {
                        Type::Class { .. } => (
                            format!("f{i}"),
                            self.classes[&ty.class_key()].fields[*i].ty.clone(),
                        ),
                        Type::Record { fields, .. } => (format!("f{i}"), fields[*i].ty.clone()),
                        Type::Tuple(fields) => (format!("f{i}"), fields[*i].clone()),
                        _ => unreachable!(),
                    };
                    address = format!("std::ptr::addr_of_mut!((*({address})).{field})");
                    ty = field_ty;
                }
                Projection::Variant(i) => {
                    let Type::Enum { variants, .. } = &ty else {
                        unreachable!()
                    };
                    ty = Type::Tuple(variants[*i].fields.clone());
                    address = format!(
                        "std::ptr::addr_of_mut!((*({address})).data.v{i}).cast::<{}>()",
                        self.ty(&ty)
                    );
                }
                Projection::Index(_) | Projection::ConstantIndex(_) => {
                    let Type::Array(element, count) = &ty else {
                        unreachable!()
                    };
                    let index = match p {
                        Projection::Index(id) => format!("l{id}.read()"),
                        Projection::ConstantIndex(n) => format!("{n}usize"),
                        _ => unreachable!(),
                    };
                    address = format!("{{ let index = {index}; if index >= {count} {{ return Err(fault); }} ({address}).cast::<{}>().add(index) }}", self.ty(element));
                    ty = (**element).clone();
                }
            }
        }
        (address, ty)
    }
}
