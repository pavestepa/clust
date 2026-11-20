use super::*;

/// Required value reads, as distinct from merely forming an assignment address.
pub fn required_initialized(terminator: &Terminator) -> Vec<Place> {
    let mut places = Vec::new();
    let mut add_operand = |operand: &Operand| match operand {
        Operand::Local(local) => places.push(Place::local(*local)),
        Operand::Address(place) => places.push(place.clone()),
        _ => (),
    };
    match terminator {
        Terminator::Evaluate { operation, .. } => match operation {
            Operation::Identity(a, b) => {
                places.push(a.clone());
                places.push(b.clone());
            }
            Operation::TagPlace(place)
            | Operation::Borrow(place)
            | Operation::CapturePlace(place)
            | Operation::LoadView(place)
            | Operation::StoreView(place)
            | Operation::Copy(place)
            | Operation::Move(place) => places.push(place.clone()),
            Operation::AllocateClass(_) => {}
            Operation::CompleteClass(value)
            | Operation::Use(value)
            | Operation::Unary(_, value)
            | Operation::Cast(value, _)
            | Operation::Tag(value) => add_operand(value),
            Operation::Binary(_, left, right) => {
                add_operand(left);
                add_operand(right);
            }
            Operation::CallIndirect(callee, values, _) => {
                add_operand(callee);
                for value in values {
                    add_operand(value);
                }
            }
            Operation::StorageView {
                arguments: values, ..
            }
            | Operation::Aggregate(values)
            | Operation::Enum(_, values)
            | Operation::Call(_, values)
            | Operation::CallStorage {
                arguments: values, ..
            }
            | Operation::CallView {
                arguments: values, ..
            } => {
                for value in values {
                    add_operand(value);
                }
            }
        },
        Terminator::Branch { condition, .. } | Terminator::Return(condition) => {
            add_operand(condition)
        }
        _ => (),
    }
    let address = match terminator {
        Terminator::Evaluate { destination, .. } => Some(destination),
        Terminator::CheckPlace { place, .. } => Some(place),
        _ => None,
    };
    let mut indices = Vec::new();
    for place in places.iter().chain(address) {
        for projection in &place.projections {
            if let Projection::Index(local) = projection {
                indices.push(Place::local(*local));
            }
        }
    }
    places.extend(indices);
    for place in &mut places {
        if matches!(place.projections.first(), Some(Projection::Dereference)) {
            place.projections.clear();
        }
    }
    places
}

pub fn place_initialization_errors(function: &Function) -> Vec<(BlockId, Place)> {
    let flow = reachability::analyze(function);
    use super::super::initialization::{State, Status};
    let mut universe: std::collections::BTreeSet<Place> =
        (0..function.locals.len()).map(Place::local).collect();
    for block in &function.blocks {
        universe.extend(required_initialized(&block.terminator));
        if let Terminator::Evaluate { destination, .. } = &block.terminator {
            universe.insert(destination.clone());
        }
    }
    for place in universe.clone() {
        for length in 0..place.projections.len() {
            universe.insert(Place {
                local: place.local,
                projections: place.projections[..length].to_vec(),
            });
        }
    }
    let mut initial = State::default();
    for local in &function.parameters {
        initial.initialize(Place::local(*local));
    }
    let mut incoming: Vec<Option<State>> = vec![None; function.blocks.len()];
    incoming[function.entry] = Some(initial);
    let mut queue = std::collections::VecDeque::from([function.entry]);
    while let Some(id) = queue.pop_front() {
        let state = incoming[id].as_ref().unwrap().clone();
        for next in &flow.successors[id] {
            let next = *next;
            let mut output = state.clone();
            if let Terminator::EndStorage { locals, .. } = &function.blocks[id].terminator {
                for local in locals {
                    output.set(Place::local(*local), Status::Uninitialized);
                }
            }
            if let Terminator::Evaluate {
                operation,
                destination,
                next: success,
                ..
            } = &function.blocks[id].terminator
            {
                if *success == next
                    || matches!(
                        operation,
                        Operation::Call(..)
                            | Operation::CallIndirect(..)
                            | Operation::CallView { .. }
                            | Operation::CallStorage { .. }
                    )
                {
                    for source in consumed(function, operation) {
                        output.set(source, Status::Uninitialized);
                    }
                }
                if *success == next {
                    output.initialize(destination.clone());
                }
            }
            let updated = match &incoming[next] {
                Some(old) => old.join(&output, universe.iter().cloned()),
                None => output,
            };
            let changed = incoming[next].as_ref().is_none_or(|old| {
                universe
                    .iter()
                    .any(|place| old.status(place) != updated.status(place))
            });
            if changed {
                incoming[next] = Some(updated);
                queue.push_back(next);
            }
        }
    }
    let mut errors = Vec::new();
    for (id, state) in incoming.iter().enumerate() {
        if let Some(state) = state {
            for place in required_initialized(&function.blocks[id].terminator) {
                if state.status(&place) != Status::Initialized {
                    errors.push((id, place));
                }
            }
        }
    }
    errors
}

pub fn initialization_errors(function: &Function) -> Vec<(BlockId, LocalId)> {
    place_initialization_errors(function)
        .into_iter()
        .map(|(block, place)| (block, place.local))
        .collect()
}

/// Value operands transfer non-Copy ownership into the receiving operation.
pub fn consumed(function: &Function, operation: &Operation) -> Vec<Place> {
    let mut places = Vec::new();
    let mut operand = |value: &Operand| {
        if let Operand::Local(local) = value {
            if function.locals[*local].view.is_none() && function.locals[*local].ty.owns_storage() {
                places.push(Place::local(*local));
            }
        }
    };
    match operation {
        Operation::Move(source) => places.push(source.clone()),
        Operation::Use(value)
        | Operation::CompleteClass(value)
        | Operation::Unary(
            "raw_initialize" | "storage_initialize" | "storage_replace" | "storage_detach",
            value,
        ) => operand(value),
        Operation::Aggregate(values)
        | Operation::Enum(_, values)
        | Operation::Call(_, values)
        | Operation::CallIndirect(_, values, _)
        | Operation::CallStorage {
            arguments: values, ..
        }
        | Operation::CallView {
            arguments: values, ..
        } => values.iter().for_each(operand),
        _ => (),
    }
    places
}
