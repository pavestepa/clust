use super::*;

#[test]
fn unsafe_declarations_are_limited_to_imports() {
    let parsed =
        module(r#"#[rust("i32.unchecked_add")] extern "Rust" unsafe fn add(a:i32,b:i32)->i32;"#);
    let ItemKind::Function(function) = &parsed.items[0].kind else {
        panic!()
    };
    assert!(function.unsafe_);
    failure("unsafe fn local() {}");
    failure("class S {new() {} unsafe local(self) {}}");
    failure("interface I {unsafe local(self);}");
}

#[test]
fn newlines_return_array_and_block_contexts_are_distinct() {
    for newline in ["\n", "\r\n", "\r"] {
        let text =
            "pub fn test() -> () {\nlet empty: {i32; 0} = {};\nif true {}\nreturn;\n(10);\n}"
                .replace('\n', newline);
        let body = body(&text);
        assert_eq!(body.statements.len(), 4);
        assert!(
            matches!(&body.statements[0].kind, StatementKind::Binding { value: Expr {kind: ExprKind::Array(v),..},..} if v.is_empty())
        );
        assert!(matches!(
            body.statements[1].kind,
            StatementKind::Expression(Expr {
                kind: ExprKind::If { .. },
                ..
            })
        ));
        assert!(matches!(
            body.statements[2].kind,
            StatementKind::Return(None)
        ));
        assert!(matches!(
            body.statements[3].kind,
            StatementKind::Expression(Expr {
                kind: ExprKind::Group(_),
                ..
            })
        ));
    }
}

#[test]
fn continuations_do_not_turn_new_statements_into_postfix_operations() {
    let body =
        body("pub fn f() -> () {\nlet total = 1 +\n2;\nservice\n.load(\n1,\n2,\n);\n(next);\n}");
    assert_eq!(body.statements.len(), 3);
    assert!(matches!(
        body.statements[1].kind,
        StatementKind::Expression(Expr {
            kind: ExprKind::Call { .. },
            ..
        })
    ));
    assert!(matches!(
        body.statements[2].kind,
        StatementKind::Expression(Expr {
            kind: ExprKind::Group(_),
            ..
        })
    ));
    failure("fn f() { let a=1 let b=2; }");
    failure("pub fn f() -> () { call(1\n2) }");
}

#[test]
fn operators_have_correct_precedence_and_assignment_associativity() {
    let body = body("pub fn f() -> bool { a = b = 1 + 2 * 3; return a == 7 && b == 7;}");
    let StatementKind::Expression(Expr {
        kind: ExprKind::Assign { value, .. },
        ..
    }) = &body.statements[0].kind
    else {
        panic!()
    };
    let ExprKind::Assign { value, .. } = &value.kind else {
        panic!()
    };
    let ExprKind::Binary {
        operator: "+",
        right,
        ..
    } = &value.kind
    else {
        panic!()
    };
    assert!(matches!(right.kind, ExprKind::Binary { operator: "*", .. }));
    failure("pub fn f() -> bool { return 1 < 2 < 3 }");
    module("pub fn f() -> bool { return (1 < 2) == true;}");
}

#[test]
fn full_declaration_surface_and_nested_generic_closers() {
    let input = r#"
use crate.users.{User, read};
pub use crate.users.{User};
#[Copy]
#[repr(C)] pub class Point<T: Copy = i32> {new(x: T) { self.x = x; } 
    pub x: T
    pub get(self) -> T { return self.x;}
    pub set(var self, value: T) -> () { self.x = value;}
}
pub interface Read<T> {  read(self) -> T }
pub class Box<T> : Read<T> {
     value: T
    new(value: T) { self.value = value;}
    drop() { cleanup(self.value);}
    pub read(self) -> T { return value;}
    pub create(value: T) -> Box<T> { return Box<T>.new(value);}
}
pub enum Result<T, E> { Ok(T), Err { error: E }, Empty, }
const LENGTH: usize = 4;
pub fn nested<const N: usize = 4>(value: Box<Box<i32>>) -> {i32; N} {
    let pointer: *var i32 = raw();
    let location = pointer + 1;
    return {1,2,3,4};
}
#[link("c")]
extern "C" { fn puts(data: *u8) -> i32 }
extern "C" fn callback(value: i32) -> i32;
"#;
    assert_eq!(module(input).items.len(), 10);
}

#[test]
fn generic_calls_and_comparisons_are_disambiguated_without_losing_shifts() {
    let body = body("pub fn f() -> () { let x = make<Box<i32>>();\nlet y = a < b >> 1;\nlet z = Vec<i32>.create();\nlet p = Pair<i32>.new(1, 2);}");
    assert_eq!(body.statements.len(), 4);
    let StatementKind::Binding { value, .. } = &body.statements[1].kind else {
        panic!()
    };
    let ExprKind::Binary {
        operator: "<",
        right,
        ..
    } = &value.kind
    else {
        panic!()
    };
    assert!(matches!(
        right.kind,
        ExprKind::Binary { operator: ">>", .. }
    ));
}

#[test]
fn patterns_guards_and_branch_results_preserve_binding_modes() {
    let body = body("pub fn f(value: Result<i32,i32>) -> i32 { return match value { Result.Ok(ref var x) if valid(x) => { x; }, Result.Err {error: ref e} => e, Result.Empty => 0, };}");
    let StatementKind::Return(Some(Expr {
        kind: ExprKind::Match { arms, .. },
        ..
    })) = &body.statements[0].kind
    else {
        panic!()
    };
    assert_eq!(arms.len(), 3);
    assert!(arms[0].guard.is_some());
    let PatternKind::Variant {
        fields: PatternFields::Tuple(fields),
        ..
    } = &arms[0].pattern.kind
    else {
        panic!()
    };
    assert!(matches!(
        fields[0].kind,
        PatternKind::Binding {
            by_ref: true,
            mutable: true,
            ..
        }
    ));
    module("pub fn f() -> () { let (var x,y) = (1,2); let {a,..} = {1,2,3};}");
    failure("pub fn f() -> () { let {..,..} = {1,2,3} }");
}
