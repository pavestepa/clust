use super::*;
#[derive(Clone, Default)]
pub(super) struct Bindings {
    types: BTreeMap<String, a::Type>,
    constants: BTreeMap<String, a::Expr>,
}
impl Bindings {
    fn module_constants(generics: &[a::Generic], constants: &crate::constants::Constants) -> Self {
        let mut bindings = Self::default();
        for (name, (value, ty)) in constants {
            if generics.iter().any(|g| &g.name == name) {
                continue;
            }
            if *ty == Type::Integer(I::Usize) {
                let Value::Integer(value) = value else {
                    unreachable!()
                };
                bindings.constants.insert(
                    name.clone(),
                    a::Expr {
                        kind: a::ExprKind::Literal(a::Literal::Integer(value.bits().to_string())),
                        span: generics.first().map(|g| g.span).unwrap_or(Span {
                            source: clust_frontend::source::SourceId(0),
                            start: 0,
                            end: 0,
                        }),
                    },
                );
            }
        }
        bindings
    }
}
struct Keys<'a> {
    constants: &'a crate::constants::Constants,
}
// Resolve lexical module constants before crossing an inheritance edge. A
// derived class's generic N must not capture a free module constant N in A.

mod compatibility;
mod constants;
mod instantiation;
mod shadowing;
mod substitution;
pub(super) use self::compatibility::compatible;
pub(super) use self::constants::freeze_constants;
