//! Explicit scalar Rust adapters. No Clust owner or borrow crosses this ABI.
//! Adapter effects and panics remain calls; reference execution refuses them.
use super::*;
pub(super) fn prepare(module: &a::Module) -> Result<(a::Module, BTreeMap<String, h::RustAdapter>)> {
    let mut module = module.clone();
    let mut adapters = BTreeMap::new();
    for item in &mut module.items {
        let paths: Vec<_> = item
            .attributes
            .iter()
            .filter_map(|attribute| match attribute {
                a::Attribute::Rust(path) => Some(path.clone()),
                _ => None,
            })
            .collect();
        if paths.is_empty() {
            continue;
        }
        if paths.len() != 1 {
            return Err(error(
                item.span,
                "E0505",
                "a Rust adapter requires exactly one rust attribute",
            ));
        }
        let a::ItemKind::Function(function) = &mut item.kind else {
            return Err(error(
                item.span,
                "E0505",
                "rust attribute requires a single extern function declaration",
            ));
        };
        if !function.c_abi || function.body.is_some() || !function.generics.is_empty() {
            return Err(error(
                item.span,
                "E0505",
                "Rust adapters must be nongeneric extern declarations without a body",
            ));
        }
        let safety = item
            .attributes
            .iter()
            .find_map(|attribute| match attribute {
                a::Attribute::Safety(text) => Some(text.clone()),
                _ => None,
            });
        let adapter = h::RustAdapter {
            path: paths[0].clone(),
            unsafe_: function.unsafe_,
            safety,
        };
        if !adapter.valid_path() {
            return Err(error(
                item.span,
                "E0505",
                "Rust adapter path must contain only identifier segments separated by .",
            ));
        }
        for parameter in &function.parameters {
            if parameter.mutable || parameter.receiver {
                return Err(error(
                    parameter.span,
                    "E0505",
                    "scalar Rust adapters take immutable value parameters",
                ));
            }
            scalar_default(parameter.ty.as_ref().ok_or_else(|| {
                error(parameter.span, "E0505", "adapter parameter requires a type")
            })?)?;
        }
        let result = scalar_default(&function.result)?;
        // This typed declaration body is used only for scalar ABI analysis.
        // Both reference engines reject adapter entries before reading it;
        // native code generation emits the actual Rust call instead.
        function.body = Some(a::Block {
            statements: vec![a::Statement {
                kind: a::StatementKind::Return(Some(result)),
                span: function.span,
            }],
            span: function.span,
        });
        function.c_abi = false;
        function.unsafe_ = false;
        adapters.insert(function.name.clone(), adapter);
    }
    Ok((module, adapters))
}
fn scalar_default(ty: &a::Type) -> Result<a::Expr> {
    let kind = match &ty.kind {
        a::TypeKind::Tuple(fields) if fields.is_empty() => a::ExprKind::Tuple(vec![]),
        a::TypeKind::Named { path, arguments } if path.len() == 1 && arguments.is_empty() => {
            a::ExprKind::Literal(match path[0].as_str() {
                "bool" => a::Literal::Bool(false),
                "char" => a::Literal::Char('\0'),
                "str" => a::Literal::String(String::new()),
                "f32" | "f64" => a::Literal::Float("0.0".into()),
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
                | "u128" | "usize" => a::Literal::Integer("0".into()),
                _ => return Err(error(
                    ty.span,
                    "E0505",
                    "Rust adapter boundary currently supports only scalar values and static str",
                )),
            })
        }
        _ => {
            return Err(error(
                ty.span,
                "E0505",
                "Rust adapter boundary currently supports only scalar values and static str",
            ))
        }
    };
    Ok(a::Expr {
        kind,
        span: ty.span,
    })
}
