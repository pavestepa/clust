use super::*;

#[test]
fn destructor_precedes_fields_and_resource_free_fields_keep_their_indices() {
    let leaf = record("Leaf", vec![Type::Bool]);
    let parent = record(
        "Parent",
        vec![Type::Bool, leaf.clone(), Type::Unit, leaf.clone()],
    );
    let mut registry = Destructors::default();
    registry.insert(&leaf, 7).unwrap();
    registry.insert(&parent, 9).unwrap();
    let leaf_plan = Plan::Aggregate {
        destructor: Some(7),
        fields: vec![],
    };
    assert_eq!(
        registry.plan(&parent),
        Plan::Aggregate {
            destructor: Some(9),
            fields: vec![(1, leaf_plan.clone()), (3, leaf_plan)]
        }
    );
    assert_eq!(registry.entry(&leaf), Some(7));
}

#[test]
fn enum_and_arrays_preserve_active_payload_and_element_order_without_expansion() {
    let resource = record("Resource", vec![]);
    let mut registry = Destructors::default();
    registry.insert(&resource, 4).unwrap();
    let item = registry.plan(&resource);
    let array = Type::Array(Box::new(resource.clone()), 1_000_000_000);
    assert_eq!(
        registry.plan(&array),
        Plan::Array {
            element: Box::new(item.clone()),
            count: 1_000_000_000
        }
    );
    assert_eq!(
        registry.plan(&Type::Array(Box::new(resource.clone()), 0)),
        Plan::None
    );
    let ty = Type::Enum {
        name: "E".into(),
        copy: false,
        variants: vec![
            Variant {
                name: "Empty".into(),
                shape: VariantShape::Unit,
                fields: vec![],
            },
            Variant {
                name: "Live".into(),
                shape: VariantShape::Tuple,
                fields: vec![Type::Bool, resource],
            },
        ],
    };
    assert_eq!(
        registry.plan(&ty),
        Plan::Enum {
            variants: vec![
                Plan::None,
                Plan::Aggregate {
                    destructor: None,
                    fields: vec![(1, item)]
                }
            ]
        }
    );
}

#[test]
fn invalid_destructor_registration_does_not_replace_existing_entries() {
    let mut registry = Destructors::default();
    let resource = record("Resource", vec![]);
    registry.insert(&resource, 1).unwrap();
    assert!(registry.insert(&resource, 2).is_err());
    assert_eq!(registry.entry(&resource), Some(1));
    assert!(registry.insert(&Type::Bool, 3).is_err());
    let Type::Record { name, fields, .. } = resource else {
        unreachable!()
    };
    assert!(registry
        .insert(
            &Type::Record {
                name,
                fields,
                copy: true
            },
            4
        )
        .is_err());
}

#[test]
fn first_panic_continues_fields_and_second_panic_stops_immediately() {
    let plan = Plan::Aggregate {
        destructor: Some(1),
        fields: vec![(0, leaf(2)), (1, leaf(3))],
    };
    let mut memory = Model {
        fail: vec![1],
        ..Model::default()
    };
    let mut pending = None;
    drop_plan::execute(&plan, &mut memory, &mut pending).unwrap();
    assert_eq!(memory.events, [1, 2, 3]);
    assert_eq!(pending, Some(1));
    drop_plan::execute(&plan, &mut memory, &mut pending).unwrap();
    assert_eq!(memory.events, [1, 2, 3]);
    let mut memory = Model {
        fail: vec![1, 2],
        ..Model::default()
    };
    assert_eq!(
        drop_plan::execute(&plan, &mut memory, &mut None),
        Err(Failure::DoublePanic {
            first: 1,
            second: 2
        })
    );
    assert_eq!(memory.events, [1, 2]);
    let mut memory = Model {
        fail: vec![2],
        ..Model::default()
    };
    assert_eq!(
        drop_plan::execute(&plan, &mut memory, &mut Some(99)),
        Err(Failure::DoublePanic {
            first: 99,
            second: 2
        })
    );
    assert_eq!(memory.events, [1, 2]);
}
