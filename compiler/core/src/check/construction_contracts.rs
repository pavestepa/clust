//! Source-level base initialization obligations, checked before payload lowering.
use super::*;
use std::collections::BTreeSet;

pub(super) fn validate(module: &a::Module) -> Result<()> {
    let classes: BTreeSet<_> = module
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            a::ItemKind::Aggregate {
                class: true, name, ..
            } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    for item in &module.items {
        let a::ItemKind::Aggregate {
            class: true,
            interfaces,
            members,
            ..
        } = &item.kind
        else {
            continue;
        };
        let base = interfaces.iter().any(|ty| matches!(&ty.kind, a::TypeKind::Named {path, ..} if classes.contains(path.join("::").as_str())));
        for member in members {
            match member {
                a::Member::Constructor {
                    body, parameters, ..
                } => {
                    let mut checker = Contract {
                        base,
                        constructor: true,
                        repeated: false,
                    };
                    let mut state = State {
                        counts: BTreeSet::from([0]),
                        locals: BTreeSet::new(),
                        fields: members
                            .iter()
                            .filter_map(|m| match m {
                                a::Member::Field { name, .. } => Some(name.clone()),
                                _ => None,
                            })
                            .collect(),
                    };
                    for parameter in parameters {
                        bind(&parameter.pattern, &mut state.locals);
                    }
                    checker.block(body, &mut state)?;
                    if base && state.counts != BTreeSet::from([1]) {
                        return Err(error(
                            body.span,
                            "E0393",
                            "every constructor path must initialize the direct base exactly once",
                        ));
                    }
                }
                a::Member::Method {
                    function:
                        a::Function {
                            body: Some(body), ..
                        },
                    ..
                }
                | a::Member::Destructor(body) => {
                    Contract {
                        base,
                        constructor: false,
                        repeated: false,
                    }
                    .block(
                        body,
                        &mut State {
                            counts: BTreeSet::from([1]),
                            locals: BTreeSet::new(),
                            fields: BTreeSet::new(),
                        },
                    )?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

#[derive(Clone)]
struct State {
    counts: BTreeSet<u8>,
    locals: BTreeSet<String>,
    fields: BTreeSet<String>,
}
struct Contract {
    base: bool,
    constructor: bool,
    repeated: bool,
}
impl Contract {
    fn block(&mut self, block: &a::Block, state: &mut State) -> Result<()> {
        let outer = state.locals.clone();
        for statement in &block.statements {
            match &statement.kind {
                a::StatementKind::Binding { pattern, value, .. } => {
                    self.expr(value, state)?;
                    bind(pattern, &mut state.locals);
                }
                a::StatementKind::Expression(value) => self.expr(value, state)?,
                a::StatementKind::Return(value) => {
                    if self.constructor {
                        return Err(error(
                            statement.span,
                            "E0393",
                            "constructors cannot return early",
                        ));
                    }
                    if let Some(value) = value {
                        self.expr(value, state)?;
                    }
                }
                a::StatementKind::Unsafe(body) => self.block(body, state)?,
                a::StatementKind::Break | a::StatementKind::Continue => {}
            }
        }
        state.locals = outer;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn constructor(body: &str) -> Result<()> {
        let source = format!(
            "class Base {{new() {{}}}} class Derived:Base {{value:i32;new(flag:bool) {{{body}}}}}"
        );
        let parsed =
            clust_frontend::parser::parse_clust(clust_frontend::source::SourceId(0), &source);
        assert!(
            parsed.diagnostics.is_empty(),
            "{source}: {:?}",
            parsed.diagnostics
        );
        validate(&parsed.module.unwrap())
    }
    #[test]
    fn every_path_initializes_the_base_once_before_self_access() {
        for body in [
            "super.new();self.value=1;",
            "let n=1;if flag {super.new();} else {super.new();} self.value=n;",
            "if flag {let n=1;super.new();self.value=n;} else {super.new();self.value=2;}",
            "match flag {true=>{super.new();},false=>{super.new();}};self.value=1;",
            "let value=7;let n=value;super.new();self.value=n;",
        ] {
            constructor(body).unwrap_or_else(|error| panic!("{body}: {error:?}"));
        }
        for body in [
            "self.value=1;super.new();",
            "let alias=self;super.new();self.value=1;",
            "super.new();super.new();self.value=1;",
            "if flag {super.new();} self.value=1;",
            "if flag {super.new();} super.new();self.value=1;",
            "while flag {super.new();} self.value=1;",
            "super.new(self.value);self.value=1;",
            "super.new(if flag {super.new();} else {super.new();});self.value=1;",
            "match flag {true=>{super.new();},false=>{}};self.value=1;",
            "value=1;super.new();",
        ] {
            assert_eq!(constructor(body).unwrap_err().code, "E0393", "{body}");
        }
    }
    #[test]
    fn base_construction_is_not_an_ordinary_method_call() {
        for source in [
            "class Root {new() {super.new();}}",
            "class Base {new() {}} class Derived:Base {new() {super.new();} pub reset(var self) {super.new();}}",
            "class Base {new() {}} class Derived:Base {new() {super.new();} drop() {super.new();}}",
        ] {
            let parsed=clust_frontend::parser::parse_clust(clust_frontend::source::SourceId(0),source);
            assert!(parsed.diagnostics.is_empty(),"{:?}",parsed.diagnostics);
            assert_eq!(validate(&parsed.module.unwrap()).unwrap_err().code,"E0393");
        }
    }
}

mod bindings;
mod expressions;
use self::bindings::bind;
