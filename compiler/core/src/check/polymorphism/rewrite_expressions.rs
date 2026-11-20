use super::*;

pub(super) fn rewrite_block(block: &mut a::Block, roots: &BTreeSet<String>) {
    for statement in &mut block.statements {
        match &mut statement.kind {
            a::StatementKind::Binding { ty, value, .. } => {
                if let Some(ty) = ty {
                    rewrite_type(ty, roots);
                }
                rewrite_expr(value, roots);
            }
            a::StatementKind::Expression(value) | a::StatementKind::Return(Some(value)) => {
                rewrite_expr(value, roots)
            }
            a::StatementKind::Unsafe(body) => rewrite_block(body, roots),
            _ => {}
        }
    }
}

pub(super) fn rewrite_expr(value: &mut a::Expr, roots: &BTreeSet<String>) {
    use a::ExprKind as E;
    match &mut value.kind {
        E::Call {
            callee,
            types,
            arguments,
        } => {
            rewrite_expr(callee, roots);
            for arg in types {
                rewrite_argument(arg, roots);
            }
            for value in arguments {
                rewrite_expr(value, roots);
            }
        }
        E::New { ty, arguments } => {
            if let a::TypeKind::Named { arguments, .. } = &mut ty.kind {
                for arg in arguments {
                    rewrite_argument(arg, roots);
                }
            }
            for value in arguments {
                rewrite_expr(value, roots);
            }
        }
        E::Construct { ty, fields } => {
            if let a::TypeKind::Named { arguments, .. } = &mut ty.kind {
                for arg in arguments {
                    rewrite_argument(arg, roots);
                }
            }
            for (_, value) in fields {
                rewrite_expr(value, roots);
            }
        }
        E::Specialize { value, arguments } => {
            rewrite_expr(value, roots);
            for arg in arguments {
                rewrite_argument(arg, roots);
            }
        }
        E::Cast { value, ty } => {
            rewrite_expr(value, roots);
            rewrite_type(ty, roots);
        }
        E::Group(value) | E::Unary { value, .. } | E::Member { value, .. } => {
            rewrite_expr(value, roots)
        }
        E::Tuple(values) | E::Array(values) => {
            for value in values {
                rewrite_expr(value, roots);
            }
        }
        E::Index { value, index } => {
            rewrite_expr(value, roots);
            rewrite_expr(index, roots);
        }
        E::Binary { left, right, .. }
        | E::Assign {
            place: left,
            value: right,
            ..
        } => {
            rewrite_expr(left, roots);
            rewrite_expr(right, roots);
        }
        E::If {
            condition,
            then_block,
            else_branch,
        } => {
            rewrite_expr(condition, roots);
            rewrite_block(then_block, roots);
            if let Some(branch) = else_branch {
                rewrite_expr(branch, roots);
            }
        }
        E::While { condition, body } => {
            rewrite_expr(condition, roots);
            rewrite_block(body, roots);
        }
        E::Branch(body) => rewrite_block(body, roots),
        E::Match { value, arms } => {
            rewrite_expr(value, roots);
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    rewrite_expr(guard, roots);
                }
                rewrite_expr(&mut arm.value, roots);
            }
        }
        _ => {}
    }
}
