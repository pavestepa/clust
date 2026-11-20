use super::*;

impl Nominals {
    pub fn classes(&self) -> h::Classes {
        self.classes.borrow().clone()
    }
}

impl Nominals {
    pub fn fields(&self, ty: &Type) -> Option<Vec<h::RecordField>> {
        match ty {
            Type::Record { fields, .. } => Some(fields.clone()),
            Type::Class {
                storage: Some(fields),
                ..
            } => Some(fields.clone()),
            Type::Class { name, .. } => self.classes.borrow().get(name).map(|c| c.fields.clone()),
            _ => None,
        }
    }
}

impl Nominals {
    pub fn is_enum(&self, name: &str) -> bool {
        matches!(
            self.definitions.get(name).map(|item| &item.kind),
            Some(a::ItemKind::Enum { .. })
        ) || matches!(self.get(name), Some(Type::Enum { .. }))
    }
}

impl Nominals {
    pub fn marker(&self, name: String, copy: bool) -> Type {
        let ty = Type::Record {
            copy,
            name: name.clone(),
            fields: Vec::new(),
        };
        self.opaque.borrow_mut().insert(name.clone(), copy);
        self.instances.borrow_mut().insert(name, ty.clone());
        ty
    }
}

impl Nominals {
    pub fn copyable(&self, ty: &Type) -> bool {
        ty.copyable()
    }
}
