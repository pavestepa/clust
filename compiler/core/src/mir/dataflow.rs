//! Backward liveness with distinct successful and panic edges.
use super::*;
use std::collections::BTreeSet;
pub type Locals = BTreeSet<LocalId>;
#[derive(Debug)]
pub struct Liveness {
    pub live_in: Vec<Locals>,
    pub live_out: Vec<Locals>,
}
pub fn liveness(function: &Function) -> Liveness {
    let flow = reachability::analyze(function);
    let mut input = vec![Locals::new(); function.blocks.len()];
    let mut output = input.clone();
    loop {
        let mut changed = false;
        for (id, block) in function.blocks.iter().enumerate().rev() {
            if !flow.reachable[id] {
                continue;
            }
            let mut live = Locals::new();
            for next in &flow.successors[id] {
                live.extend(&input[*next]);
            }
            output[id] = live.clone();
            if let Terminator::Evaluate {
                destination,
                next,
                unwind,
                ..
            } = &block.terminator
            {
                live = input[*next].clone();
                if destination.projections.is_empty() {
                    live.remove(&destination.local);
                }
                live.extend(&input[*unwind]);
            }
            if let Terminator::EndStorage { locals, .. } = &block.terminator {
                for local in locals {
                    live.remove(local);
                }
            }
            live.extend(uses(&block.terminator));
            match &block.terminator {
                Terminator::EndStorage { locals, .. } => {
                    live.extend(locals.iter().filter(|l| function.drop_reads.contains(l)))
                }
                Terminator::Evaluate { destination, .. }
                    if destination.projections.is_empty()
                        && function.drop_reads.contains(&destination.local) =>
                {
                    live.insert(destination.local);
                }
                _ => {}
            }
            if input[id] != live {
                input[id] = live;
                changed = true;
            }
        }
        if !changed {
            return Liveness {
                live_in: input,
                live_out: output,
            };
        }
    }
}
pub fn uses(terminator: &Terminator) -> Locals {
    let mut result = Locals::new();
    match terminator {
        Terminator::Evaluate {
            operation,
            destination,
            ..
        } => {
            if !destination.projections.is_empty() {
                place_uses(destination, &mut result);
            }
            match operation {
                Operation::Identity(a, b) => {
                    place_uses(a, &mut result);
                    place_uses(b, &mut result);
                }
                Operation::TagPlace(place)
                | Operation::Borrow(place)
                | Operation::CapturePlace(place)
                | Operation::LoadView(place)
                | Operation::StoreView(place)
                | Operation::Copy(place)
                | Operation::Move(place) => place_uses(place, &mut result),
                Operation::AllocateClass(_) => {}
                Operation::CompleteClass(value)
                | Operation::Use(value)
                | Operation::Unary(_, value)
                | Operation::Cast(value, _)
                | Operation::Tag(value) => operand_uses(value, &mut result),
                Operation::Binary(_, a, b) => {
                    operand_uses(a, &mut result);
                    operand_uses(b, &mut result);
                }
                Operation::CallIndirect(callee, values, _) => {
                    operand_uses(callee, &mut result);
                    for value in values {
                        operand_uses(value, &mut result);
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
                        operand_uses(value, &mut result);
                    }
                }
            }
        }
        Terminator::CheckPlace { place, .. } => place_uses(place, &mut result),
        Terminator::Branch { condition, .. } | Terminator::Return(condition) => {
            operand_uses(condition, &mut result)
        }
        _ => (),
    }
    result
}
fn operand_uses(value: &Operand, output: &mut Locals) {
    match value {
        Operand::Local(local) => {
            output.insert(*local);
        }
        Operand::Address(place) => place_uses(place, output),
        _ => (),
    }
}
fn place_uses(place: &Place, output: &mut Locals) {
    output.insert(place.local);
    for projection in &place.projections {
        if let Projection::Index(local) = projection {
            output.insert(*local);
        }
    }
}

mod initialization;
pub use self::initialization::consumed;
pub use self::initialization::initialization_errors;
pub use self::initialization::place_initialization_errors;
pub use self::initialization::required_initialized;
