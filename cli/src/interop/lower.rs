use std::collections::BTreeSet;
pub(super) fn matching(t: &[&str], start: usize, open: &str, close: &str) -> Result<usize, String> {
    let mut depth = 0;
    for (i, w) in t.iter().enumerate().skip(start) {
        if *w == open {
            depth += 1
        } else if *w == close {
            depth -= 1;
            if depth == 0 {
                return Ok(i);
            }
        }
    }
    Err(format!("expected closing '{close}'"))
}
pub(super) fn source(t: &[&str], roots: &BTreeSet<String>) -> Result<String, String> {
    let mut out = String::new();
    let mut i = 0;
    let mut path = false;
    let mut import = false;
    while i < t.len() {
        let w = t[i];
        let prev = i.checked_sub(1).map(|j| t[j]).unwrap_or("");
        let next = t.get(i + 1).copied().unwrap_or("");
        if w == "class" {
            let (code, end) = super::data::class(t, i, roots)?;
            out.push_str(&code);
            i = end + 1;
            continue;
        }
        if w == "#" && next == "[" {
            let end = matching(t, i + 1, "[", "]")?;
            let attrs = source(&t[i + 2..end], roots)?;
            out.push_str(&format!("#[{attrs}] "));
            i = end + 1;
            continue;
        }
        let ty = prev.chars().next().is_some_and(|c| c.is_ascii_uppercase());
        if matches!(w, "println" | "print" | "assert") && next == "(" && prev != "." && prev != "fn"
        {
            let end = matching(t, i + 1, "(", ")")?;
            let arg = source(&t[i + 2..end], roots)?;
            if w == "assert" {
                out.push_str(&format!(
                    "{{let __clust_condition={arg}; assert!(__clust_condition);}} "
                ));
            } else {
                out.push_str(&format!(
                    "{{let __clust_print=&({arg}); {w}!(\"{{}}\",__clust_print);}} "
                ));
            }
            i = end + 1;
            continue;
        }
        let raw = match w {
            "core"
                if next == "."
                    && matches!(
                        t.get(i + 2),
                        Some(&"vec" | &"string" | &"option" | &"result")
                    ) =>
            {
                "crate::__clust_standard"
            }
            "use" => {
                import = true;
                w
            }
            ";" => {
                import = false;
                w
            }
            "." if import || path || roots.contains(prev) || ty || matches!(prev, ">" | ">>") => {
                "::"
            }
            "var" if matches!(prev, "(" | ",") => "mut",
            "var" => "let mut",
            "unsafe" | "extern" | "struct" | "impl" | "mut" => {
                return Err(format!(
                    "'{w}' in mixed Clust source requires a Rust module"
                ))
            }
            "<" if i.checked_sub(2).and_then(|j| t.get(j)) != Some(&"fn")
                && generic_call(t, i, ty) =>
            {
                "::<"
            }
            _ => w,
        };
        path = raw == "::" || (path && next == ".");
        out.push_str(raw);
        out.push(' ');
        i += 1;
    }
    Ok(out)
}

fn generic_call(t: &[&str], i: usize, ty: bool) -> bool {
    let mut depth = 0i32;
    for j in i..t.len() {
        match t[j] {
            "<" => depth += 1,
            ">" => depth -= 1,
            ">>" => depth -= 2,
            ";" | "{" | "=" => return false,
            _ => {}
        }
        if depth <= 0 {
            return depth == 0 && (t.get(j + 1) == Some(&"(") || ty && t.get(j + 1) == Some(&"."));
        }
    }
    false
}
