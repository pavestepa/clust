use super::*;

impl Checker<'_> {
    pub(in super::super) fn inherited_field_path(
        &self,
        ty: &Type,
        name: &str,
    ) -> Option<Vec<usize>> {
        let mut ty = ty.clone();
        let mut prefix = Vec::new();
        let mut nearest = None;
        loop {
            let fields = self.nominals.fields(&ty)?;
            if let Some(index) = fields.iter().position(|field| field.name == name) {
                let mut path = prefix.clone();
                path.push(index);
                if let Type::Class { name: owner, .. } | Type::Record { name: owner, .. } = &ty {
                    if self.member_owner.as_ref() == Some(&self.nominals.origin(owner)) {
                        return Some(path);
                    }
                }
                if nearest.is_none() {
                    nearest = Some(path);
                }
            }
            let Some(index) = fields.iter().position(|field| field.name == BASE) else {
                return nearest;
            };
            ty = fields[index].ty.clone();
            prefix.push(index);
        }
    }
}
