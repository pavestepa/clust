use super::*;

pub(super) fn rewrite_block(
    body: &mut a::Block,
    bound: &mut Scope,
    visit: &mut impl FnMut(&mut a::Expr),
) {
    for statement in &mut body.statements {
        match &mut statement.kind {
            a::StatementKind::Binding { pattern, value, .. } => {
                rewrite(value, bound, visit);
                bind(
                    pattern,
                    Binding {
                        ty: None,
                        mutable: false,
                        parameter: false,
                        scoped: None,
                    },
                    bound,
                );
            }
            a::StatementKind::Expression(value) | a::StatementKind::Return(Some(value)) => {
                rewrite(value, bound, visit)
            }
            a::StatementKind::Unsafe(body) => rewrite_block(body, &mut bound.clone(), visit),
            _ => {}
        }
    }
}

pub(super) fn rewrite(value: &mut a::Expr, bound: &Scope, visit: &mut impl FnMut(&mut a::Expr)) {
    use a::ExprKind as E;
    match &mut value.kind {
        E::Name(name) if !bound.contains_key(name) => visit(value),
        E::Call {
            callee, arguments, ..
        } => {
            rewrite(callee, bound, visit);
            for value in arguments {
                rewrite(value, bound, visit);
            }
        }
        E::New { arguments, .. } | E::Tuple(arguments) | E::Array(arguments) => {
            for value in arguments {
                rewrite(value, bound, visit);
            }
        }
        E::Construct { fields, .. } => {
            for (_, value) in fields {
                rewrite(value, bound, visit);
            }
        }
        E::Group(value)
        | E::Unary { value, .. }
        | E::Cast { value, .. }
        | E::Member { value, .. }
        | E::Specialize { value, .. } => rewrite(value, bound, visit),
        E::Index { value, index } => {
            rewrite(value, bound, visit);
            rewrite(index, bound, visit);
        }
        E::Binary { left, right, .. }
        | E::Assign {
            place: left,
            value: right,
            ..
        } => {
            rewrite(left, bound, visit);
            rewrite(right, bound, visit);
        }
        E::If {
            condition,
            then_block,
            else_branch,
        } => {
            rewrite(condition, bound, visit);
            rewrite_block(then_block, &mut bound.clone(), visit);
            if let Some(value) = else_branch {
                rewrite(value, bound, visit);
            }
        }
        E::While { condition, body } => {
            rewrite(condition, bound, visit);
            rewrite_block(body, &mut bound.clone(), visit);
        }
        E::Branch(body) => rewrite_block(body, &mut bound.clone(), visit),
        E::Match { value, arms } => {
            rewrite(value, bound, visit);
            for arm in arms {
                let mut local = bound.clone();
                bind(
                    &arm.pattern,
                    Binding {
                        ty: None,
                        mutable: false,
                        parameter: false,
                        scoped: None,
                    },
                    &mut local,
                );
                if let Some(value) = &mut arm.guard {
                    rewrite(value, &local, visit);
                }
                rewrite(&mut arm.value, &local, visit);
            }
        }
        _ => {}
    }
}
