use super::*;

impl Nominals {
    pub(super) fn build(
        &self,
        item: &a::Item,
        name: &str,
        substitution: &mut Substitution,
        constants: &crate::constants::Constants,
    ) -> Result<Type> {
        let mut resolve = |ty: &a::Type| {
            let mut ty = ty.clone();
            substitution.ty(&mut ty);
            resolve_type_with(&ty, constants, self)
        };
        match &item.kind {
            a::ItemKind::Aggregate {
                class,
                interfaces,
                members,
                ..
            } => {
                for interface in interfaces {
                    if !matches!(&interface.kind,a::TypeKind::Named {path,..} if self.interfaces.contains(&path.join("::")))
                    {
                        return Err(error(interface.span,"E0900","embedded base construction and inherited method lowering require the remaining C4.2 work"));
                    }
                }
                if *class {
                    super::super::constructors::validate(members)?;
                }
                let mut fields = Vec::new();
                for member in members {
                    if *class && !matches!(member, a::Member::Field { .. }) {
                        continue;
                    }
                    let a::Member::Field {
                        name,
                        ty,
                        private,
                        weak: false,
                        span,
                    } = member
                    else {
                        return Err(error(
                            item.span,
                            "E0900",
                            "struct methods and destructors require ownership checking",
                        ));
                    };
                    if fields.iter().any(|f: &h::RecordField| f.name == *name) {
                        return Err(error(*span, "E0301", "duplicate field"));
                    }
                    fields.push(h::RecordField {
                        name: name.clone(),
                        ty: resolve(ty)?,
                        private: *private,
                        source: span.source,
                    });
                }
                let copy = item.attributes.contains(&a::Attribute::Copy);
                if *class {
                    if copy && fields.iter().any(|f| !f.ty.copyable()) {
                        return Err(error(item.span, "E0373", "Copy class requires Copy fields"));
                    }
                    // Validate the payload separately: the handle itself always
                    // has pointer layout, even when a payload field is recursive.
                    crate::layout::layout(&Type::Tuple(
                        fields.iter().map(|f| f.ty.clone()).collect(),
                    ))
                    .map_err(|message| error(item.span, "E0318", message))?;
                    self.classes
                        .borrow_mut()
                        .insert(name.into(), h::ClassDefinition { copy, fields });
                    return Ok(Type::Class {
                        name: name.into(),
                        copy,
                        storage: None,
                    });
                }
                Ok(Type::Record {
                    copy,
                    name: name.into(),
                    fields,
                })
            }
            a::ItemKind::Enum { variants, .. } => {
                let mut checked = Vec::new();
                let mut names = BTreeSet::new();
                for variant in variants {
                    if !names.insert(&variant.name) {
                        return Err(error(variant.span, "E0301", "duplicate enum variant"));
                    }
                    let (types, shape): (Vec<&a::Type>, _) = match &variant.fields {
                        a::VariantFields::Unit => (vec![], h::VariantShape::Unit),
                        a::VariantFields::Tuple(fields) => {
                            (fields.iter().collect(), h::VariantShape::Tuple)
                        }
                        a::VariantFields::Named(fields) => {
                            let mut names = BTreeSet::new();
                            for (name, _) in fields {
                                if !names.insert(name) {
                                    return Err(error(
                                        variant.span,
                                        "E0301",
                                        "duplicate variant field",
                                    ));
                                }
                            }
                            (
                                fields.iter().map(|(_, ty)| ty).collect(),
                                h::VariantShape::Named(
                                    fields.iter().map(|(name, _)| name.clone()).collect(),
                                ),
                            )
                        }
                    };
                    let fields = types
                        .into_iter()
                        .map(&mut resolve)
                        .collect::<Result<Vec<_>>>()?;
                    checked.push(h::Variant {
                        name: variant.name.clone(),
                        fields,
                        shape,
                    });
                }
                Ok(Type::Enum {
                    copy: item.attributes.contains(&a::Attribute::Copy),
                    name: name.into(),
                    variants: checked,
                })
            }
            _ => unreachable!(),
        }
    }
}
