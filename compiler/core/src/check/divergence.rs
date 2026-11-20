use super::*;

pub(super) fn expression_diverges(expr: &h::Expr) -> bool {
    match &expr.kind {
        h::ExprKind::While { condition, body } => {
            matches!(condition.kind, h::ExprKind::Value(Value::Bool(true))) && !block_breaks(body)
        }
        h::ExprKind::Block(block) => block.diverges,
        h::ExprKind::Match(arms) => arms.iter().all(|arm| arm.body.diverges),
        h::ExprKind::If {
            yes, no: Some(no), ..
        } => yes.diverges && no.diverges,
        _ => false,
    }
}

pub(super) fn block_breaks(block: &h::Block) -> bool {
    block.statements.iter().any(|s| match s {
        h::Statement::Break => true,
        h::Statement::Initialize(_, e)
        | h::Statement::Evaluate(e)
        | h::Statement::Return(Some(e)) => expr_breaks(e),
        _ => false,
    })
}

pub(super) fn expr_breaks(e: &h::Expr) -> bool {
    use h::ExprKind as E;
    match &e.kind {
        E::While { condition, .. } => expr_breaks(condition),
        E::Block(b) => block_breaks(b),
        E::If { condition, yes, no } => {
            expr_breaks(condition) || block_breaks(yes) || no.as_ref().is_some_and(block_breaks)
        }
        E::Match(arms) => arms.iter().any(|a| {
            expr_breaks(&a.condition)
                || a.guard.as_ref().is_some_and(expr_breaks)
                || block_breaks(&a.body)
                || a.bindings.iter().chain(&a.guard_bindings).any(|s| match s {
                    h::Statement::Evaluate(e) | h::Statement::Initialize(_, e) => expr_breaks(e),
                    h::Statement::Break => true,
                    _ => false,
                })
        }),
        E::StoreView(e)
        | E::StoredView(e)
        | E::Take(e)
        | E::Unary(_, e)
        | E::Cast(e)
        | E::Tag(e)
        | E::Payload(e, _)
        | E::Field(e, _)
        | E::ArgumentView { place: e, .. }
        | E::Borrow { place: e, .. } => expr_breaks(e),
        E::Identity(a, b)
        | E::Binary(_, a, b)
        | E::Index(a, b)
        | E::Assign {
            place: a, value: b, ..
        } => expr_breaks(a) || expr_breaks(b),
        E::StorageCall { arguments, .. }
        | E::BorrowedCall { arguments, .. }
        | E::New { arguments, .. }
        | E::Call(_, arguments)
        | E::Tuple(arguments)
        | E::Array(arguments)
        | E::Enum {
            fields: arguments, ..
        } => arguments.iter().any(expr_breaks),
        E::CallIndirect(c, args, _) => expr_breaks(c) || args.iter().any(expr_breaks),
        E::Value(_) | E::Local(_) | E::View(_) => false,
    }
}

pub(super) fn same(expected: &Type, actual: &Type, span: Span) -> Result<()> {
    if expected.logical() == actual.logical() {
        Ok(())
    } else {
        Err(error(
            span,
            "E0320",
            format!("expected {expected:?}, found {actual:?}"),
        ))
    }
}

pub(super) fn error(span: Span, code: &'static str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(code, message, span)
}
