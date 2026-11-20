use super::*;

impl Reference {
    pub fn normalized(&self) -> Result<Self, TrapKind> {
        let mut result = Self {
            slot: self.slot.clone(),
            projections: vec![],
        };
        for projection in &self.projections {
            if let Value::Class(value) = result.read_raw()? {
                result = Self {
                    slot: value.payload,
                    projections: vec![],
                };
            }
            result.projections.push(projection.clone());
        }
        Ok(result)
    }
}

impl Reference {
    pub fn captured(&self) -> Result<Self, TrapKind> {
        if let Value::Class(value) = self.read()? {
            Ok(class_reference(value, self.slot.0.clock.clone()))
        } else {
            Ok(self.clone())
        }
    }
}

impl Reference {
    pub fn read(&self) -> Result<Value, TrapKind> {
        self.normalized()?.read_raw()
    }
}

impl Reference {
    pub fn parts(&self) -> Vec<crate::mir::drop_plan::Part> {
        use crate::mir::drop_plan::Part;
        let mut ty = self.slot.0.ty.clone();
        let mut parts = Vec::new();
        for projection in &self.projections {
            match (projection, &ty) {
                (Projection::Field(i), Type::Array(element, _)) => {
                    parts.push(Part::Element(*i as u64));
                    ty = (**element).clone();
                }
                (Projection::Field(i), Type::Record { fields, .. }) => {
                    parts.push(Part::Field(*i));
                    ty = fields[*i].ty.clone();
                }
                (Projection::Field(i), Type::Tuple(fields)) => {
                    parts.push(Part::Field(*i));
                    ty = fields[*i].clone();
                }
                (Projection::Variant(i), Type::Enum { variants, .. }) => {
                    parts.push(Part::Variant(*i));
                    ty = Type::Tuple(variants[*i].fields.clone());
                }
                (Projection::Field(i), _) => parts.push(Part::Field(*i)),
                _ => unreachable!("reference must have resolved projections"),
            }
        }
        parts
    }
}

impl Reference {
    pub fn projected(&self, parts: &[crate::mir::drop_plan::Part]) -> Self {
        use crate::mir::drop_plan::Part;
        let mut result = self.clone();
        result
            .projections
            .extend(parts.iter().map(|part| match part {
                Part::Field(i) => Projection::Field(*i),
                Part::Element(i) => Projection::Field(*i as usize),
                Part::Variant(i) => Projection::Variant(*i),
            }));
        result.normalized().expect("checked drop path")
    }
}

impl Reference {
    pub fn forget(&self) -> Result<(), TrapKind> {
        self.slot
            .state()
            .borrow_mut()
            .forget(&self.parts())
            .map_err(|_| TrapKind::InvalidIr)
    }
}

impl Reference {
    pub(super) fn read_raw(&self) -> Result<Value, TrapKind> {
        let stored = self.slot.borrow();
        let mut value = stored.as_ref().ok_or(TrapKind::InvalidIr)?;
        let mut path = self.projections.as_slice();
        while let Some((first, rest)) = path.split_first() {
            match (first, value) {
                (Projection::Field(index), Value::Aggregate(fields)) => {
                    value = fields.get(*index).ok_or(TrapKind::Bounds)?;
                    path = rest;
                }
                (Projection::Variant(expected), Value::Enum { variant, fields })
                    if expected == variant =>
                {
                    if rest.is_empty() {
                        return Ok(Value::Aggregate(fields.clone()));
                    }
                    let Some((Projection::Field(index), tail)) = rest.split_first() else {
                        return Err(TrapKind::InvalidIr);
                    };
                    value = fields.get(*index).ok_or(TrapKind::Bounds)?;
                    path = tail;
                }
                _ => return Err(TrapKind::InvalidIr),
            }
        }
        Ok(value.clone())
    }
}

impl Reference {
    pub fn write(&self, value: Value) -> Result<(), TrapKind> {
        if self.projections.is_empty() {
            self.slot.initialize()?;
            *self.slot.borrow_mut() = Some(value);
            return Ok(());
        }
        self.slot
            .state()
            .borrow_mut()
            .initialize(&self.parts())
            .map_err(|_| TrapKind::InvalidIr)?;
        write(
            self.slot.borrow_mut().as_mut().ok_or(TrapKind::InvalidIr)?,
            &self.projections,
            value,
        )
    }
}

pub(super) fn write(target: &mut Value, path: &[Projection], value: Value) -> Result<(), TrapKind> {
    let Some((first, rest)) = path.split_first() else {
        *target = value;
        return Ok(());
    };
    match (first, target) {
        (Projection::Field(index), Value::Aggregate(fields)) => {
            write(fields.get_mut(*index).ok_or(TrapKind::Bounds)?, rest, value)
        }
        (Projection::Variant(expected), Value::Enum { variant, fields }) if expected == variant => {
            if let Some((Projection::Field(index), tail)) = rest.split_first() {
                write(fields.get_mut(*index).ok_or(TrapKind::Bounds)?, tail, value)
            } else if rest.is_empty() {
                let Value::Aggregate(v) = value else {
                    return Err(TrapKind::InvalidIr);
                };
                *fields = v;
                Ok(())
            } else {
                Err(TrapKind::InvalidIr)
            }
        }
        _ => Err(TrapKind::InvalidIr),
    }
}
