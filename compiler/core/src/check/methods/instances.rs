use super::*;

impl Checker<'_> {
    pub(in super::super) fn instance_call(
        &mut self,
        callee: &a::Expr,
        types: &[a::TypeArgument],
        arguments: &[a::Expr],
        expected: Option<&Type>,
        span: Span,
    ) -> Result<Option<h::Expr>> {
        let (receiver, method) = match &callee.kind {
            a::ExprKind::Member { value, name } => {
                if let a::ExprKind::Name(name) = &value.kind {
                    if self.lookup(name).is_none() && self.nominals.contains(name) {
                        return Ok(None);
                    }
                }
                if matches!(value.kind, a::ExprKind::Specialize { .. }) {
                    return Ok(None);
                }
                ((**value).clone(), name.clone())
            }
            a::ExprKind::Name(name)
                if self.lookup(name).is_none() && self.lookup("this").is_some() =>
            {
                (
                    a::Expr {
                        kind: a::ExprKind::Name("this".into()),
                        span,
                    },
                    name.clone(),
                )
            }
            _ => return Ok(None),
        };
        let value = self.expr(&receiver, None, true)?;
        let (Type::Record { name, .. } | Type::Class { name, .. } | Type::Enum { name, .. }) =
            &value.ty
        else {
            return Ok(None);
        };
        let mut candidate = format!("{}.{}", self.nominals.origin(name), method);
        if let Some(owner) = &self.member_owner {
            let private = class_bases::helper(owner, &self.nominals.origin(name), &method, true);
            if self.signatures.contains_key(&private)
                || self.mono.borrow().templates.contains_key(&private)
            {
                candidate = private;
            }
        }
        let receiver_method = self
            .signatures
            .get(&candidate)
            .is_some_and(|s| s.modes.first().is_some_and(Option::is_some))
            || self
                .mono
                .borrow()
                .templates
                .get(&candidate)
                .is_some_and(|f| f.parameters.first().is_some_and(|p| p.receiver));
        if !receiver_method {
            return Ok(None);
        }
        let mut supplied = self
            .nominals
            .arguments(name)
            .into_iter()
            .map(|a| match a {
                records::Argument::Type(ty) => {
                    a::TypeArgument::Type(generics::substitute::ast_type(&ty, span))
                }
                records::Argument::Const(n) => a::TypeArgument::Const(a::Expr {
                    kind: a::ExprKind::Literal(a::Literal::Integer(n.to_string())),
                    span,
                }),
            })
            .collect::<Vec<_>>();
        supplied.extend_from_slice(types);
        let mut args = vec![receiver];
        args.extend_from_slice(arguments);
        let call = a::Expr {
            kind: a::ExprKind::Call {
                callee: Box::new(a::Expr {
                    kind: a::ExprKind::Name(candidate),
                    span,
                }),
                types: supplied,
                arguments: args,
            },
            span,
        };
        self.expr(&call, expected, true).map(Some)
    }
}
