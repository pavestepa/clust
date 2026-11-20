use super::*;

impl Nominals {
    pub fn resolve(
        &self,
        name: &str,
        args: &[a::TypeArgument],
        constants: &crate::constants::Constants,
        span: Span,
    ) -> Result<Type> {
        if args.is_empty() {
            if let Some(ty) = self.get(name) {
                return Ok(ty);
            }
        }
        let item = self
            .definitions
            .get(name)
            .ok_or_else(|| error(span, "E0310", format!("unknown type '{name}'")))?;
        let generics = match &item.kind {
            a::ItemKind::Aggregate { generics, .. } | a::ItemKind::Enum { generics, .. } => {
                generics
            }
            _ => unreachable!(),
        };
        if args.len() > generics.len() {
            return Err(error(span, "E0370", "too many type arguments"));
        }
        let mut substitution = Substitution::default();
        let mut names = BTreeSet::new();
        for (index, generic) in generics.iter().enumerate() {
            if !names.insert(&generic.name) {
                return Err(error(generic.span, "E0301", "duplicate generic parameter"));
            }
            match &generic.kind {
                a::GenericKind::Type {
                    constraints: _,
                    default,
                } => {
                    let mut ty = match args.get(index) {
                        Some(a::TypeArgument::Type(ty)) => ty.clone(),
                        None => default
                            .clone()
                            .ok_or_else(|| error(span, "E0371", "missing generic type argument"))?,
                        _ => return Err(error(span, "E0370", "expected type argument")),
                    };
                    substitution.ty(&mut ty);
                    let ty = resolve_type_with(&ty, constants, self)?;
                    substitution.types.insert(generic.name.clone(), ty);
                }
                a::GenericKind::Const { default, .. } => {
                    let mut value = match args.get(index) {
                        Some(a::TypeArgument::Const(value)) => value.clone(),
                        Some(a::TypeArgument::Type(a::Type {
                            kind: a::TypeKind::Named { path, arguments },
                            ..
                        })) if path.len() == 1 && arguments.is_empty() => a::Expr {
                            kind: a::ExprKind::Name(path[0].clone()),
                            span,
                        },
                        None => default.clone().ok_or_else(|| {
                            error(span, "E0371", "missing const generic argument")
                        })?,
                        _ => return Err(error(span, "E0370", "expected const argument")),
                    };
                    substitution.expr(&mut value);
                    let Value::Integer(value) =
                        constant_value(&value, &Type::Integer(I::Usize), constants)?
                    else {
                        unreachable!()
                    };
                    substitution
                        .constants
                        .insert(generic.name.clone(), value.bits() as u64);
                }
            }
        }
        for generic in generics {
            if let a::GenericKind::Type { constraints, .. } = &generic.kind {
                for constraint in constraints {
                    self.interfaces.require(
                        &substitution.types[&generic.name],
                        constraint,
                        &substitution,
                        constants,
                        self,
                    )?;
                }
            }
        }
        let key = if generics.is_empty() {
            name.to_owned()
        } else {
            format!(
                "{name}<{:?};{:?}>",
                substitution.types, substitution.constants
            )
        };
        if let Some(ty) = self.get(&key) {
            return Ok(ty);
        }
        let is_class = matches!(item.kind, a::ItemKind::Aggregate { class: true, .. });
        if is_class {
            // Register the identity handle before resolving its payload. A class
            // edge breaks inline layout recursion without leaking Rc cycles.
            self.instances.borrow_mut().insert(
                key.clone(),
                Type::Class {
                    name: key.clone(),
                    storage: None,
                    copy: item.attributes.contains(&a::Attribute::Copy),
                },
            );
        }
        let defer_class = is_class && !self.active.borrow().is_empty();
        let depth = self.active.borrow().len() + self.class_depth.get();
        if depth >= 32 {
            return Err(error(
                span,
                "E0350",
                "recursive inline type or type nesting limit",
            ));
        }
        let result = if defer_class {
            // Class handles break layout recursion even when resolution started
            // from an inline enum/struct. Complete this payload after that inline
            // layout is available, independent of declaration order.
            self.pending_classes.borrow_mut().push((
                item.clone(),
                key.clone(),
                substitution.clone(),
                depth,
            ));
            Ok(self.get(&key).unwrap())
        } else {
            {
                let mut active = self.active.borrow_mut();
                if !active.insert(key.clone()) {
                    return Err(error(
                        span,
                        "E0350",
                        "recursive inline type or type nesting limit",
                    ));
                }
            }
            let result = self.build(item, &key, &mut substitution, constants);
            self.active.borrow_mut().remove(&key);
            result
        };
        let ty = match result {
            Ok(ty) => ty,
            Err(error) => {
                if is_class {
                    self.instances.borrow_mut().remove(&key);
                }
                return Err(error);
            }
        };
        if item.attributes.contains(&a::Attribute::Copy) && !self.copyable(&ty) {
            return Err(error(span, "E0373", "Copy aggregate requires Copy fields"));
        }
        crate::layout::layout(&ty).map_err(|message| error(span, "E0318", message))?;
        let arguments = generics
            .iter()
            .map(|generic| match generic.kind {
                a::GenericKind::Type { .. } => {
                    Argument::Type(substitution.types[&generic.name].clone())
                }
                a::GenericKind::Const { .. } => {
                    Argument::Const(substitution.constants[&generic.name])
                }
            })
            .collect();
        self.arguments.borrow_mut().insert(key.clone(), arguments);
        self.origins
            .borrow_mut()
            .insert(key.clone(), name.to_owned());
        self.instances.borrow_mut().insert(key, ty.clone());
        if let a::ItemKind::Aggregate { interfaces, .. } = &item.kind {
            self.interfaces.validate_implementation(
                &ty,
                interfaces,
                &substitution,
                constants,
                self,
            )?;
        }
        if self.active.borrow().is_empty() {
            loop {
                let pending = self.pending_classes.borrow_mut().pop();
                let Some((item, key, mut substitution, depth)) = pending else {
                    break;
                };
                self.active.borrow_mut().insert(key.clone());
                let previous = self.class_depth.replace(depth);
                let result = self.build(&item, &key, &mut substitution, constants);
                self.class_depth.set(previous);
                self.active.borrow_mut().remove(&key);
                result?;
            }
        }
        Ok(ty)
    }
}
