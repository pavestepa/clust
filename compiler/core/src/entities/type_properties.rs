use super::*;
impl Type {
    pub fn copyable(&self) -> bool {
        match self {
            Self::Class { copy, .. } => *copy,
            Self::Record { copy, fields, .. } => *copy && fields.iter().all(|f| f.ty.copyable()),
            Self::Enum { copy, variants, .. } => {
                *copy && variants.iter().flat_map(|v| &v.fields).all(Type::copyable)
            }
            Self::Tuple(fields) => fields.iter().all(Type::copyable),
            Self::Array(element, _) => element.copyable(),
            _ => true,
        }
    }

    /// Copy classes still own allocations. MIR value operands transfer those
    /// temporary owners; only an explicit Copy operation duplicates identity.
    pub fn owns_storage(&self) -> bool {
        !self.copyable()
            || match self {
                Self::Class { .. } => true,
                Self::Record { fields, .. } => fields.iter().any(|f| f.ty.owns_storage()),
                Self::Enum { variants, .. } => variants
                    .iter()
                    .flat_map(|v| &v.fields)
                    .any(Self::owns_storage),
                Self::Tuple(fields) => fields.iter().any(Self::owns_storage),
                Self::Array(element, count) => *count != 0 && element.owns_storage(),
                _ => false,
            }
    }

    pub fn contains_views(&self) -> bool {
        match self {
            Self::StoredView(_) => true,
            Self::Class {
                storage: Some(fields),
                ..
            } => fields.iter().any(|f| f.ty.contains_views()),
            Self::Record { fields, .. } => fields.iter().any(|f| f.ty.contains_views()),
            Self::Tuple(fields) => fields.iter().any(Self::contains_views),
            Self::Array(element, _) => element.contains_views(),
            Self::Enum { variants, .. } => variants
                .iter()
                .flat_map(|v| &v.fields)
                .any(Self::contains_views),
            _ => false,
        }
    }
    /// Source annotations name the logical type; physical storage modes remain
    /// part of the concrete HIR type and cannot change on assignment/call.
    pub fn logical(&self) -> Self {
        let mut ty = self.clone();
        match &mut ty {
            Self::StoredView(target) => return target.logical(),
            Self::Class { storage, .. } => *storage = None,
            Self::Function {
                parameters,
                result,
                contract,
                ..
            } => {
                *contract = None;
                for (_, ty) in parameters {
                    *ty = ty.logical()
                }
                **result = result.logical();
            }
            Self::Record { fields, .. } => {
                for f in fields {
                    f.ty = f.ty.logical();
                }
            }
            Self::Tuple(fields) => {
                for t in fields {
                    *t = t.logical();
                }
            }
            Self::Array(t, _) => **t = t.logical(),
            Self::Enum { variants, .. } => {
                for v in variants {
                    for t in &mut v.fields {
                        *t = t.logical();
                    }
                }
            }
            _ => (),
        }
        ty
    }
    pub fn class_key(&self) -> String {
        let Self::Class { name, storage, .. } = self else {
            panic!("class key requires class")
        };
        if storage.is_some() {
            format!("{name}${storage:?}")
        } else {
            name.clone()
        }
    }
}
