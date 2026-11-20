use clust_core::{
    check::check,
    mir::{self, effects},
};
use clust_frontend::{
    ast::Module,
    source::{SourceId, Span},
    testing::{parse_with_items, record::inline},
};
fn cell(methods: &[&str], drops: &[&str], main: bool) -> Module {
    let source = if main {
        "fn main()->i32 {var x=Cell {value:1};x.change();return x.value;}"
    } else {
        ""
    };
    let record = inline(
        "Cell",
        &[("value", "i32")],
        methods,
        drops,
        Span {
            source: SourceId(0),
            start: 0,
            end: 0,
        },
    );
    let parsed = parse_with_items(SourceId(0), source, vec![record]);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    parsed.module.unwrap()
}
fn effect(module: &Module, name: &str) -> effects::Summary {
    let program = check(module).unwrap();
    let mir = mir::lower(&program);
    let summaries = effects::infer(&mir);
    let id = mir
        .functions
        .iter()
        .position(|f| {
            f.name.starts_with(name)
                && f.parameters
                    .first()
                    .is_some_and(|p| f.locals[*p].view.is_some())
        })
        .unwrap();
    summaries[id].clone()
}
#[test]
fn finalizing_target_distinguishes_fields_aliases_and_rebinding() {
    for (body, replace, fields) in [
        ("self.value=2;", false, true),
        ("*self=Cell {value:2};", true, false),
        ("let ref var alias=self;*alias=Cell {value:2};", true, false),
        ("let ref var alias=self;alias.value=2;", false, true),
        (
            "var other=Cell {value:3};let ref var alias=self;alias=other;alias.value=2;",
            false,
            false,
        ),
        (
            "let ref var alias=self;let ref var nested=alias;*nested=Cell {value:2};",
            true,
            false,
        ),
    ] {
        let method = format!("change(var self) {{ {body} }}");
        let summary = effect(&cell(&[&method], &[], true), "Cell.change");
        assert_eq!(
            summary.parameters[0],
            effects::TargetEffects {
                replace,
                write_fields: fields
            },
            "{body}"
        );
        assert_eq!(effects::permits_finalizing(&summary, 0), !replace);
    }
}
#[test]
fn effects_propagate_through_mutually_recursive_calls() {
    for (write, replace) in [("*self=Cell {value:2};", true), ("self.value=2;", false)] {
        let second =
            format!("second(var self,n:i32) {{if n==0 {{{write} return;}} self.first(n-1);}}");
        let module = cell(
            &[
                "first(var self,n:i32) {if n==0 {return;} self.second(n-1);}",
                &second,
            ],
            &[],
            false,
        );
        let summary = effect(&module, "Cell.first");
        assert_eq!(summary.parameters[0].replace, replace);
        assert_eq!(summary.parameters[0].write_fields, !replace);
        assert_eq!(effects::permits_finalizing(&summary, 0), !replace);
    }
}
#[test]
fn replacing_a_field_via_a_call_does_not_replace_the_parent() {
    let span = Span {
        source: SourceId(0),
        start: 0,
        end: 0,
    };
    let cell = inline(
        "Cell",
        &[("value", "i32")],
        &["reset(var self) {*self=Cell {value:2};}"],
        &[],
        span,
    );
    let pair = inline(
        "Pair",
        &[("cell", "Cell")],
        &["change(var self) {self.cell.reset();}"],
        &[],
        span,
    );
    let parsed = parse_with_items(SourceId(0), "", vec![cell, pair]);
    let summary = effect(&parsed.module.unwrap(), "Pair.change");
    assert_eq!(
        summary.parameters[0],
        effects::TargetEffects {
            replace: false,
            write_fields: true
        }
    );
    assert!(effects::permits_finalizing(&summary, 0));
}
#[test]
fn recursive_returned_aliases_carry_finalizing_effects() {
    for (write, replace) in [("*alias=Cell {value:2};", true), ("alias.value=2;", false)] {
        let methods = [
            "first(var self,n:i32)->Cell {if n==0 {return self;} return self.second(n-1);}",
            "second(var self,n:i32)->Cell {return self.first(n);}",
        ];
        let body = format!("let ref var alias=self.first(2);{write}");
        let method = format!("change(var self) {{{body}}}");
        let module = cell(&[methods[0], methods[1], &method], &[], true);
        let summary = effect(&module, "Cell.change");
        assert_eq!(
            summary.parameters[0],
            effects::TargetEffects {
                replace,
                write_fields: !replace
            }
        );
        let result = check(&cell(&methods, &[&body], false));
        if replace {
            assert!(result.unwrap_err().iter().any(|e| e.code == "E0391"));
        } else {
            result.unwrap();
        }
    }
}
