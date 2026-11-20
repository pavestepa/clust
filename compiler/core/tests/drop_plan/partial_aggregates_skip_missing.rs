use super::*;

#[test]
fn partial_aggregates_skip_missing_parts_and_enum_only_drops_active_payload() {
    let plan = Plan::Aggregate {
        destructor: None,
        fields: vec![
            (0, leaf(1)),
            (
                1,
                Plan::Array {
                    element: Box::new(leaf(2)),
                    count: 3,
                },
            ),
        ],
    };
    let mut memory = Model::default();
    memory.states.insert(vec![], Initialization::Partial);
    memory
        .states
        .insert(vec![Part::Field(0)], Initialization::Vacant);
    memory.states.insert(
        vec![Part::Field(1), Part::Element(1)],
        Initialization::Vacant,
    );
    drop_plan::execute(&plan, &mut memory, &mut None).unwrap();
    assert_eq!(memory.events, [2, 2]);
    let plan = Plan::Enum {
        variants: vec![leaf(4), leaf(5)],
    };
    let mut memory = Model {
        tag: 1,
        ..Model::default()
    };
    drop_plan::execute(&plan, &mut memory, &mut None).unwrap();
    assert_eq!(memory.events, [5]);
    let mut memory = Model {
        tag: usize::MAX,
        ..Model::default()
    };
    memory.states.insert(vec![], Initialization::Vacant);
    drop_plan::execute(&plan, &mut memory, &mut None).unwrap();
    assert!(memory.events.is_empty());
}

#[test]
fn partially_moved_user_destructor_and_reentrant_drop_are_invalid() {
    for state in [Initialization::Partial, Initialization::Finalizing] {
        let mut memory = Model::default();
        memory.states.insert(vec![], state);
        assert_eq!(
            drop_plan::execute(&leaf(1), &mut memory, &mut None),
            Err(Failure::InvalidLifecycle)
        );
        assert!(memory.events.is_empty());
    }
}

#[test]
fn unfinished_construction_cleans_fields_without_calling_parent_destructor() {
    let plan = Plan::Aggregate {
        destructor: Some(9),
        fields: vec![(0, leaf(1)), (1, leaf(2))],
    };
    let mut memory = Model::default();
    memory.states.insert(vec![], Initialization::Constructing);
    memory
        .states
        .insert(vec![Part::Field(1)], Initialization::Vacant);
    drop_plan::execute(&plan, &mut memory, &mut Some(55)).unwrap();
    assert_eq!(memory.events, [1]);
    let mut memory = Model {
        tag: usize::MAX,
        ..Model::default()
    };
    memory.states.insert(vec![], Initialization::Constructing);
    drop_plan::execute(
        &Plan::Enum {
            variants: vec![leaf(1)],
        },
        &mut memory,
        &mut Some(55),
    )
    .unwrap();
    assert!(memory.events.is_empty());
}
