use super::*;

pub(super) fn visibility(
    item: &Item,
    current: &[String],
) -> Result<Option<Vec<String>>, Diagnostic> {
    let Some(path) = item.attributes.iter().find_map(|attr| match attr {
        Attribute::Visibility(path) => Some(path),
        _ => None,
    }) else {
        return Ok(item.private.then(|| current.to_vec()));
    };
    let mut scope = current.to_vec();
    let mut index = 0;
    match path.first().map(String::as_str) {
        Some("crate") => {
            scope.clear();
            index = 1;
        }
        Some("self") => index = 1,
        Some("super") => {
            while path.get(index).is_some_and(|s| s == "super") {
                if scope.pop().is_none() {
                    return Err(Diagnostic::new(
                        "E0501",
                        "visibility escapes the crate root",
                        item.span,
                    ));
                }
                index += 1;
            }
        }
        _ => {}
    }
    scope.extend_from_slice(&path[index..]);
    if !current.starts_with(&scope) {
        return Err(Diagnostic::new(
            "E0504",
            "visibility must name an ancestor module",
            item.span,
        ));
    }
    Ok(Some(scope))
}

pub(super) fn accessible(symbol: &Symbol, requester: &[String]) -> bool {
    symbol
        .restriction
        .as_ref()
        .is_none_or(|scope| requester.starts_with(scope))
}

pub(super) fn resolve(
    scopes: &BTreeMap<Vec<String>, Symbols>,
    current: &[String],
    path: &[String],
    span: Span,
) -> Result<Symbol, Diagnostic> {
    let mut id = current.to_vec();
    let mut start = 0;
    match path.first().map(String::as_str) {
        Some("crate") => {
            id.clear();
            start = 1;
        }
        Some("self") => start = 1,
        Some("super") => {
            while path.get(start).is_some_and(|s| s == "super") {
                if id.pop().is_none() {
                    return Err(Diagnostic::new(
                        "E0501",
                        "super escapes the crate root",
                        span,
                    ));
                }
                start += 1;
            }
        }
        Some("core")
            if scopes.contains_key(&vec!["core".into()])
                && !scopes[current].contains_key("core") =>
        {
            id = vec!["core".into()];
            start = 1;
        }
        _ => {}
    }
    let mut symbol = Symbol {
        canonical: String::new(),

        module: Some(id.clone()),
        restriction: None,
        origin: Origin::Declaration,
    };
    for segment in &path[start..] {
        let Some(module) = &symbol.module else {
            return Err(Diagnostic::new(
                "E0503",
                "path segment is not a module",
                span,
            ));
        };
        symbol = scopes
            .get(module)
            .and_then(|s| s.get(segment))
            .cloned()
            .ok_or_else(|| {
                Diagnostic::new(
                    "E0503",
                    format!("unresolved module path '{}'", path.join(".")),
                    span,
                )
            })?;
        if symbol.origin == Origin::Ambiguous {
            return Err(Diagnostic::new(
                "E0502",
                format!("ambiguous glob name '{segment}'"),
                span,
            ));
        }
        if !accessible(&symbol, current) {
            return Err(Diagnostic::new(
                "E0504",
                format!("'{segment}' is private"),
                span,
            ));
        }
    }
    Ok(symbol)
}
