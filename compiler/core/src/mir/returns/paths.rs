use super::*;

pub(in super::super) fn concrete_paths(
    ty: &Type,
    path: &[Projection],
    classes: &h::Classes,
) -> Vec<Vec<Projection>> {
    let Some((head, tail)) = path.split_first() else {
        return vec![Vec::new()];
    };
    let choices = match (head, ty) {
        (Projection::Index(_), Type::Array(element, count)) => (0..*count)
            .map(|i| (Projection::ConstantIndex(i), (**element).clone()))
            .collect(),
        (Projection::ConstantIndex(i), Type::Array(element, _)) => {
            vec![(Projection::ConstantIndex(*i), (**element).clone())]
        }
        (Projection::Field(i), Type::Record { fields, .. }) => {
            vec![(head.clone(), fields[*i].ty.clone())]
        }
        (Projection::Field(i), Type::Class { .. }) => {
            vec![(head.clone(), classes[&ty.class_key()].fields[*i].ty.clone())]
        }
        (Projection::Field(i), Type::Tuple(fields)) => vec![(head.clone(), fields[*i].clone())],
        (Projection::Variant(i), Type::Enum { variants, .. }) => {
            vec![(head.clone(), Type::Tuple(variants[*i].fields.clone()))]
        }
        (Projection::Dereference, Type::StoredView(target)) => {
            vec![(head.clone(), (**target).clone())]
        }
        _ => return Vec::new(),
    };
    choices
        .into_iter()
        .flat_map(|(head, ty)| {
            concrete_paths(&ty, tail, classes)
                .into_iter()
                .map(move |mut path| {
                    path.insert(0, head.clone());
                    path
                })
        })
        .collect()
}

impl Fact {
    pub(super) fn owned() -> Self {
        Self {
            owned: true,
            ..Self::default()
        }
    }
}

impl Fact {
    pub(super) fn merge(&mut self, other: &Self) {
        self.owned |= other.owned;
        self.readonly |= other.readonly;
        self.origins.extend(other.origins.iter().cloned());
        self.normalize();
    }
}

impl Fact {
    pub(super) fn normalize(&mut self) {
        // Origins are dependencies, not runtime addresses. A containing place
        // subsumes its descendants; widen large recursive path sets to the root.
        let mut counts = BTreeMap::<usize, usize>::new();
        for p in &self.origins {
            *counts.entry(p.local).or_default() += 1;
        }
        let origins = self
            .origins
            .iter()
            .map(|p| {
                let mut p = p.clone();
                if p.projections.len() > 32 || counts[&p.local] > 64 {
                    p.projections.clear();
                }
                if let Some(i) = p
                    .projections
                    .iter()
                    .position(|p| matches!(p, Projection::Index(_)))
                {
                    p.projections.truncate(i);
                }
                p
            })
            .collect::<BTreeSet<_>>();
        self.origins = origins
            .iter()
            .filter(|p| {
                !origins.iter().any(|parent| {
                    parent.local == p.local
                        && parent.projections.len() < p.projections.len()
                        && p.projections.starts_with(&parent.projections)
                })
            })
            .cloned()
            .collect();
    }
}

impl Fact {
    pub(super) fn project(mut self, path: &[Projection]) -> Self {
        self.origins = self
            .origins
            .into_iter()
            .map(|mut p| {
                // Dynamic callee indices do not escape into caller local numbering.
                // The containing place is a conservative lifetime dependency.
                let end = path
                    .iter()
                    .position(|p| matches!(p, Projection::Index(_)))
                    .unwrap_or(path.len());
                p.projections.extend_from_slice(&path[..end]);
                if p.projections.len() > 32 {
                    p.projections.clear();
                }
                p
            })
            .collect();
        self.normalize();
        self
    }
}
