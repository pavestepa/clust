use super::*;

pub(crate) fn allocate_class(
    name: &str,
    definition: &crate::hir::ClassDefinition,
    clock: Clock,
) -> ClassValue {
    let payload = typed_slot(
        Type::Record {
            name: name.into(),
            copy: false,
            fields: definition.fields.clone(),
        },
        clock,
    );
    *payload.borrow_mut() = Some(Value::Aggregate(vec![Value::Unit; definition.fields.len()]));
    payload
        .state()
        .borrow_mut()
        .constructing(&[])
        .expect("new class storage");
    ClassValue { payload }
}

pub(crate) fn class_reference(value: ClassValue, clock: Clock) -> Reference {
    let Type::Record { name, .. } = value.payload.ty() else {
        unreachable!()
    };
    let slot = typed_slot(
        Type::Class {
            name: name.clone(),
            copy: false,
            storage: None,
        },
        clock,
    );
    let place = Reference {
        slot,
        projections: vec![],
    };
    place.write(Value::Class(value)).expect("new handle");
    place
}

pub(crate) fn complete_class(value: &Value) -> Result<(), TrapKind> {
    let Value::Class(value) = value else {
        return Err(TrapKind::InvalidIr);
    };
    value
        .payload
        .state()
        .borrow_mut()
        .initialize(&[])
        .map_err(|_| TrapKind::InvalidIr)
}

/// Explicit language Copy; ordinary host clones are identity-preserving reads.
pub(crate) fn copy_value(value: &Value, clock: Clock) -> Result<Value, TrapKind> {
    Ok(match value {
        Value::Class(object) => {
            let copied = copy_value(
                object
                    .payload
                    .borrow()
                    .as_ref()
                    .ok_or(TrapKind::InvalidIr)?,
                clock.clone(),
            )?;
            let payload = typed_slot(object.payload.ty().clone(), clock);
            payload.initialize()?;
            *payload.borrow_mut() = Some(copied);
            Value::Class(ClassValue { payload })
        }
        Value::Aggregate(fields) => Value::Aggregate(
            fields
                .iter()
                .map(|v| copy_value(v, clock.clone()))
                .collect::<Result<_, _>>()?,
        ),
        Value::Enum { variant, fields } => Value::Enum {
            variant: *variant,
            fields: fields
                .iter()
                .map(|v| copy_value(v, clock.clone()))
                .collect::<Result<_, _>>()?,
        },
        value => value.clone(),
    })
}
