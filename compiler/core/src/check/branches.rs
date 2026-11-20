use super::*;

impl Checker<'_> {
    pub(super) fn branch(
        &mut self,
        branch: &a::Expr,
        expected: Option<&Type>,
        value_context: bool,
    ) -> Result<h::Block> {
        if let a::ExprKind::Branch(block) = &branch.kind {
            return self.block(block, expected, value_context);
        }
        let expression = self.expr(branch, expected, value_context)?;
        let result = expression.ty.clone();
        let diverges = expression_diverges(&expression);
        Ok(h::Block {
            construction: vec![],
            storage: vec![],
            statements: vec![h::Statement::Evaluate(expression)],
            result,
            diverges,
            span: branch.span,
        })
    }
}

impl Checker<'_> {
    pub(super) fn known_type(&self, value: &a::Expr) -> Option<Type> {
        match &value.kind {
            a::ExprKind::Name(name) => self
                .lookup(name)
                .map(|id| self.locals[id].ty.clone())
                .or_else(|| self.constants.get(name).map(|(_, ty)| ty.clone())),
            a::ExprKind::Group(value) => self.known_type(value),
            a::ExprKind::Unary { operator, value } => {
                let ty = self.known_type(value)?;
                match *operator {
                    "&" | "&var" => Some(Type::Pointer {
                        mutable: *operator == "&var",
                        pointee: Box::new(ty),
                    }),
                    "*" if {
                        let mut source = &**value;
                        while let a::ExprKind::Group(inner) = &source.kind {
                            source = inner;
                        }
                        matches!(&source.kind, a::ExprKind::Name(name) if self.lookup(name).is_some_and(|id| self.locals[id].view.is_some()))
                    } =>
                    {
                        Some(ty)
                    }
                    "*" => match ty {
                        Type::Pointer { pointee, .. } => Some(*pointee),
                        other => Some(other),
                    },
                    _ => Some(ty),
                }
            }
            a::ExprKind::Index { value, .. } => match self.known_type(value)? {
                Type::Array(element, _) => Some(*element),
                _ => None,
            },
            a::ExprKind::Member { value, name } => match self.known_type(value)? {
                Type::Tuple(fields) => fields.get(name.parse::<usize>().ok()?).cloned(),
                ty => self
                    .nominals
                    .fields(&ty)?
                    .iter()
                    .find(|f| f.name == *name)
                    .map(|f| f.ty.clone()),
            },
            a::ExprKind::Cast { ty, .. } => {
                resolve_type_with(ty, self.constants, self.nominals).ok()
            }
            a::ExprKind::Call { callee, .. } => {
                if let a::ExprKind::Name(name) = &callee.kind {
                    self.signatures.get(name).map(|s| s.result.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
