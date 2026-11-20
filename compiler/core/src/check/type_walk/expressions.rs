use super::*;

pub(super) fn block(body: &a::Block, visit: &mut impl FnMut(&a::Type)) {
    for statement in &body.statements {
        match &statement.kind {
            a::StatementKind::Binding {
                ty: value,
                value: expr,
                ..
            } => {
                if let Some(value) = value {
                    ty(value, visit);
                }
                expression(expr, visit);
            }
            a::StatementKind::Expression(value) | a::StatementKind::Return(Some(value)) => {
                expression(value, visit)
            }
            a::StatementKind::Unsafe(body) => block(body, visit),
            _ => {}
        }
    }
}

pub(super) fn expression(value: &a::Expr, visit: &mut impl FnMut(&a::Type)) {
    use a::ExprKind as E;
    match &value.kind {
        E::Call {
            callee,
            types,
            arguments,
        } => {
            expression(callee, visit);
            for value in types {
                arg(value, visit);
            }
            for value in arguments {
                expression(value, visit);
            }
        }
        E::New {
            ty: value,
            arguments,
        } => {
            ty(value, visit);
            for value in arguments {
                expression(value, visit);
            }
        }
        E::Construct { ty: value, fields } => {
            ty(value, visit);
            for (_, value) in fields {
                expression(value, visit);
            }
        }
        E::Specialize { value, arguments } => {
            expression(value, visit);
            for value in arguments {
                arg(value, visit);
            }
        }
        E::Cast { value, ty: target } => {
            expression(value, visit);
            ty(target, visit);
        }
        E::Group(value) | E::Unary { value, .. } | E::Member { value, .. } => {
            expression(value, visit)
        }
        E::Tuple(values) | E::Array(values) => {
            for value in values {
                expression(value, visit);
            }
        }
        E::Index { value, index } => {
            expression(value, visit);
            expression(index, visit);
        }
        E::Binary { left, right, .. }
        | E::Assign {
            place: left,
            value: right,
            ..
        } => {
            expression(left, visit);
            expression(right, visit);
        }
        E::If {
            condition,
            then_block,
            else_branch,
        } => {
            expression(condition, visit);
            block(then_block, visit);
            if let Some(value) = else_branch {
                expression(value, visit);
            }
        }
        E::While { condition, body } => {
            expression(condition, visit);
            block(body, visit);
        }
        E::Branch(body) => block(body, visit),
        E::Match { value, arms } => {
            expression(value, visit);
            for arm in arms {
                if let Some(value) = &arm.guard {
                    expression(value, visit);
                }
                expression(&arm.value, visit);
            }
        }
        _ => {}
    }
}
