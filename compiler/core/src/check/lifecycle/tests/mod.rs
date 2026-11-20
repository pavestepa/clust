use super::*;
use clust_frontend::{source::SourceId, testing::parse};
fn analyzed(source: &str) -> std::result::Result<h::Program, Vec<Diagnostic>> {
    let parsed = parse(SourceId(0), source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    super::super::check(&parsed.module.unwrap())
}
#[test]
fn typed_destructors_allow_fields_but_reject_transitive_whole_replacement() {
    for body in [
        "value=2;",
        "let ref var alias=self;alias.value=3;",
        "self.update();",
    ] {
        let source = "fn main()->i32 {let x=Resource {value:1};return x.value;}";
        let record = clust_frontend::testing::record::inline(
            "Resource",
            &[("value", "i32")],
            &["update(var self) {self.value+=1;}"],
            &[body],
            Span {
                source: SourceId(0),
                start: 0,
                end: 0,
            },
        );
        let parsed = clust_frontend::testing::parse_with_items(SourceId(0), source, vec![record]);
        let p = super::super::check(&parsed.module.unwrap())
            .unwrap_or_else(|e| panic!("{body}: {e:?}"));
        let registry = validate(&p).unwrap();
        let destructor = p.functions.iter().find(|f| is_destructor(&f.name)).unwrap();
        assert!(registry
            .entry(&destructor.locals[destructor.parameters[0]].ty)
            .is_some());
    }
    for body in [
        "*self=Resource {value:2};",
        "let ref var alias=self;*alias=Resource {value:2};",
        "self.replace();",
    ] {
        let record = clust_frontend::testing::record::inline(
            "Resource",
            &[("value", "i32")],
            &["replace(var self) {*self=Resource {value:2};}"],
            &[body],
            Span {
                source: SourceId(0),
                start: 0,
                end: 0,
            },
        );
        let parsed = clust_frontend::testing::parse_with_items(SourceId(0), "", vec![record]);
        assert!(
            super::super::check(&parsed.module.unwrap())
                .unwrap_err()
                .iter()
                .any(|e| e.code == "E0391"),
            "{body}"
        );
    }
}
#[test]
fn copy_duplicate_and_partial_move_are_rejected() {
    for source in [
        "use compiler_fixtures.record_137.{R};",
        "use compiler_fixtures.record_138.{R};",
    ] {
        assert!(analyzed(source)
            .unwrap_err()
            .iter()
            .any(|e| e.code == "E0390"));
    }
    let source="use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_139.{R}; use compiler_fixtures.record_140.{H}; pub fn main() -> i32 {let r=R {cell: Cell {value: 1}};let h=H {c: r.cell};return h.c.value;}";
    assert!(analyzed(source)
        .unwrap_err()
        .iter()
        .any(|e| e.code == "E0392"));
}

mod cleanup;
mod native;
