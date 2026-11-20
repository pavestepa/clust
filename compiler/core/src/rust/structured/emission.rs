use super::*;

pub(super) fn scalar_name(ty: &Type) -> Option<String> {
    Some(match ty {
        Type::Unit => "()".into(),
        Type::Bool => "bool".into(),
        Type::Integer(t) => format!("{t:?}").to_lowercase(),
        _ => return None,
    })
}

pub(super) fn block(b: &HBlock, active: &BTreeSet<usize>) -> Option<String> {
    let mut out = String::new();
    for s in &b.statements {
        out.push_str(&match s {
            S::Initialize(id, e) => format!("l{id} = {};\n", expr(e, active)?),
            S::Evaluate(e) => format!("{};\n", expr(e, active)?),
            S::Return(e) => format!(
                "return {};\n",
                match e {
                    Some(e) => expr(e, active)?,
                    None => "()".into(),
                }
            ),
            S::Break => "break;\n".into(),
            S::Continue => "continue;\n".into(),
        });
    }
    Some(out)
}

pub(in super::super) fn emit(p: &hir::Program, active: &BTreeSet<usize>) -> String {
    let mut out = String::new();
    for id in active {
        let f = &p.functions[*id];
        let params = f
            .parameters
            .iter()
            .enumerate()
            .map(|(i, l)| format!("a{i}:{}", scalar_name(&f.locals[*l].ty).unwrap()))
            .collect::<Vec<_>>()
            .join(",");
        out.push_str(&format!(
            "// infallible structured entry\nunsafe fn fast{id}({params})->{} {{\n",
            scalar_name(&f.result).unwrap()
        ));
        for (l, local) in f.locals.iter().enumerate() {
            let init = if let Some(i) = f.parameters.iter().position(|p| *p == l) {
                format!("a{i}")
            } else {
                match local.ty {
                    Type::Unit => "()",
                    Type::Bool => "false",
                    _ => "0",
                }
                .into()
            };
            out.push_str(&format!(
                "let mut l{l}:{}={init};\n",
                scalar_name(&local.ty).unwrap()
            ));
        }
        out.push_str(&block(&f.body, active).unwrap());
        if f.result != Type::Unit {
            out.push_str("unreachable!();\n");
        }
        out.push_str("}\n");
    }
    out
}
