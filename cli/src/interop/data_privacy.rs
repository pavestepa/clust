//! Give each native class its own Rust module so its private fields and methods
//! are accessible only to its implementation. Derives run on the actual struct.
use syn::{parse_quote, Item};
pub(super) fn seal(items: &mut Vec<Item>) {
    let mut input = std::mem::take(items).into_iter().peekable();
    while let Some(mut item) = input.next() {
        if let Item::Mod(m) = &mut item {
            if let Some((_, body)) = &mut m.content {
                seal(body);
            }
        }
        if let Item::Struct(s) = &mut item {
            let name = s.ident.clone();
            let vis = s.vis.clone();
            if matches!(input.peek(), Some(Item::Impl(_))) {
                let implementation = input.next().unwrap();
                let destructor = if matches!(input.peek(), Some(Item::Impl(i)) if i.trait_.as_ref().is_some_and(|(_,path,_)| path.is_ident("Drop")))
                {
                    input.next()
                } else {
                    None
                };
                let module = quote::format_ident!("__clust_class_{}", name);
                s.vis = parse_quote!(pub);
                items
                    .push(parse_quote!(mod #module {use super::*; #s #implementation #destructor}));
                items.push(parse_quote!(#vis use #module::#name;));
                continue;
            }
        }
        items.push(item);
    }
}
