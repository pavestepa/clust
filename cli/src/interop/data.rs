//! A native data class has source-named fields and a checked field constructor.
use super::lower::{matching, source};
use std::collections::BTreeSet;
pub(super) fn class(
    t: &[&str],
    i: usize,
    roots: &BTreeSet<String>,
) -> Result<(String, usize), String> {
    let name = t.get(i + 1).ok_or("missing class name")?;
    let open = (i + 2..t.len())
        .find(|&k| t[k] == "{")
        .ok_or("missing class body")?;
    let header = source(&t[i + 2..open], roots)?;
    let generic: syn::Generics = syn::parse_str(&header)
        .map_err(|_| "unsupported native class parameters or inheritance")?;
    let (impl_params, type_args, bounds) = generic.split_for_impl();
    let impl_params = quote::quote!(#impl_params);
    let type_args = quote::quote!(#type_args);
    let bounds = quote::quote!(#bounds);
    let generic = quote::quote!(#generic);
    let mut destructor = None;
    let close = matching(t, open, "{", "}")?;
    let mut fields = vec![];
    let mut names = BTreeSet::new();
    let mut methods = String::new();
    let mut ctor = false;
    let mut j = open + 1;
    while j < close {
        if matches!(t[j], ";" | ",") {
            j += 1;
            continue;
        }
        let public = t[j] == "pub";
        if public {
            j += 1
        }
        let member = *t.get(j).ok_or("missing class member")?;
        j += 1;
        let vis = if public { "pub " } else { "" };
        if t.get(j) == Some(&":") {
            let start = j + 1;
            j = start;
            let mut angle = 0i32;
            while j < close {
                if angle == 0
                    && (matches!(t[j], ";" | "," | "pub")
                        || j > start && matches!(t.get(j + 1), Some(&":") | Some(&"(")))
                {
                    break;
                }
                match t[j] {
                    "<" => angle += 1,
                    ">" => angle -= 1,
                    ">>" => angle -= 2,
                    _ => {}
                }
                j += 1;
            }
            if !names.insert(member) {
                return Err("duplicate native data field".into());
            }
            fields.push(format!("{vis}{member}: {}", source(&t[start..j], roots)?));
            continue;
        }
        if t.get(j) != Some(&"(") {
            return Err("expected data class field or method".into());
        }
        let end = matching(t, j, "(", ")")?;
        let params = super::parameters::parameters(&t[j..=end], roots)?;
        let body = (end + 1..close)
            .find(|&k| t[k] == "{")
            .ok_or("expected method body")?;
        let last = matching(t, body, "{", "}")?;
        if member == "new" {
            if ctor {
                return Err("duplicate constructor".into());
            }
            ctor = true;
            let mut init = vec![];
            let mut k = body + 1;
            while k < last {
                if t.get(k..k + 2) != Some(&["self", "."][..]) || t.get(k + 3) != Some(&"=") {
                    return Err(
                        "native constructors require self.field = expression; initializers".into(),
                    );
                }
                let sem = (k + 4..last)
                    .find(|&n| t[n] == ";")
                    .ok_or("expected constructor semicolon")?;
                init.push(format!("{}: {}", t[k + 2], source(&t[k + 4..sem], roots)?));
                k = sem + 1;
            }
            methods.push_str(&format!(
                "pub fn new({params})->Self {{Self{{{}}}}}",
                init.join(",")
            ));
        } else if member == "drop" {
            if public || !params.is_empty() || destructor.is_some() || end + 1 != body {
                return Err("expected one private drop() with no parameters or result".into());
            }
            let body = source(&t[body + 1..last], roots)?;
            destructor = Some(format!("impl {impl_params} Drop for {name}{type_args} {bounds} {{ fn drop(&mut self) {{ {body} }} }}"));
        } else {
            methods.push_str(&format!(
                "{vis}fn {member}({params}) {} {{ {} }}",
                source(&t[end + 1..body], roots)?,
                source(&t[body + 1..last], roots)?
            ));
        }
        j = last + 1;
    }
    if !ctor {
        return Err("native class requires an explicit new constructor".into());
    }
    let destructor = destructor.unwrap_or_default();
    Ok((
        format!(
            "struct {name}{generic} {bounds} {{ {} }} impl {impl_params} {name}{type_args} {bounds} {{ {methods} }} {destructor}",
            fields.join(",")
        ),
        close,
    ))
}
