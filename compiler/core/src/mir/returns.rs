//! Static result provenance. Facts flow through value temporaries before final
//! ownership validation, allowing a borrowed entry to return its input view.
use super::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Origin {
    pub parameter: usize,
    pub path: Vec<Projection>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract {
    pub access: loans::Access,
    pub origins: BTreeSet<Origin>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Component {
    pub path: Vec<Projection>,
    pub origins: BTreeSet<Origin>,
}
pub type StorageContract = Vec<Component>;

pub(crate) fn call_view(op: &Operation) -> Option<(&[Operand], &Contract)> {
    match op {
        Operation::StorageView {
            arguments,
            contract,
        } => Some((arguments, contract)),
        Operation::CallView {
            arguments,
            contract,
            ..
        } => Some((arguments, contract)),
        Operation::CallIndirect(
            _,
            args,
            h::CallContract {
                view: Some(view), ..
            },
        ) => Some((args, view)),
        _ => None,
    }
}
pub(super) fn call_storage(op: &Operation) -> Option<(&[Operand], &StorageContract)> {
    match op {
        Operation::CallStorage {
            arguments,
            contract,
            ..
        } => Some((arguments, contract)),
        Operation::CallIndirect(_, args, h::CallContract { storage, .. })
            if !storage.is_empty() =>
        {
            Some((args, storage))
        }
        _ => None,
    }
}
/// Dependencies of descriptors carried inside an owned aggregate result.
/// The wrapper owns its storage but never the targets reached through these paths.
pub fn infer_storage(
    program: &Program,
    views: &[Option<Contract>],
) -> Result<Vec<StorageContract>, Error> {
    program.functions.iter().enumerate().map(|(id, f)| {
        if !f.result.contains_views() || views[id].is_some() {
            return Ok(Vec::new());
        }
        let provenance = borrow_check::provenance(f);
        let mut components = BTreeMap::<Vec<Projection>, BTreeSet<Origin>>::new();
        for (block, body) in f.blocks.iter().enumerate() {
            let Terminator::Return(Operand::Local(local)) = &body.terminator else { continue };
            for (path, origins) in provenance.components(block, &Place::local(*local)) {
                for mut origin in origins {
                    if let Some(index) = origin.projections.iter().position(|p| matches!(p, Projection::Index(_))) {
                        origin.projections.truncate(index);
                    }
                    if origin.projections.len() > 32 {
                        return Err(Error {origin:None,function:id,message:"aggregate result provenance exceeds the supported path limit"});
                    }
                    let parameter = f.parameters.iter().position(|p| *p == origin.local);
                    let Some(parameter) = parameter.filter(|_| f.locals[origin.local].view.is_some()
                        || origin.projections.contains(&Projection::Dereference)) else {
                        return Err(Error {origin:f.locals.get(origin.local).map(|l| l.span),function:id,
                            message:"returned aggregate contains a view of an owner destroyed by this function"});
                    };
                    for path in concrete_paths(&f.result, &path, &f.classes) {
                        components.entry(path).or_default().insert(Origin {parameter,path:origin.projections.clone()});
                    }
                }
            }
        }
        Ok(components.into_iter().map(|(path, origins)| Component {path,origins}).collect())
    }).collect()
}
// Result storage paths must not contain callee-local dynamic index IDs.
// Expand a dynamic write to every potentially affected fixed-array component.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Fact {
    owned: bool,
    readonly: bool,
    origins: BTreeSet<Place>,
}
#[derive(Clone, Debug)]
pub struct Error {
    pub origin: Option<Span>,
    pub function: usize,
    pub message: &'static str,
}

mod paths;
mod view_inference;
pub(super) use self::paths::concrete_paths;
pub use self::view_inference::infer;
