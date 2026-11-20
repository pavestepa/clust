use super::*;

#[test]
fn approved_profile_enforces_constructors_privacy_and_rust_only_unsafe() {
    use clust_frontend::{
        ast::{ItemKind, Member},
        parser::parse_clust,
    };
    let parsed = parse_clust(SourceId(0), "pub class C {value:i32;new() {self.value=0;} pub read(self)->i32 {return self.value;} hidden(self) {}} fn main()->i32 {return 0;}");
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let module = parsed.module.unwrap();
    assert!(!module.items[0].private);
    assert!(module.items[1].private);
    let ItemKind::Aggregate { members, .. } = &module.items[0].kind else {
        panic!()
    };
    assert!(matches!(members[0], Member::Field { private: true, .. }));
    assert!(matches!(members[2], Member::Method { private: false, .. }));
    assert!(matches!(members[3], Member::Method { private: true, .. }));
    for source in [
        "class C {}",
        "class C {new() {} new(n:i32) {}}",
        "class C {new() {return;}}",
        "class C {new() {if true {return;}}}",
        "class C {new() {while true {return;}}}",
        "class C {new() {let value=if true {return;} else {0};}}",
        "class C {new() {} pub fn read(self) {}}",
        "class C {new() {} pub virtual read(self) {}}",
        "class C {constructor() {}}",
        "class C {new() {} read(this) {}}",
        "pub struct C {}",
        "import {Item} from \"./items\";",
        "export {Item} from \"./items\";",
        "use crate::items::Item;",
        "fn main() { let n = 1 }",
        "fn main() { println(1) }",
        "fn main()->i32 { return 1 }",
        "class C {new() {} change(var this) {}}",
        "#[Copy] public class C {new() {}}",
        "fn main():i32 {return 0;}",
        "fn main() {unsafe {}}",
        "unsafe fn main() {}",
        "class C {new() {}} fn main() {let c=new C();}",
    ] {
        let parsed = parse_clust(SourceId(0), source);
        assert!(!parsed.diagnostics.is_empty(), "accepted {source}");
    }
    for source in [
        include_str!("../../../../examples/clust_counter/main.cl"),
        "class C {pub new() {}}",
        "extern \"C\" {unsafe fn external(x:i32)->i32;} fn main()->i32 {return external(1);}",
        "fn main() {let text=\"unsafe { this constructor }\";}",
    ] {
        let parsed = parse_clust(SourceId(0), source);
        assert!(
            parsed.diagnostics.is_empty(),
            "{source}: {:?}",
            parsed.diagnostics
        );
    }
}

#[test]
fn tokenizes_the_documented_class_constructor_example() {
    let document = include_str!("../../../../docs/syntax.md");
    let example = document
        .split("```clust\n")
        .nth(1)
        .unwrap()
        .split("```")
        .next()
        .unwrap();
    let result = lex(SourceId(0), example);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    for word in [
        "Animal", "Cat", "new", "self", "super", "color", "u8", "u32",
    ] {
        assert!(result
            .tokens
            .iter()
            .any(|token| token.kind == TokenKind::Word(word.into())));
    }
    for token in &result.tokens {
        assert!(example.get(token.span.start..token.span.end).is_some());
    }
    assert_eq!(result.tokens.last().unwrap().kind, TokenKind::Eof);
}

#[test]
fn parses_documented_constructors_and_base_initialization() {
    let document = include_str!("../../../../docs/syntax.md");
    let example = document
        .split("```clust\n")
        .nth(1)
        .unwrap()
        .split("```")
        .next()
        .unwrap();
    let parsed = clust_frontend::parser::parse_clust(SourceId(0), example);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let module = parsed.module.unwrap();
    assert_eq!(
        module
            .items
            .iter()
            .filter(|item| matches!(
                item.kind,
                clust_frontend::ast::ItemKind::Aggregate { class: true, .. }
            ))
            .count(),
        2
    );
    assert!(module.items.iter().any(|item| matches!(&item.kind, clust_frontend::ast::ItemKind::Function(function) if function.name == "main")));
}

#[test]
fn parses_free_functions_and_methods_with_clust_receivers() {
    use clust_frontend::ast::{ItemKind, Member, TypeKind};
    let source = "class Counter { value:i32; new(value:i32) {self.value=value;} pub read(self)->i32 {return self.value;} pub add(var self,n:i32) {self.value+=n;} } fn main()->i32 {var c=Counter.new(2);c.add(3);return c.read();}";
    let parsed = clust_frontend::parser::parse(SourceId(0), source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let module = parsed.module.unwrap();
    let ItemKind::Aggregate { members, .. } = &module.items[0].kind else {
        panic!("expected class")
    };
    let Member::Method {
        function, private, ..
    } = &members[3]
    else {
        panic!("expected method")
    };
    assert!(!private);
    assert!(function.parameters[0].receiver && function.parameters[0].mutable);
    assert!(matches!(&function.result.kind,TypeKind::Tuple(items) if items.is_empty()));
}

#[test]
fn only_public_instance_methods_use_inferred_dispatch() {
    use clust_frontend::{
        ast::{ItemKind, Member, MethodDispatch},
        parser::parse,
    };
    let parsed = parse(
        SourceId(0),
        "class C {new() {} pub read(self) {} pub create() {} hidden(self) {}} ",
    );
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let module = parsed.module.unwrap();
    let ItemKind::Aggregate { members, .. } = &module.items[0].kind else {
        panic!()
    };
    for (member, (expected_private, expected_dispatch)) in members.iter().skip(1).zip([
        (false, MethodDispatch::Inferred),
        (false, MethodDispatch::Ordinary),
        (true, MethodDispatch::Ordinary),
    ]) {
        let Member::Method {
            private, dispatch, ..
        } = member
        else {
            panic!()
        };
        assert_eq!((*private, *dispatch), (expected_private, expected_dispatch));
    }
}
