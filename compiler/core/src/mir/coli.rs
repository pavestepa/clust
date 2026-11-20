//! Ownership diagnostics for checked control flow.
use super::{borrow_check, dataflow, Function, Program};
use clust_frontend::source::Diagnostic;

/// Report live loan conflicts and moved or uninitialized places, in source-check order.
pub fn validate(program: &Program) -> Vec<Diagnostic> {
    let mut errors: Vec<_> = program
        .functions
        .iter()
        .flat_map(|f| loan_errors(f, "E0381"))
        .collect();
    for function in &program.functions {
        for (block, place) in dataflow::place_initialization_errors(function) {
            errors.push(Diagnostic::new(
                "E0384",
                format!(
                    "use of moved or uninitialized place '{}'",
                    function.locals[place.local].name
                ),
                function.blocks[block].span,
            ));
        }
    }
    errors
}
pub(super) fn loan_errors<'a>(
    f: &'a Function,
    code: &'static str,
) -> impl Iterator<Item = Diagnostic> + 'a {
    borrow_check::check(f)
        .into_iter()
        .map(move |e| borrow_check::diagnostic(f, e, code))
}
pub mod foreign;
