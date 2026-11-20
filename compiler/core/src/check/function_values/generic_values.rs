use super::*;

impl Checker<'_> {
    pub(super) fn generic_function_value(
        &mut self,
        name: &str,
        types: &[a::TypeArgument],
        expected: Option<&Type>,
        span: Span,
    ) -> Result<h::Expr> {
        let Some(Type::Function {
            parameters, result, ..
        }) = expected
        else {
            return Err(error(
                span,
                "E0390",
                "generic function value requires a concrete function type",
            ));
        };
        let template = self.mono.borrow().templates[name].clone();
        if template.parameters.len() != parameters.len()
            || template
                .parameters
                .iter()
                .zip(parameters)
                .any(|(p, (mutable, _))| p.mutable != *mutable)
        {
            return Err(error(
                span,
                "E0390",
                "function pointer parameter modes or count do not match",
            ));
        }
        // Typed synthetic arguments reuse ordinary specialization and abstract
        // body auditing, but never enter the resulting HIR.
        let start = self.locals.len();
        self.scopes.push(BTreeMap::new());
        self.unsafe_depth += 1;
        let resolved = (|| {
            let mut arguments = Vec::new();
            for (index, (mutable, ty)) in parameters.iter().enumerate() {
                let name = format!("<function value argument {index}>");
                let id = self.local(name.clone(), ty.clone(), *mutable, span)?;
                self.callable_parameters.insert(id);
                arguments.push(a::Expr {
                    kind: a::ExprKind::Name(name),
                    span,
                });
            }
            self.generic_call(name, types, &arguments, Some(result), span)
        })();
        self.scopes.pop();
        self.locals.truncate(start);
        self.callable_parameters.retain(|id| *id < start);
        self.unsafe_depth -= 1;
        let call = resolved?;
        same(result, &call.ty, span)?;
        let pointer = self.pointer_from_call(call, span)?;
        let Value::Function(id) = (match &pointer.kind {
            h::ExprKind::Value(v) => v.clone(),
            _ => unreachable!(),
        }) else {
            unreachable!()
        };
        let signature = {
            let mono = self.mono.borrow();
            mono.instances
                .values()
                .chain(mono.entries.values())
                .find(|s| s.id == id)
                .cloned()
                .unwrap()
        };
        self.pointer_entry(signature, expected, span)
    }
}
