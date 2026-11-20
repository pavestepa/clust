use clust_core::{
    hir::{RecordField, Type, Variant, VariantShape},
    mir::drop_plan::{Destructors, Plan},
};
use clust_frontend::source::SourceId;
fn record(name: &str, fields: Vec<Type>) -> Type {
    Type::Record {
        copy: false,
        name: name.into(),
        fields: fields
            .into_iter()
            .enumerate()
            .map(|(i, ty)| RecordField {
                name: format!("f{i}"),
                ty,
                private: false,
                source: SourceId(0),
            })
            .collect(),
    }
}

use clust_core::mir::drop_plan::{self, Failure, Initialization, Memory, Part};
use std::collections::BTreeMap;
#[derive(Default)]
struct Model {
    states: BTreeMap<Vec<Part>, Initialization>,
    events: Vec<usize>,
    fail: Vec<usize>,
    tag: usize,
}
impl Memory for Model {
    type Panic = usize;
    fn initialization(&self, path: &[Part]) -> Initialization {
        self.states
            .get(path)
            .copied()
            .unwrap_or(Initialization::Live)
    }
    fn begin(&mut self, path: &[Part]) {
        self.states.insert(path.into(), Initialization::Finalizing);
    }
    fn finish(&mut self, path: &[Part]) {
        self.states.insert(path.into(), Initialization::Vacant);
    }
    fn variant(&self, _: &[Part]) -> Option<usize> {
        (self.tag != usize::MAX).then_some(self.tag)
    }
    fn destructor(&mut self, entry: usize, path: &[Part]) -> Result<(), usize> {
        assert_eq!(self.initialization(path), Initialization::Finalizing);
        self.events.push(entry);
        if self.fail.contains(&entry) {
            Err(entry)
        } else {
            Ok(())
        }
    }
}
fn leaf(entry: usize) -> Plan {
    Plan::Aggregate {
        destructor: Some(entry),
        fields: vec![],
    }
}

#[path = "drop_plan/destructor_precedes_fields_and.rs"]
mod destructor_precedes_fields_and;
#[path = "drop_plan/partial_aggregates_skip_missing.rs"]
mod partial_aggregates_skip_missing;
