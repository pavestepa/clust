//! Conservative place-sensitive future reads before definite replacement.
use super::*;
#[derive(Clone, PartialEq, Eq)]
enum Projection {
    Field(String),
    Index(Option<u128>),
}
#[derive(Clone)]
struct Place {
    name: String,
    path: Vec<Projection>,
}
fn as_place(value: &a::Expr) -> Option<Place> {
    match &value.kind {
        a::ExprKind::Name(name) => Some(Place {
            name: name.clone(),
            path: vec![],
        }),
        a::ExprKind::Group(value) => as_place(value),
        a::ExprKind::Member { value, name } => {
            let mut p = as_place(value)?;
            p.path.push(Projection::Field(name.clone()));
            Some(p)
        }
        a::ExprKind::Index { value, index } => {
            let mut p = as_place(value)?;
            let n = match &index.kind {
                a::ExprKind::Literal(a::Literal::Integer(n)) => magnitude(n, index.span).ok(),
                _ => None,
            };
            p.path.push(Projection::Index(n));
            Some(p)
        }
        _ => None,
    }
}
fn overlaps(a: &Place, b: &Place) -> bool {
    a.name == b.name
        && a.path.iter().zip(&b.path).all(|(a, b)| match (a, b) {
            (Projection::Field(a), Projection::Field(b)) => a == b,
            (Projection::Index(Some(a)), Projection::Index(Some(b))) => a == b,
            _ => true,
        })
}
fn covers(a: &Place, b: &Place) -> bool {
    a.name == b.name
        && a.path.len() <= b.path.len()
        && a.path
            .iter()
            .zip(&b.path)
            .all(|(a, b)| a == b && !matches!(a, Projection::Index(None)))
}
#[derive(Clone, Copy)]
struct Flow {
    live: bool,
    used: bool,
    stopped: bool,
}
impl Flow {
    fn join(self, other: Self) -> Self {
        Self {
            live: self.live && !self.stopped || other.live && !other.stopped,
            used: self.used || other.used,
            stopped: self.stopped && other.stopped,
        }
    }
}

mod addresses;
mod queries;
mod reads;
