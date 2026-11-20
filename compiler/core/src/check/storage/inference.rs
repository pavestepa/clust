use super::*;

pub(super) fn promote(
    ty: &mut Type,
    path: &[Step],
    requested: &Type,
    span: Span,
    nominals: &records::Nominals,
) -> Result<bool> {
    let Some((head, tail)) = path.split_first() else {
        *ty = union(ty, requested, span)?;
        return Ok(true);
    };
    if let Type::Class { .. } = ty {
        let Some(mut fields) = nominals.fields(ty) else {
            return Ok(false);
        };
        let Step::Field(i) = head else {
            return Ok(false);
        };
        if !promote(&mut fields[*i].ty, tail, requested, span, nominals)? {
            return Ok(false);
        }
        let Type::Class { storage, .. } = ty else {
            unreachable!()
        };
        *storage = Some(fields);
        return Ok(true);
    }
    match (head, ty) {
        (Step::Field(i), Type::Record { fields, .. }) => {
            promote(&mut fields[*i].ty, tail, requested, span, nominals)
        }
        (Step::Field(i), Type::Tuple(fields)) => {
            promote(&mut fields[*i], tail, requested, span, nominals)
        }
        (Step::Index, Type::Array(element, _)) => promote(element, tail, requested, span, nominals),
        (Step::Variant(i), Type::Enum { variants, .. }) => {
            let Some((Step::Field(field), tail)) = tail.split_first() else {
                return Ok(false);
            };
            promote(
                &mut variants[*i].fields[*field],
                tail,
                requested,
                span,
                nominals,
            )
        }
        _ => Ok(false),
    }
}

impl Checker<'_> {
    pub(in super::super) fn infers_storage_place(&self, place: &h::Expr) -> bool {
        let paths = place_paths(self, place, &mut Vec::new());
        !paths.is_empty()
            && paths.iter().all(|(root, path)| {
                promote(
                    &mut self.locals[*root].ty.clone(),
                    path,
                    &place.ty,
                    place.span,
                    self.nominals,
                )
                .unwrap_or(false)
            })
    }
}

impl Checker<'_> {
    pub(in super::super) fn request_storage_mode(
        &mut self,
        place: &h::Expr,
        requested: &Type,
    ) -> Result<bool> {
        if place.ty == *requested {
            return Ok(false);
        }
        let paths = place_paths(self, place, &mut Vec::new());
        if paths.is_empty() {
            if self.request_result_mode(place, requested)? {
                return Ok(true);
            }
            if matches!(
                place.kind,
                h::ExprKind::Block(_)
                    | h::ExprKind::If { .. }
                    | h::ExprKind::Tuple(_)
                    | h::ExprKind::Array(_)
                    | h::ExprKind::Match(_)
                    | h::ExprKind::New { .. }
            ) {
                return self.request_expression_mode(place.span, &place.ty, requested);
            }
            return Ok(false);
        }
        let mut changed = false;
        for (root, path) in paths {
            let key = key(self.locals[root].span);
            let mut target = self
                .storage_requests
                .get(&key)
                .or_else(|| self.storage_hints.get(&key))
                .unwrap_or(&self.locals[root].ty)
                .clone();
            if !promote(
                match &mut target {
                    Type::StoredView(t) => t,
                    _ => &mut target,
                },
                &path,
                requested,
                place.span,
                self.nominals,
            )? || matches!(target, Type::StoredView(_))
                && !self.storage_parameters.contains(&root)
            {
                continue;
            }
            if target != self.locals[root].ty {
                self.storage_requests.insert(key, target);
                changed = true;
            }
        }
        Ok(changed)
    }
}

impl Checker<'_> {
    pub(in super::super) fn request_expression_mode(
        &mut self,
        span: Span,
        actual: &Type,
        required: &Type,
    ) -> Result<bool> {
        let key = key(span);
        let known = self
            .storage_requests
            .get(&key)
            .or_else(|| self.storage_hints.get(&key))
            .unwrap_or(actual);
        let wanted = union(known, required, span)?;
        if &wanted != known {
            self.storage_requests.insert(key, wanted);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
