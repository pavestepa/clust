use super::*;

#[test]
fn drop_has_an_implicit_receiver_and_no_parameters_or_result_annotation() {
    use clust_frontend::{
        ast::{ItemKind, Member},
        parser::parse,
    };
    let parsed = parse(
        SourceId(0),
        "class C {new() {} drop() {self.value=0;} value:i32}",
    );
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let module = parsed.module.unwrap();
    let ItemKind::Aggregate { members, .. } = &module.items[0].kind else {
        panic!()
    };
    assert!(matches!(&members[1], Member::Destructor(_)));
    for source in [
        "class C {new() {} drop(self) {}}",
        "class C {new() {} drop(n:i32) {}}",
        "class C {new() {} drop()->i32 {return 1}}",
    ] {
        assert!(
            !parse(SourceId(0), source).diagnostics.is_empty(),
            "accepted {source}"
        );
    }
}

#[test]
fn generated_diagnostics_resolve_to_original_source() {
    use clust_frontend::source::{Diagnostic, Source, SourceId, Span};
    let original = Span {
        source: SourceId(0),
        start: 4,
        end: 8,
    };
    let generated = original.synthetic(23).synthetic(12);
    assert_ne!(original, generated);
    assert_eq!(generated.source_origin().start, original.start);
    let diagnostic = Diagnostic::new("E0000", "generated error", generated)
        .with_note(original.synthetic(7), "origin");
    assert_eq!(
        diagnostic.render(&[Source::new("main.cl", "one\ntwo\n")]),
        "main.cl:2:1: error[E0000]: generated error\nmain.cl:2:1: note: origin"
    );
}

#[test]
fn backend_fixture_support_preserves_source_class_semantics() {
    let source="#[repr(C)] class C {pub value:i32;new(value:i32) {self.value=value;} pub read(self)->i32 {return self.value;}} fn main()->i32 {return C.new(7).read();}";
    let ordinary = clust_frontend::parser::parse(SourceId(0), source);
    let backend = clust_frontend::testing::parse(SourceId(0), source);
    assert!(ordinary.diagnostics.is_empty());
    assert_eq!(ordinary.module, backend.module);
    let clust_frontend::ast::ItemKind::Aggregate { class, members, .. } =
        &backend.module.unwrap().items[0].kind
    else {
        panic!()
    };
    assert!(*class);
    assert!(members
        .iter()
        .any(|member| matches!(member, clust_frontend::ast::Member::Constructor { .. })));
}

#[test]
fn source_fixture_parsing_does_not_infer_unsafe_operations_from_names() {
    let source="fn requires_unsafe_caller() {} fn trusted_region()->bool {return true;} fn main()->i32 {requires_unsafe_caller();if trusted_region() {return 7;} return 0;}";
    let ordinary = clust_frontend::parser::parse(SourceId(0), source);
    let fixture = clust_frontend::testing::parse(SourceId(0), source);
    assert!(ordinary.diagnostics.is_empty());
    assert_eq!(ordinary.module, fixture.module);
}

#[test]
fn excessive_module_and_use_tree_nesting_reports_a_diagnostic() {
    for source in [
        format!("{}fn f() {{}}{}", "mod a {".repeat(100), "}".repeat(100)),
        format!("use {}item{};", "{".repeat(100), "}".repeat(100)),
    ] {
        let parsed = clust_frontend::parser::parse(SourceId(0), &source);
        assert!(
            parsed.diagnostics.iter().any(|error| error.code == "E0201"),
            "{:?}",
            parsed.diagnostics
        );
    }
}
