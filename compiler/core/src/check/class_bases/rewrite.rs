use super::*;

pub(super) fn rewrite_block(
    block: &mut a::Block,
    owner: &str,
    actual: &str,
    classes: &BTreeMap<String, Class>,
    constructor: bool,
) -> Result<()> {
    for statement in &mut block.statements {
        match &mut statement.kind {
            a::StatementKind::Binding { value, .. }
            | a::StatementKind::Expression(value)
            | a::StatementKind::Return(Some(value)) => {
                rewrite(value, owner, actual, classes, constructor)?
            }
            a::StatementKind::Unsafe(body) => {
                rewrite_block(body, owner, actual, classes, constructor)?
            }
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn rewrite(
    value: &mut a::Expr,
    owner: &str,
    actual: &str,
    classes: &BTreeMap<String, Class>,
    constructor: bool,
) -> Result<()> {
    use a::ExprKind as E;
    let span = value.span;
    match &mut value.kind {
        E::Closure(_) => return Err(error(span, "E0405", "closure was not lowered")),
        E::Call {
            callee,
            types,
            arguments,
        } => {
            for argument in arguments.iter_mut() {
                rewrite(argument, owner, actual, classes, constructor)?;
            }
            if let E::Member {
                value: receiver,
                name: method,
            } = &callee.kind
            {
                if matches!(&receiver.kind,E::Name(name) if name=="super") {
                    let base = classes[owner]
                        .base
                        .as_ref()
                        .ok_or_else(|| error(span, "E0413", "root class has no super"))?;
                    if method == "new" {
                        if !constructor {
                            return Err(error(
                                span,
                                "E0393",
                                "super.new is only valid in a derived constructor",
                            ));
                        }
                        value.kind = E::Assign {
                            operator: "=",
                            place: Box::new(a::Expr {
                                kind: E::Member {
                                    value: Box::new(name("this", span)),
                                    name: BASE.into(),
                                },
                                span,
                            }),
                            value: Box::new(a::Expr {
                                kind: E::New {
                                    ty: a::Type {
                                        kind: a::TypeKind::Named {
                                            path: vec![base.clone()],
                                            arguments: match &classes[owner].item.kind {
                                                a::ItemKind::Aggregate { interfaces, .. } => {
                                                    interfaces
                                                        .iter()
                                                        .find_map(|ty| match &ty.kind {
                                                            a::TypeKind::Named {
                                                                path,
                                                                arguments,
                                                            } if path.join("::") == *base => {
                                                                Some(arguments.clone())
                                                            }
                                                            _ => None,
                                                        })
                                                        .unwrap()
                                                }
                                                _ => unreachable!(),
                                            },
                                        },
                                        span,
                                    },
                                    arguments: arguments.clone(),
                                },
                                span,
                            }),
                        };
                    } else {
                        let mut declaring = Some(base.clone());
                        let mut target = None;
                        while let Some(base) = declaring {
                            let a::ItemKind::Aggregate { members, .. } = &classes[&base].item.kind
                            else {
                                unreachable!()
                            };
                            if members.iter().any(|member|matches!(member,a::Member::Method {function,private:false,..} if function.name==*method)) {target=Some(base);break;}
                            declaring = classes[&base].base.clone();
                        }
                        let target = target.ok_or_else(|| {
                            error(span, "E0413", "super method is not accessible")
                        })?;
                        value.kind = E::Call {
                            callee: Box::new(name(&helper(&target, actual, method, false), span)),
                            types: match &classes[actual].item.kind {
                                a::ItemKind::Aggregate { generics, .. } => {
                                    closures::generic_arguments(generics)
                                        .into_iter()
                                        .chain(types.clone())
                                        .collect()
                                }
                                _ => unreachable!(),
                            },
                            arguments: std::iter::once(name("this", span))
                                .chain(arguments.clone())
                                .collect(),
                        };
                    }
                    return Ok(());
                }
            }
            rewrite(callee, owner, actual, classes, constructor)?;
        }
        E::Group(v)
        | E::Unary { value: v, .. }
        | E::Cast { value: v, .. }
        | E::Specialize { value: v, .. }
        | E::Member { value: v, .. } => rewrite(v, owner, actual, classes, constructor)?,
        E::Tuple(values)
        | E::Array(values)
        | E::New {
            arguments: values, ..
        } => {
            for value in values {
                rewrite(value, owner, actual, classes, constructor)?;
            }
        }
        E::Construct { fields, .. } => {
            for (_, value) in fields {
                rewrite(value, owner, actual, classes, constructor)?;
            }
        }
        E::Index { value, index } => {
            rewrite(value, owner, actual, classes, constructor)?;
            rewrite(index, owner, actual, classes, constructor)?;
        }
        E::Binary { left, right, .. }
        | E::Assign {
            place: left,
            value: right,
            ..
        } => {
            rewrite(left, owner, actual, classes, constructor)?;
            rewrite(right, owner, actual, classes, constructor)?;
        }
        E::If {
            condition,
            then_block,
            else_branch,
        } => {
            rewrite(condition, owner, actual, classes, constructor)?;
            rewrite_block(then_block, owner, actual, classes, constructor)?;
            if let Some(branch) = else_branch {
                rewrite(branch, owner, actual, classes, constructor)?;
            }
        }
        E::While { condition, body } => {
            rewrite(condition, owner, actual, classes, constructor)?;
            rewrite_block(body, owner, actual, classes, constructor)?;
        }
        E::Branch(body) => rewrite_block(body, owner, actual, classes, constructor)?,
        E::Match { value, arms } => {
            rewrite(value, owner, actual, classes, constructor)?;
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    rewrite(guard, owner, actual, classes, constructor)?;
                }
                rewrite(&mut arm.value, owner, actual, classes, constructor)?;
            }
        }
        E::Name(_) | E::Literal(_) => {}
    }
    Ok(())
}
