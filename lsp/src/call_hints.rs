//! Show the checker-selected parameter mode at the source argument.
use super::analysis::Analysis;
use clust_core::{hir::*, mir::loans::Access};
use serde_json::json;
pub fn collect(out: &mut Analysis, p: &Program, ids: &[Option<String>]) {
    for f in &p.functions {
        block(&f.body, &mut |e| {
            let (id, args) = match &e.kind {
                ExprKind::Call(id, args)
                | ExprKind::BorrowedCall {
                    function: id,
                    arguments: args,
                    ..
                }
                | ExprKind::StorageCall {
                    function: id,
                    arguments: args,
                    ..
                } => (*id, args),
                _ => return,
            };
            let target = &p.functions[id];
            for (a, param) in args.iter().zip(&target.parameters) {
                let local = &target.locals[*param];
                let label = match local.view {
                    Some(Access::Mutable) => "mut borrow",
                    Some(Access::Readonly) => "borrow",
                    None if !local.ty.copyable() => "move",
                    None => continue,
                };
                let span = a.span.source_origin();
                let Some(Some(u)) = ids.get(span.source.0) else {
                    continue;
                };
                let d = &out.files[u];
                let first = d.words.partition_point(|w| w.end <= span.start);
                if d.words.get(first).is_none_or(|w| w.start != span.start)
                    || !super::symbols::ident(d.word(first))
                {
                    continue;
                }
                let hint = json!({"position":d.pos(span.start),"label":label,"paddingRight":true,"kind":2});
                let hints = out.hints.entry(u.clone()).or_default();
                if !hints.contains(&hint) {
                    hints.push(hint)
                }
            }
        })
    }
}
fn block(b: &Block, f: &mut impl FnMut(&Expr)) {
    statements(&b.statements, f)
}
fn statements(items: &[Statement], f: &mut impl FnMut(&Expr)) {
    for item in items {
        match item {
            Statement::Initialize(_, e) | Statement::Evaluate(e) | Statement::Return(Some(e)) => {
                walk(e, f)
            }
            _ => {}
        }
    }
}
fn walk(e: &Expr, f: &mut impl FnMut(&Expr)) {
    use ExprKind::*;
    f(e);
    match &e.kind {
        StoreView(x)
        | StoredView(x)
        | Take(x)
        | Tag(x)
        | Payload(x, _)
        | Unary(_, x)
        | Cast(x)
        | Field(x, _)
        | ArgumentView { place: x, .. }
        | Borrow { place: x, .. } => walk(x, f),
        Identity(a, b)
        | Binary(_, a, b)
        | Index(a, b)
        | Assign {
            place: a, value: b, ..
        } => {
            walk(a, f);
            walk(b, f)
        }
        Tuple(args)
        | Array(args)
        | Call(_, args)
        | StorageCall {
            arguments: args, ..
        }
        | BorrowedCall {
            arguments: args, ..
        }
        | New {
            arguments: args, ..
        }
        | Enum { fields: args, .. } => {
            for a in args {
                walk(a, f)
            }
        }
        CallIndirect(c, args, _) => {
            walk(c, f);
            for a in args {
                walk(a, f)
            }
        }
        Block(b) => block(b, f),
        If { condition, yes, no } => {
            walk(condition, f);
            block(yes, f);
            if let Some(b) = no {
                block(b, f)
            }
        }
        While { condition, body } => {
            walk(condition, f);
            block(body, f)
        }
        Match(arms) => {
            for a in arms {
                statements(&a.guard_bindings, f);
                walk(&a.condition, f);
                statements(&a.bindings, f);
                if let Some(g) = &a.guard {
                    walk(g, f)
                }
                block(&a.body, f)
            }
        }
        View(_) | Value(_) | Local(_) => {}
    }
}
