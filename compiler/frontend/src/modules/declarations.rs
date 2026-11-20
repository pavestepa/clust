use super::*;

impl Rewrite<'_> {
    pub(super) fn declaration(&self, name: &mut String) {
        if let Some(symbol) = self.globals.get(name) {
            *name = symbol.canonical.clone();
        }
    }
}

impl Rewrite<'_> {
    pub(super) fn generics(&mut self, generics: &mut [Generic]) {
        self.scopes
            .last_mut()
            .unwrap()
            .extend(generics.iter().map(|g| g.name.clone()));
        for generic in generics {
            match &mut generic.kind {
                GenericKind::Type {
                    constraints,
                    default,
                } => {
                    for constraint in constraints {
                        self.ty(constraint);
                    }
                    if let Some(default) = default {
                        self.ty(default);
                    }
                }
                GenericKind::Const { ty, default } => {
                    self.ty(ty);
                    if let Some(default) = default {
                        self.expr(default);
                    }
                }
            }
            self.scopes.last_mut().unwrap().insert(generic.name.clone());
        }
    }
}

impl Rewrite<'_> {
    pub(super) fn item(&mut self, item: &mut Item) {
        self.scopes.push(BTreeSet::new());
        match &mut item.kind {
            ItemKind::Function(function) => {
                self.declaration(&mut function.name);
                self.function(function);
            }
            ItemKind::Const { name, ty, value } => {
                self.declaration(name);
                self.ty(ty);
                self.expr(value);
            }
            ItemKind::Aggregate {
                name,
                generics,
                interfaces,
                members,
                ..
            } => {
                self.declaration(name);
                self.generics(generics);
                self.scopes.last_mut().unwrap().insert("super".into());
                for interface in interfaces {
                    self.ty(interface);
                }
                let member_names: BTreeSet<String> = members
                    .iter()
                    .filter_map(|member| match member {
                        Member::Field { name, .. } | Member::Const { name, .. } => {
                            Some(name.clone())
                        }
                        Member::Method { function, .. } => Some(function.name.clone()),
                        _ => None,
                    })
                    .collect();
                for member in members {
                    match member {
                        Member::Field { ty, .. } => self.ty(ty),
                        Member::Method { function, .. } => {
                            let instance = function.parameters.iter().any(|p| p.receiver);
                            if instance {
                                self.scopes.push(member_names.clone());
                            }
                            self.function(function);
                            if instance {
                                self.scopes.pop();
                            }
                        }
                        Member::Constructor {
                            parameters, body, ..
                        } => {
                            self.scopes.push(BTreeSet::from(["this".into()]));
                            self.parameters(parameters);
                            self.block(body);
                            self.scopes.pop();
                        }
                        Member::Destructor(body) => {
                            self.scopes.push(BTreeSet::from(["this".into()]));
                            self.block(body);
                            self.scopes.pop();
                        }
                        Member::Const { ty, value, .. } => {
                            self.ty(ty);
                            self.expr(value);
                        }
                    }
                }
            }
            ItemKind::Enum {
                name,
                generics,
                variants,
            } => {
                self.declaration(name);
                self.generics(generics);
                for variant in variants {
                    match &mut variant.fields {
                        VariantFields::Unit => {}
                        VariantFields::Tuple(fields) => {
                            for ty in fields {
                                self.ty(ty);
                            }
                        }
                        VariantFields::Named(fields) => {
                            for (_, ty) in fields {
                                self.ty(ty);
                            }
                        }
                    }
                }
            }
            ItemKind::Interface {
                name,
                generics,
                methods,
            } => {
                self.declaration(name);
                self.generics(generics);
                for method in methods {
                    self.function(method);
                }
            }
            ItemKind::Extern(functions) => {
                for function in functions {
                    self.function(function);
                }
            }
            ItemKind::Use(_) | ItemKind::Module { .. } => {}
        }
        self.scopes.pop();
    }
}

impl Rewrite<'_> {
    pub(super) fn function(&mut self, function: &mut Function) {
        self.scopes.push(BTreeSet::new());
        self.generics(&mut function.generics);
        self.parameters(&mut function.parameters);
        self.ty(&mut function.result);
        if let Some(body) = &mut function.body {
            self.block(body);
        }
        self.scopes.pop();
    }
}

impl Rewrite<'_> {
    pub(super) fn parameters(&mut self, parameters: &mut [Parameter]) {
        for parameter in parameters {
            if let Some(ty) = &mut parameter.ty {
                self.ty(ty);
            }
            self.pattern(&mut parameter.pattern);
        }
    }
}
