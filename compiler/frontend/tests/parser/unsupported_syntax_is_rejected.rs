use super::*;

#[test]
fn unsupported_syntax_is_rejected_at_source() {
    for (text, code) in [
        ("async pub fn f() -> () {}", "E0100"),
        ("pub fn f(...args: i32) -> () {}", "E0101"),
        ("pub fn f() -> () { let x = [1,2] }", "E0105"),
        ("pub fn f() -> () { operation()? }", "E0107"),
        ("pub fn f() -> () { if let x = value {} }", "E0107"),
    ] {
        assert_eq!(failure(text), code);
    }
    failure("pub fn f(self) -> () {}");
    failure("class S { weak next: S }");
    failure("pub fn f<T = i32, U>() -> () {}");
    failure("pub fn f<const N: i32>() -> () {}");
}

#[test]
fn incomplete_and_deep_inputs_report_errors_without_panicking() {
    let sample = "pub fn test<T>(a: {T; 2}) -> T { return if true { a[0] } else { a[1] };}";
    for end in 0..sample.len() {
        let _ = parse(SourceId(0), &sample[..end]);
    }
    let nested = format!(
        "pub fn f() -> i32 {{ return {}1{};}}",
        "(".repeat(300),
        ")".repeat(300)
    );
    assert_eq!(failure(&nested), "E0201");
    let chain = format!("pub fn f() -> i32 {{ return 1{} }}", "+1".repeat(600));
    assert_eq!(failure(&chain), "E0201");
    for token in [
        "{", "}", "(", ")", "fn", "=>", "let", "this", "...", "<", ">>", "\n",
    ] {
        let _ = parse(SourceId(0), &token.repeat(100));
    }
}

#[test]
fn type_closers_split_adjacent_assignment_tokens() {
    module("pub fn f() -> () {let a:Option<i32>=get();let b:Outer<Inner<i32>>=get();} ");
    module("pub fn f(a:i32,b:i32,c:i32) -> bool {return a < b >> c;}");
}

#[test]
fn public_methods_use_inferred_dispatch() {
    let parsed = module("class A {new() {} pub read(self)->i32 {return 1;}} class B:A {new() {super.new();} pub read(self)->i32 {return 2;}}");
    for item in parsed.items {
        let ItemKind::Aggregate { members, .. } = item.kind else {
            panic!()
        };
        assert!(matches!(
            members[1],
            Member::Method {
                dispatch: MethodDispatch::Inferred,
                ..
            }
        ));
    }
    failure("class A {new() {} virtual read(self) {}}");
    failure("class A {new() {} override read(self) {}}");
}

#[test]
fn unsafe_callable_annotation_and_loop_parse_without_erasing_safety() {
    let parsed = clust_frontend::parser::parse(
        clust_frontend::source::SourceId(0),
        "pub fn invoke(callback:unsafe (i32)=>i32) -> i32 {loop {return callback(7);}}",
    );
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let module = parsed.module.unwrap();
    let clust_frontend::ast::ItemKind::Function(f) = &module.items[0].kind else {
        panic!()
    };
    assert!(matches!(
        f.parameters[0].ty.as_ref().unwrap().kind,
        clust_frontend::ast::TypeKind::Function { unsafe_: true, .. }
    ));
    let parsed = clust_frontend::parser::parse(
        clust_frontend::source::SourceId(0),
        "pub fn f(x:unsafe i32) -> () {}",
    );
    assert!(!parsed.diagnostics.is_empty());
}

#[test]
fn dynamic_types_and_collection_iteration_are_supported_syntax() {
    module("pub fn f(x: dyn Thing) -> () {}");
    let body =
        body("pub fn f() -> () { for x in values { if x == 1 {continue;} if x == 2 {break;} } }");
    assert!(matches!(
        &body.statements[0].kind,
        StatementKind::Expression(Expr {
            kind: ExprKind::Branch(_),
            ..
        })
    ));
}
