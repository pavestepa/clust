use super::*;

impl Emitter {
    pub(super) fn constant(&self, value: &Value) -> String {
        match value {
            Value::Unit => "()".into(),
            Value::Bool(v) => v.to_string(),
            Value::Char(v) => format!("{v:?}"),
            Value::Integer(v) => {
                format!("({}u128 as {})", v.bits(), self.ty(&Type::Integer(v.ty())))
            }
            Value::F32(v) => format!("f32::from_bits({})", v.to_bits()),
            Value::F64(v) => format!("f64::from_bits({})", v.to_bits()),
            Value::Str(v) => format!("{v:?}"),
            Value::Function(id) => format!("f{id}"),
            _ => unreachable!("aggregate constants must be lowered to construction"),
        }
    }
}

impl Emitter {
    pub(super) fn aggregate(&self, values: &[String], ty: &Type) -> String {
        match ty {
            Type::Array(_, _) => format!("[{}]", values.join(",")),
            Type::Unit => "()".into(),
            _ => format!(
                "{} {{ {} }}",
                self.ty(ty),
                values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| format!("f{i}: {v},"))
                    .collect::<String>()
            ),
        }
    }
}

impl Emitter {
    pub(super) fn enum_value(&self, variant: usize, values: &[String], ty: &Type) -> String {
        let id = self.types.iter().position(|t| t == ty).unwrap();
        let Type::Enum { variants, .. } = ty else {
            unreachable!()
        };
        let payload = self.aggregate(values, &Type::Tuple(variants[variant].fields.clone()));
        format!("T{id} {{ tag: {variant}, data: U{id} {{ v{variant}: ManuallyDrop::new({payload}) }} }}")
    }
}

impl Emitter {
    pub(super) fn typed_constant(&self, value: &Value, ty: &Type) -> String {
        match value {
            Value::Aggregate(values) => {
                let fields = match ty {
                    Type::Array(element, _) => vec![(**element).clone(); values.len()],
                    Type::Tuple(fields) => fields.clone(),
                    Type::Record { fields, .. } => fields.iter().map(|f| f.ty.clone()).collect(),
                    _ => unreachable!(),
                };
                self.aggregate(
                    &values
                        .iter()
                        .zip(fields)
                        .map(|(v, t)| self.typed_constant(v, &t))
                        .collect::<Vec<_>>(),
                    ty,
                )
            }
            Value::Enum { variant, fields } => {
                let Type::Enum { variants, .. } = ty else {
                    unreachable!()
                };
                self.enum_value(
                    *variant,
                    &fields
                        .iter()
                        .zip(&variants[*variant].fields)
                        .map(|(v, t)| self.typed_constant(v, t))
                        .collect::<Vec<_>>(),
                    ty,
                )
            }
            _ => self.constant(value),
        }
    }
}
