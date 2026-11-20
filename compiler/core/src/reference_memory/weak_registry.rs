use super::*;

impl WeakRegistry {
    pub fn create(&mut self, value: ClassValue) -> Result<Value, TrapKind> {
        let value = value.actual_identity();
        let Type::Record { name: ty, .. } = value.payload.ty() else {
            return Err(TrapKind::InvalidIr);
        };
        let ty = ty.clone();
        let existing = self
            .entries
            .iter()
            .find(|(_, (weak, _))| {
                weak.upgrade()
                    .is_some_and(|slot| Rc::ptr_eq(&slot, &value.payload.0))
            })
            .map(|(id, _)| *id);
        let id = if value.payload.0.finalizing.get() {
            0
        } else if let Some(id) = existing {
            id
        } else {
            self.next = self.next.checked_add(1).ok_or(TrapKind::Allocation)?;
            self.entries
                .insert(self.next, (Rc::downgrade(&value.payload.0), ty));
            self.next
        };
        Ok(Value::Aggregate(vec![
            Value::Integer(
                clust_semantics::Integer::from_bits(
                    clust_semantics::IntegerType::Usize,
                    id as u128
                )
            );
            2
        ]))
    }
}

impl WeakRegistry {
    pub fn get(&self, token: &Value, expected: Option<&Type>) -> Option<ClassValue> {
        let Value::Aggregate(values) = token else {
            return None;
        };
        let [Value::Integer(id), Value::Integer(generation)] = values.as_slice() else {
            return None;
        };
        if id != generation {
            return None;
        }
        let (weak, ty) = self.entries.get(&usize::try_from(id.bits()).ok()?)?;
        if expected.is_some_and(|expected| expected.class_key() != *ty) {
            return None;
        }
        let slot = weak.upgrade()?;
        if slot.value.borrow().is_none() {
            return None;
        }
        Some(ClassValue {
            payload: Slot(slot),
        })
    }
}

impl WeakRegistry {
    pub fn access(
        &self,
        token: &Value,
        expected: &Type,
        classes: &std::collections::BTreeMap<String, crate::hir::ClassDefinition>,
        clock: Clock,
    ) -> Option<(ClassValue, bool)> {
        let value = self.get(token, None)?;
        let Type::Record { name: actual, .. } = value.payload.ty() else {
            return None;
        };
        if expected.class_key() == *actual {
            return Some((value, false));
        }
        let Type::Class { name, .. } = expected else {
            return None;
        };
        if !name.starts_with("$poly$") {
            return None;
        }
        let definition = classes.get(&expected.class_key())?;
        let Type::Enum { variants, .. } = &definition.fields[0].ty else {
            return None;
        };
        let index=variants.iter().position(|variant|matches!(variant.fields.first(),Some(ty@Type::Class {..}) if ty.class_key()==*actual))?;
        let wrapper = allocate_class(&expected.class_key(), definition, clock);
        *wrapper.payload.borrow_mut() = Some(Value::Aggregate(vec![Value::Enum {
            variant: index,
            fields: vec![Value::Class(value)],
        }]));
        complete_class(&Value::Class(wrapper.clone())).ok()?;
        Some((wrapper, true))
    }
}

impl WeakRegistry {
    pub fn invalidate(&mut self, value: &ClassValue) {
        value.payload.0.finalizing.set(true);
        self.entries.retain(|_, (weak, _)| {
            weak.upgrade()
                .is_some_and(|slot| !Rc::ptr_eq(&slot, &value.payload.0))
        });
    }
}
