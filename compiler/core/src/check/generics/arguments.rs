use super::*;
impl Checker<'_> {
    pub(super) fn generic_arguments(
        &mut self,
        template: &a::Function,
        types: &[a::TypeArgument],
        arguments: &[a::Expr],
        expected: Option<&Type>,
        span: Span,
    ) -> Result<(Substitution, Vec<h::Expr>)> {
        if template.parameters.len() != arguments.len() {
            return Err(error(span, "E0313", "wrong number of arguments"));
        }
        if types.len() > template.generics.len() {
            return Err(error(span, "E0370", "too many generic arguments"));
        }
        let mut substitutions = Substitution::default();
        let mut type_names = std::collections::BTreeSet::new();
        let mut const_names = std::collections::BTreeSet::new();
        for generic in &template.generics {
            match &generic.kind {
                a::GenericKind::Type { .. } => {
                    if !type_names.insert(generic.name.clone())
                        || const_names.contains(&generic.name)
                    {
                        return Err(error(generic.span, "E0301", "duplicate generic parameter"));
                    }
                }
                a::GenericKind::Const { .. } => {
                    if !const_names.insert(generic.name.clone())
                        || type_names.contains(&generic.name)
                    {
                        return Err(error(generic.span, "E0301", "duplicate generic parameter"));
                    }
                }
            }
        }
        for (generic, value) in template.generics.iter().zip(types) {
            match (&generic.kind, value) {
                (a::GenericKind::Type { .. }, a::TypeArgument::Type(ty)) => {
                    substitutions.types.insert(
                        generic.name.clone(),
                        resolve_type_with(ty, self.constants, self.nominals)?,
                    );
                }
                (a::GenericKind::Const { .. }, a::TypeArgument::Const(value)) => {
                    let Value::Integer(value) =
                        constant_value(value, &Type::Integer(I::Usize), self.constants)?
                    else {
                        unreachable!()
                    };
                    substitutions
                        .constants
                        .insert(generic.name.clone(), value.bits() as u64);
                }
                (
                    a::GenericKind::Const { .. },
                    a::TypeArgument::Type(a::Type {
                        kind: a::TypeKind::Named { path, arguments },
                        ..
                    }),
                ) if path.len() == 1 && arguments.is_empty() => {
                    let value = a::Expr {
                        kind: a::ExprKind::Name(path[0].clone()),
                        span,
                    };
                    let Value::Integer(value) =
                        constant_value(&value, &Type::Integer(I::Usize), self.constants)?
                    else {
                        unreachable!()
                    };
                    substitutions
                        .constants
                        .insert(generic.name.clone(), value.bits() as u64);
                }
                _ => return Err(error(span, "E0370", "generic argument kind mismatch")),
            }
        }
        if let Some(expected) = expected {
            infer(
                &template.result,
                &expected.logical(),
                &type_names,
                &const_names,
                &mut substitutions,
                self.nominals,
                span,
            )?;
        }
        // Use nonliteral argument information before defaulting integer literals.
        for (parameter, arg) in template.parameters.iter().zip(arguments) {
            if let Some(ty) = self.known_type(arg) {
                let mut source = parameter.ty.as_ref().unwrap().clone();
                substitutions.ty(&mut source);
                if resolve_type_with(&source, self.constants, self.nominals)
                    .is_ok_and(|expected| self.class_subtype(&expected, &ty))
                {
                    continue;
                }
                infer(
                    parameter.ty.as_ref().unwrap(),
                    &ty.logical(),
                    &type_names,
                    &const_names,
                    &mut substitutions,
                    self.nominals,
                    span,
                )?;
            }
        }
        let mut checked = Vec::new();
        for (parameter, arg) in template.parameters.iter().zip(arguments) {
            let source = parameter.ty.as_ref().unwrap();
            let mut ty = source.clone();
            substitutions.ty(&mut ty);
            let hint = resolve_type_with(&ty, self.constants, self.nominals).ok();
            let arg = self.expr(
                arg,
                hint.as_ref().filter(|ty| !polymorphism::is_polymorphic(ty)),
                true,
            )?;
            if !hint
                .as_ref()
                .is_some_and(|expected| self.class_subtype(expected, &arg.ty))
            {
                infer(
                    source,
                    &arg.ty.logical(),
                    &type_names,
                    &const_names,
                    &mut substitutions,
                    self.nominals,
                    span,
                )?;
            }
            checked.push(arg);
        }
        for generic in &template.generics {
            match &generic.kind {
                a::GenericKind::Type {
                    default: Some(ty), ..
                } if !substitutions.types.contains_key(&generic.name) => {
                    let mut ty = ty.clone();
                    substitutions.ty(&mut ty);
                    if let Ok(ty) = resolve_type_with(&ty, self.constants, self.nominals) {
                        substitutions.types.insert(generic.name.clone(), ty);
                    }
                }
                a::GenericKind::Const {
                    default: Some(value),
                    ..
                } if !substitutions.constants.contains_key(&generic.name) => {
                    let mut value = value.clone();
                    substitutions.expr(&mut value);
                    if let Ok(Value::Integer(value)) =
                        constant_value(&value, &Type::Integer(I::Usize), self.constants)
                    {
                        substitutions
                            .constants
                            .insert(generic.name.clone(), value.bits() as u64);
                    }
                }
                _ => (),
            }
        }
        if type_names
            .iter()
            .any(|n| !substitutions.types.contains_key(n))
            || const_names
                .iter()
                .any(|n| !substitutions.constants.contains_key(n))
        {
            return Err(error(
                span,
                "E0371",
                "generic arguments cannot be inferred; supply them explicitly",
            ));
        }
        for ty in substitutions.types.values() {
            validate_complexity(ty, span)?;
        }
        for generic in &template.generics {
            if let a::GenericKind::Type { constraints, .. } = &generic.kind {
                for constraint in constraints {
                    self.nominals
                        .interfaces
                        .require(
                            &substitutions.types[&generic.name],
                            constraint,
                            &substitutions,
                            self.constants,
                            self.nominals,
                        )
                        .map_err(|mut diagnostic| {
                            diagnostic.span = span;
                            diagnostic
                                .with_note(constraint.span, "required by this generic constraint")
                        })?;
                }
            }
        }
        Ok((substitutions, checked))
    }
}
