//! Definite field initialization before a constructor exposes its whole receiver.
//! The AST check is retained for nominal-registry validation. Source compilation
//! validates declarations here and uses typed MIR field analysis in classes_init.
use super::*;
use std::collections::BTreeSet;
#[derive(Clone)]
struct State {
    initialized: BTreeSet<String>,
    locals: BTreeSet<String>,
    reachable: bool,
}
struct Constructor {
    fields: BTreeSet<String>,
    methods: BTreeSet<String>,
}
pub(super) fn declarations(members: &[a::Member]) -> Result<()> {
    let mut seen = false;
    for member in members {
        if let a::Member::Constructor {
            parameters, span, ..
        } = member
        {
            if seen {
                return Err(error(*span, "E0393", "duplicate constructor"));
            }
            seen = true;
            if let Some(p) = parameters.iter().find(|p| p.receiver) {
                return Err(error(p.span, "E0393", "constructor receiver is implicit"));
            }
        }
    }
    Ok(())
}
pub(super) fn validate(members: &[a::Member]) -> Result<()> {
    let fields = members
        .iter()
        .filter_map(|m| match m {
            a::Member::Field { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();
    let methods = members
        .iter()
        .filter_map(|m| match m {
            a::Member::Method { function, .. }
                if function.parameters.first().is_some_and(|p| p.receiver) =>
            {
                Some(function.name.clone())
            }
            _ => None,
        })
        .collect();
    let checker = Constructor { fields, methods };
    let mut seen = false;
    for member in members {
        if let a::Member::Constructor {
            parameters,
            body,
            span,
        } = member
        {
            if seen {
                return Err(error(*span, "E0393", "duplicate constructor"));
            }
            seen = true;
            let mut state = State {
                initialized: BTreeSet::new(),
                locals: BTreeSet::new(),
                reachable: true,
            };
            for parameter in parameters {
                if parameter.receiver {
                    return Err(error(
                        parameter.span,
                        "E0393",
                        "constructor receiver is implicit",
                    ));
                }
                bindings(&parameter.pattern, &mut state.locals);
            }
            checker.block(body, &mut state)?;
            if state.reachable {
                checker.complete(&state, *span)?;
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn check(body: &str) -> Result<()> {
        let source=format!("pub class C {{pub a:i32;pub b:i32;pub use_all(self) -> i32 {{return a+b;}};new(a:i32,flag:bool) {{{body};}}}}");
        let parsed = clust_frontend::parser::parse(clust_frontend::source::SourceId(0), &source);
        if let Some(error) = parsed.diagnostics.into_iter().next() {
            return Err(error);
        }
        let a::ItemKind::Aggregate { members, .. } = &parsed.module.unwrap().items[0].kind else {
            panic!()
        };
        validate(members)
    }
    #[test]
    fn all_reachable_exits_require_initialized_fields() {
        for body in [
            "self.a=a;b=2;",
            "if flag {self.a=a} else {self.a=3};b=2;",
            "self.a=a;b=self.a;use_all();",
            "if flag {self.a=1;b=2;} else {self.a=3;b=4;}",
        ] {
            check(body).unwrap_or_else(|e| panic!("{body}: {e:?}"));
        }
        for body in [
            "a=a;b=2;",
            "if flag {self.a=1};b=2;",
            "self.a=b;b=2;",
            "self.a=1;use_all();b=2;",
            "self.a=1;let alias=self;b=2;",
            "self.a=1;return;",
            "while flag {self.a=1;};b=2;",
            "self.a+=1;b=2;",
        ] {
            assert!(check(body).is_err(), "accepted {body}");
        }
    }
}

mod bindings;
mod expressions;
mod initialization;
use self::bindings::bindings;
use self::bindings::is_this;
use self::bindings::join;
