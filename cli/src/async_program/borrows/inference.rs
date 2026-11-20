use super::*;

pub(crate) fn lower(file: &mut syn::File) -> Result<(), String> {
    infer(&mut file.items, &|_, _| false, false)
}

pub(crate) fn lower_with_foreign(
    file: &mut syn::File,
    foreign: &BorrowedInput<'_>,
) -> Result<(), String> {
    infer(&mut file.items, foreign, true)
}
fn infer(items: &mut Vec<Item>, foreign: &BorrowedInput<'_>, native: bool) -> Result<(), String> {
    if native {
        for item in items.iter_mut() {
            if let Item::Mod(module) = item {
                if let Some((_, body)) = &mut module.content {
                    infer(body, foreign, true)?;
                }
            }
        }
    }
    let functions = items
        .iter()
        .filter_map(|i| if let Item::Fn(f) = i { Some(f) } else { None })
        .collect::<Vec<_>>();
    let mut contracts = Contracts::new();
    for f in &functions {
        contracts.insert(
            f.sig.ident.to_string(),
            params(f)?.into_iter().map(|(_, m)| m).collect(),
        );
    }
    loop {
        let mut next = contracts.clone();
        for f in &functions {
            let receiver_types = f
                .sig
                .inputs
                .iter()
                .filter_map(|arg| {
                    let FnArg::Typed(a) = arg else { return None };
                    let (Pat::Ident(p), Type::Path(ty)) = (&*a.pat, &*a.ty) else {
                        return None;
                    };
                    let mut path = ty.path.clone();
                    for segment in &mut path.segments {
                        segment.arguments = syn::PathArguments::None;
                    }
                    Some((p.ident.to_string(), path))
                })
                .collect();
            let mut uses = Uses {
                receiver_types,
                contracts: &contracts,
                foreign,
                consumed: BTreeSet::new(),
                hidden: BTreeSet::new(),
            };
            uses.visit_block(&f.block);
            let modes = next.get_mut(&f.sig.ident.to_string()).unwrap();
            for ((id, _), mode) in params(f)?.into_iter().zip(modes) {
                if *mode == Mode::Shared && uses.consumed.contains(&id.to_string()) {
                    *mode = Mode::Value;
                }
            }
        }
        if contracts == next {
            break;
        }
        contracts = next;
    }
    let mut wrappers = Vec::new();
    for item in items.iter_mut() {
        let Item::Fn(f) = item else { continue };
        let modes = &contracts[&f.sig.ident.to_string()];
        let mut locals = BTreeSet::new();
        let mut args = Vec::new();
        let mut wrapper = f.clone();
        for (arg, mode) in f.sig.inputs.iter_mut().zip(modes) {
            let FnArg::Typed(arg) = arg else {
                unreachable!()
            };
            let Pat::Ident(p) = arg.pat.as_mut() else {
                unreachable!()
            };
            let id = &p.ident;
            let ty = &arg.ty;
            args.push(match mode {
                Mode::Shared => {
                    locals.insert(id.to_string());
                    arg.ty = parse_quote!(&#ty);
                    parse_quote!(&#id)
                }
                Mode::Exclusive => {
                    locals.insert(id.to_string());
                    p.mutability = None;
                    arg.ty = parse_quote!(&mut #ty);
                    parse_quote!(&mut #id)
                }
                Mode::Value => syn::Expr::Path(parse_quote!(#id)),
            });
        }
        Rewrite {
            native,
            foreign,
            contracts: &contracts,
            borrowed: locals,
        }
        .visit_block_mut(&mut f.block);
        if modes.contains(&Mode::Shared) && !modes.contains(&Mode::Exclusive) {
            if native {
                wrapper.sig.ident = owned_name(&f.sig.ident);
                wrapper
                    .attrs
                    .retain(|a| a.path().is_ident("cfg") || a.path().is_ident("cfg_attr"));
                wrapper.vis = syn::Visibility::Inherited;
            } else {
                f.sig.ident = borrowed_name(&f.sig.ident);
            }
            let target = &f.sig.ident;
            let mut call: Expr = parse_quote!(#target(#(#args),*));
            if wrapper.sig.asyncness.is_some() {
                call = parse_quote!(#call.await)
            }
            wrapper.block = parse_quote!({#call});
            wrappers.push(Item::Fn(wrapper));
        }
    }
    items.extend(wrappers);
    Ok(())
}
