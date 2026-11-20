use super::*;

pub(in super::super) fn collect(
    module: &a::Module,
    constants: &crate::constants::Constants,
) -> Result<Nominals> {
    let mut nominals = Nominals {
        interfaces: interfaces::Registry::collect(module)?,
        ..Default::default()
    };
    let mut roots = Vec::new();
    for item in &module.items {
        if let a::ItemKind::Aggregate { name, generics, .. }
        | a::ItemKind::Enum { name, generics, .. } = &item.kind
        {
            if nominals
                .definitions
                .insert(name.clone(), item.clone())
                .is_some()
            {
                return Err(error(item.span, "E0301", "duplicate type declaration"));
            }
            if generics.is_empty() {
                roots.push((name.clone(), item.span));
            }
        }
    }
    for (name, span) in roots {
        nominals.resolve(&name, &[], constants, span)?;
    }
    nominals
        .interfaces
        .validate_declarations(constants, &nominals)?;
    Ok(nominals)
}

impl Checker<'_> {
    pub(in super::super) fn record(
        &mut self,
        ty: &a::Type,
        initializers: &[(String, a::Expr)],
        expected: Option<&Type>,
        span: Span,
    ) -> Result<h::Expr> {
        if let Some(value) = self.enum_named(ty, initializers, expected, span)? {
            return Ok(value);
        }
        let contextual = match (&ty.kind, expected) {
            (a::TypeKind::Named { path, arguments }, Some(Type::Record { name, .. }))
                if arguments.is_empty() && path == &[self.nominals.origin(name)] =>
            {
                expected.cloned()
            }
            _ => None,
        };
        let ty = if let Some(ty) = contextual {
            ty
        } else {
            resolve_type_with(ty, self.constants, self.nominals)?
        };
        if let Some(expected) = expected {
            same(expected, &ty, span)?;
        }
        let mut ty = expected
            .filter(|expected| {
                matches!(expected, Type::Record { .. }) && expected.logical() == ty.logical()
            })
            .cloned()
            .unwrap_or(ty);
        let owner = ty.clone();
        let Type::Record { fields, .. } = &mut ty else {
            return Err(error(span, "E0351", "named construction requires a struct"));
        };
        let mut values = vec![None; fields.len()];
        let mut statements = Vec::new();
        // Evaluate written order, then assemble in declaration order.
        for (name, source) in initializers {
            let index = fields
                .iter()
                .position(|f| f.name == *name)
                .ok_or_else(|| error(source.span, "E0351", format!("unknown field '{name}'")))?;
            let field = &mut fields[index];
            self.check_field_access(&owner, field, source.span)?;
            if values[index].is_some() {
                return Err(error(
                    source.span,
                    "E0351",
                    "field initialized more than once",
                ));
            }
            let value = self.storage_value(source, Some(&field.ty))?;
            field.ty = value.ty.clone();
            let id = self.locals.len();
            self.locals.push(h::Local {
                view: None,
                name: format!("<field {name}>"),
                ty: field.ty.clone(),
                mutable: false,
                span: source.span,
            });
            statements.push(h::Statement::Initialize(id, value));
            values[index] = Some(h::Expr {
                kind: h::ExprKind::Local(id),
                ty: field.ty.clone(),
                span: source.span,
            });
        }
        if values.iter().any(Option::is_none) {
            return Err(error(
                span,
                "E0351",
                "every struct field must be initialized",
            ));
        }
        let construction: Vec<_> = values
            .iter()
            .map(|v| {
                let h::ExprKind::Local(id) = v.as_ref().unwrap().kind else {
                    unreachable!()
                };
                id
            })
            .collect();
        statements.push(h::Statement::Evaluate(h::Expr {
            kind: h::ExprKind::Tuple(values.into_iter().map(Option::unwrap).collect()),
            ty: ty.clone(),
            span,
        }));
        Ok(h::Expr {
            kind: h::ExprKind::Block(h::Block {
                construction: construction.clone(),
                storage: construction,
                statements,
                result: ty.clone(),
                diverges: false,
                span,
            }),
            ty,
            span,
        })
    }
}
