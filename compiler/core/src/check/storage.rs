//! Static readonly aggregate components. Descriptor storage is never ownership
//! of its target, even if the enclosing aggregate is moved or copied.
use super::*;
pub(super) fn read(value: h::Expr) -> h::Expr {
    if let Type::StoredView(target) = &value.ty {
        h::Expr {
            ty: (**target).clone(),
            span: value.span,
            kind: h::ExprKind::StoredView(Box::new(value)),
        }
    } else {
        value
    }
}
fn callable_mode(ty: &Type) -> bool {
    match ty {
        Type::Function {
            contract: Some(_), ..
        } => true,
        Type::Record { fields, .. }
        | Type::Class {
            storage: Some(fields),
            ..
        } => fields.iter().any(|f| callable_mode(&f.ty)),
        Type::Tuple(fields) => fields.iter().any(callable_mode),
        Type::Array(t, _) | Type::StoredView(t) => callable_mode(t),
        Type::Enum { variants, .. } => variants.iter().flat_map(|v| &v.fields).any(callable_mode),
        _ => false,
    }
}

pub(super) type Modes = BTreeMap<(usize, usize, usize), Type>;

// Readonly descriptors dominate owning components in a static representation.
// This chooses a type, never a runtime tag or an implicit owner allocation.

#[derive(Clone)]
enum Step {
    Field(usize),
    Index,
    Variant(usize),
}

mod arrays;
mod assignment;
mod inference;
mod modes;
mod paths;
pub(super) use self::modes::same_mode;
pub(super) use self::paths::key;
use self::paths::place_paths;
pub(super) use self::paths::union;
