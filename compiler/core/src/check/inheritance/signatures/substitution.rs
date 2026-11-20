use super::*;

impl Bindings {
    pub(in super::super) fn parent(
        generics: &[a::Generic],
        arguments: &[a::TypeArgument],
        span: Span,
        constants: &crate::constants::Constants,
    ) -> Result<Self> {
        if arguments.len() > generics.len() {
            return Err(error(span, "E0370", "too many base class arguments"));
        }
        let module = Self::module_constants(generics, constants);
        let mut result = Self::default();
        for (i, generic) in generics.iter().enumerate() {
            match &generic.kind {
                a::GenericKind::Type { default, .. } => {
                    let ty = match arguments.get(i) {
                        Some(a::TypeArgument::Type(ty)) => ty.clone(),
                        Some(_) => {
                            return Err(error(span, "E0370", "expected base class type argument"))
                        }
                        None => {
                            let mut ty = default.clone().ok_or_else(|| {
                                error(span, "E0371", "missing base class type argument")
                            })?;
                            module.ty(&mut ty);
                            result.ty(&mut ty);
                            ty
                        }
                    };
                    result.types.insert(generic.name.clone(), ty);
                }
                a::GenericKind::Const { default, .. } => {
                    let value = match arguments.get(i) {
                        Some(a::TypeArgument::Const(value)) => value.clone(),
                        Some(a::TypeArgument::Type(a::Type {
                            kind: a::TypeKind::Named { path, arguments },
                            span,
                        })) if path.len() == 1 && arguments.is_empty() => a::Expr {
                            kind: a::ExprKind::Name(path[0].clone()),
                            span: *span,
                        },
                        Some(_) => {
                            return Err(error(span, "E0370", "expected base class const argument"))
                        }
                        None => {
                            let mut value = default.clone().ok_or_else(|| {
                                error(span, "E0371", "missing base class const argument")
                            })?;
                            module.expr(&mut value);
                            result.expr(&mut value);
                            value
                        }
                    };
                    result.constants.insert(generic.name.clone(), value);
                }
            }
        }
        Ok(result)
    }
}

impl Bindings {
    pub(super) fn argument(&self, argument: &mut a::TypeArgument) {
        if let a::TypeArgument::Type(a::Type {
            kind: a::TypeKind::Named { path, arguments },
            ..
        }) = argument
        {
            if path.len() == 1 && arguments.is_empty() {
                if let Some(value) = self.constants.get(&path[0]) {
                    *argument = a::TypeArgument::Const(value.clone());
                    return;
                }
            }
        }
        match argument {
            a::TypeArgument::Type(ty) => self.ty(ty),
            a::TypeArgument::Const(value) => self.expr(value),
        }
    }
}

impl Bindings {
    pub(in super::super) fn ty(&self, ty: &mut a::Type) {
        match &mut ty.kind {
            a::TypeKind::Named { path, arguments } => {
                if path.len() == 1 && arguments.is_empty() {
                    if let Some(actual) = self.types.get(&path[0]) {
                        *ty = actual.clone();
                        return;
                    }
                }
                for arg in arguments {
                    self.argument(arg);
                }
            }
            a::TypeKind::Tuple(fields) => {
                for ty in fields {
                    self.ty(ty);
                }
            }
            a::TypeKind::Array { element, length } => {
                self.ty(element);
                self.expr(length);
            }
            a::TypeKind::Pointer { pointee, .. } => self.ty(pointee),
            a::TypeKind::Function {
                parameters, result, ..
            } => {
                for (_, ty) in parameters {
                    self.ty(ty);
                }
                self.ty(result);
            }
        }
    }
}

impl Bindings {
    pub(super) fn expr(&self, value: &mut a::Expr) {
        match &mut value.kind {
            a::ExprKind::Name(name) => {
                if let Some(actual) = self.constants.get(name) {
                    *value = actual.clone();
                }
            }
            a::ExprKind::Group(value)
            | a::ExprKind::Unary { value, .. }
            | a::ExprKind::Member { value, .. } => self.expr(value),
            a::ExprKind::Specialize { value, arguments } => {
                self.expr(value);
                for argument in arguments {
                    self.argument(argument);
                }
            }
            a::ExprKind::Tuple(values) | a::ExprKind::Array(values) => {
                for value in values {
                    self.expr(value);
                }
            }
            a::ExprKind::Construct { ty, fields } => {
                self.ty(ty);
                for (_, value) in fields {
                    self.expr(value);
                }
            }
            a::ExprKind::New { ty, arguments } => {
                self.ty(ty);
                for value in arguments {
                    self.expr(value);
                }
            }
            a::ExprKind::Call {
                callee,
                types,
                arguments,
            } => {
                self.expr(callee);
                for ty in types {
                    self.argument(ty);
                }
                for value in arguments {
                    self.expr(value);
                }
            }
            a::ExprKind::Index { value, index } => {
                self.expr(value);
                self.expr(index);
            }
            a::ExprKind::Assign { place, value, .. } => {
                self.expr(place);
                self.expr(value);
            }
            a::ExprKind::Binary { left, right, .. } => {
                self.expr(left);
                self.expr(right);
            }
            a::ExprKind::Cast { value, ty } => {
                self.expr(value);
                self.ty(ty);
            }
            a::ExprKind::If {
                condition,
                then_block,
                else_branch,
            } => {
                self.expr(condition);
                self.block(then_block);
                if let Some(value) = else_branch {
                    self.expr(value);
                }
            }
            a::ExprKind::Branch(block) => self.block(block),
            a::ExprKind::While { condition, body } => {
                self.expr(condition);
                self.block(body);
            }
            a::ExprKind::Match { value, arms } => {
                self.expr(value);
                for arm in arms {
                    let mut local = self.clone();
                    local.hide_pattern(&arm.pattern);
                    if let Some(guard) = &mut arm.guard {
                        local.expr(guard);
                    }
                    local.expr(&mut arm.value);
                }
            }
            _ => {}
        }
    }
}

impl Bindings {
    pub(super) fn block(&self, block: &mut a::Block) {
        let mut local = self.clone();
        for statement in &mut block.statements {
            match &mut statement.kind {
                a::StatementKind::Binding {
                    pattern, ty, value, ..
                } => {
                    if let Some(ty) = ty {
                        local.ty(ty);
                    }
                    local.expr(value);
                    local.hide_pattern(pattern);
                }
                a::StatementKind::Expression(value) | a::StatementKind::Return(Some(value)) => {
                    local.expr(value)
                }
                a::StatementKind::Unsafe(body) => local.block(body),
                _ => {}
            }
        }
    }
}
