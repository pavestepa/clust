use super::*;
type MethodShape = (bool, Vec<(bool, bool, Type)>, Type);
pub(super) fn signature(
    f: &a::Function,
    constants: &crate::constants::Constants,
    nominals: &records::Nominals,
) -> Result<MethodShape> {
    Ok((
        f.unsafe_,
        f.parameters
            .iter()
            .map(|p| {
                Ok((
                    p.receiver,
                    p.mutable,
                    resolve_type_with(
                        p.ty.as_ref().ok_or_else(|| {
                            error(p.span, "E0400", "interface receiver must be first")
                        })?,
                        constants,
                        nominals,
                    )?,
                ))
            })
            .collect::<Result<_>>()?,
        resolve_type_with(&f.result, constants, nominals)?,
    ))
}
// Closed const expressions are evaluated by normal type resolution. Expressions
// involving method const parameters must also be alpha-equivalent: probing one
// numeric specialization cannot establish equality for every N.
fn symbolic_constants(f: &a::Function) -> Vec<(String, String)> {
    let names = f
        .generics
        .iter()
        .enumerate()
        .filter_map(|(i, g)| {
            matches!(g.kind, a::GenericKind::Const { .. })
                .then_some((g.name.clone(), format!("#{i}")))
        })
        .collect::<BTreeMap<_, _>>();
    fn expression(e: &a::Expr, names: &BTreeMap<String, String>) -> (bool, String) {
        match &e.kind {
            a::ExprKind::Name(n) => (
                names.contains_key(n),
                names.get(n).cloned().unwrap_or_else(|| n.clone()),
            ),
            a::ExprKind::Literal(n) => (false, format!("{n:?}")),
            a::ExprKind::Group(v) => expression(v, names),
            a::ExprKind::Unary { operator, value } => {
                let (found, v) = expression(value, names);
                (found, format!("{operator}({v})"))
            }
            a::ExprKind::Binary {
                operator,
                left,
                right,
            } => {
                let (a, l) = expression(left, names);
                let (b, r) = expression(right, names);
                (a || b, format!("{operator}({l},{r})"))
            }
            _ => (true, format!("unsupported:{:?}", e.kind)),
        }
    }
    fn ty(
        t: &a::Type,
        path: String,
        names: &BTreeMap<String, String>,
        out: &mut Vec<(String, String)>,
    ) {
        match &t.kind {
            a::TypeKind::Named {
                path: name,
                arguments,
            } => {
                if name.len() == 1 && names.contains_key(&name[0]) {
                    out.push((path, names[&name[0]].clone()));
                    return;
                }
                for (i, arg) in arguments.iter().enumerate() {
                    let p = format!("{path}.arg{i}");
                    match arg {
                        a::TypeArgument::Type(t) => ty(t, p, names, out),
                        a::TypeArgument::Const(e) => {
                            let (found, v) = expression(e, names);
                            if found {
                                out.push((p, v));
                            }
                        }
                    }
                }
            }
            a::TypeKind::Array { element, length } => {
                let (found, v) = expression(length, names);
                if found {
                    out.push((format!("{path}.length"), v));
                }
                ty(element, format!("{path}.element"), names, out);
            }
            a::TypeKind::Tuple(fields) => {
                for (i, t) in fields.iter().enumerate() {
                    ty(t, format!("{path}.{i}"), names, out)
                }
            }
            a::TypeKind::Function {
                parameters, result, ..
            } => {
                for (i, (_, t)) in parameters.iter().enumerate() {
                    ty(t, format!("{path}.parameter{i}"), names, out)
                }
                ty(result, format!("{path}.result"), names, out)
            }
            a::TypeKind::Pointer { pointee, .. } => {
                ty(pointee, format!("{path}.pointee"), names, out)
            }
        }
    }
    let mut out = Vec::new();
    for (i, p) in f.parameters.iter().enumerate() {
        if let Some(t) = &p.ty {
            ty(t, format!("parameter{i}"), &names, &mut out)
        }
    }
    ty(&f.result, "result".into(), &names, &mut out);
    for (i, g) in f.generics.iter().enumerate() {
        if let a::GenericKind::Type { constraints, .. } = &g.kind {
            for (j, t) in constraints.iter().enumerate() {
                ty(
                    t,
                    format!(
                        "generic{i}.constraint:{}",
                        match &t.kind {
                            a::TypeKind::Named { path, .. } => path.join("::"),
                            _ => j.to_string(),
                        }
                    ),
                    &names,
                    &mut out,
                )
            }
        }
    }
    for (i, g) in f.generics.iter().enumerate() {
        match &g.kind {
            a::GenericKind::Type {
                default: Some(t), ..
            } => ty(t, format!("generic{i}.default"), &names, &mut out),
            a::GenericKind::Const {
                default: Some(e), ..
            } => {
                let (found, value) = expression(e, &names);
                if found {
                    out.push((format!("generic{i}.default"), value));
                }
            }
            _ => {}
        }
    }
    out.sort();
    out
}
// Defaults can refer only to preceding parameters, including in unused methods.

mod compatibility;
mod defaults;
pub(super) use self::compatibility::compatible;
use self::defaults::defaults;
