use super::*;

pub(super) fn mutations(
    body: &a::Block,
    module: &a::Module,
    scope: &Scope,
    result: &mut BTreeSet<String>,
) {
    // Mark only free uses before inspecting effects; assignments to a shadowing
    // local must not turn an outer readonly capture into a mutable one.
    let mut body = body.clone();
    rewrite_block(&mut body, &mut Scope::new(), &mut |value| {
        if let a::ExprKind::Name(name) = &mut value.kind {
            *name = format!("$free${name}");
        }
    });
    let mut found = BTreeSet::new();
    scan(&body, module, scope, &mut found);
    result.extend(
        found
            .into_iter()
            .filter_map(|name| name.strip_prefix("$free$").map(str::to_owned)),
    );
}

pub(super) fn scan(
    body: &a::Block,
    module: &a::Module,
    scope: &Scope,
    result: &mut BTreeSet<String>,
) {
    fn root(value: &a::Expr) -> Option<String> {
        match &value.kind {
            a::ExprKind::Name(name) => Some(name.clone()),
            a::ExprKind::Member { value, .. }
            | a::ExprKind::Index { value, .. }
            | a::ExprKind::Group(value) => root(value),
            _ => None,
        }
    }
    fn expression(
        value: &a::Expr,
        module: &a::Module,
        scope: &Scope,
        result: &mut BTreeSet<String>,
    ) {
        use a::ExprKind as E;
        if let E::Assign { place, .. } = &value.kind {
            if let Some(name) = root(place) {
                result.insert(name);
            }
        }
        if let E::Call {
            callee, arguments, ..
        } = &value.kind
        {
            match &callee.kind {
                E::Name(name) => {
                    let name = name.strip_prefix("$free$").unwrap_or(name);
                    if let Some(function) = module.items.iter().find_map(|item| match &item.kind {
                        a::ItemKind::Function(function) if function.name == name => Some(function),
                        _ => None,
                    }) {
                        for (parameter, value) in function.parameters.iter().zip(arguments) {
                            if parameter.mutable {
                                if let Some(name) = root(value) {
                                    result.insert(name);
                                }
                            }
                        }
                    }
                }
                E::Member { value, name } => {
                    if let Some(owner) = receiver_type(value, scope, module) {
                        if let Some(method) =
                            find_member(module, &owner, name).and_then(|member| match member {
                                ResolvedMember::Method(f) => Some(f),
                                _ => None,
                            })
                        {
                            for (parameter, argument) in method
                                .parameters
                                .iter()
                                .zip(std::iter::once(value.as_ref()).chain(arguments))
                            {
                                if parameter.mutable {
                                    result.extend(root(argument));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        // Reuse the scope-aware traversal for direct children, while explicitly
        // descending into expressions that can contain writes.
        match &value.kind {
            E::Call {
                callee, arguments, ..
            } => {
                expression(callee, module, scope, result);
                for value in arguments {
                    expression(value, module, scope, result);
                }
            }
            E::New { arguments, .. } | E::Tuple(arguments) | E::Array(arguments) => {
                for value in arguments {
                    expression(value, module, scope, result);
                }
            }
            E::Construct { fields, .. } => {
                for (_, value) in fields {
                    expression(value, module, scope, result);
                }
            }
            E::Group(value)
            | E::Unary { value, .. }
            | E::Cast { value, .. }
            | E::Member { value, .. }
            | E::Specialize { value, .. } => expression(value, module, scope, result),
            E::Index { value, index } => {
                expression(value, module, scope, result);
                expression(index, module, scope, result);
            }
            E::Binary { left, right, .. }
            | E::Assign {
                place: left,
                value: right,
                ..
            } => {
                expression(left, module, scope, result);
                expression(right, module, scope, result);
            }
            E::If {
                condition,
                then_block,
                else_branch,
            } => {
                expression(condition, module, scope, result);
                scan(then_block, module, scope, result);
                if let Some(value) = else_branch {
                    expression(value, module, scope, result);
                }
            }
            E::While { condition, body } => {
                expression(condition, module, scope, result);
                scan(body, module, scope, result);
            }
            E::Branch(body) => scan(body, module, scope, result),
            E::Match { value, arms } => {
                expression(value, module, scope, result);
                for arm in arms {
                    if let Some(value) = &arm.guard {
                        expression(value, module, scope, result);
                    }
                    expression(&arm.value, module, scope, result);
                }
            }
            _ => {}
        }
    }
    for statement in &body.statements {
        match &statement.kind {
            a::StatementKind::Binding { value, .. }
            | a::StatementKind::Expression(value)
            | a::StatementKind::Return(Some(value)) => expression(value, module, scope, result),
            a::StatementKind::Unsafe(body) => scan(body, module, scope, result),
            _ => {}
        }
    }
}
