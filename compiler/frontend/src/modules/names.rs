use super::*;

pub(super) fn item_name(item: &ItemKind) -> Option<&str> {
    Some(match item {
        ItemKind::Function(function) => &function.name,
        ItemKind::Aggregate { name, .. }
        | ItemKind::Enum { name, .. }
        | ItemKind::Interface { name, .. }
        | ItemKind::Const { name, .. } => name,
        _ => return None,
    })
}

impl Rewrite<'_> {
    pub(super) fn path(&mut self, path: &mut Vec<String>, span: Span) {
        if path.is_empty()
            || self
                .scopes
                .iter()
                .rev()
                .any(|scope| scope.contains(&path[0]))
        {
            return;
        }
        let mut lookup = path.clone();
        if lookup[0] == "this" {
            lookup[0] = "self".into();
        }
        for count in (1..=path.len()).rev() {
            match resolve(self.modules, self.current, &lookup[..count], span) {
                Ok(symbol) if !symbol.canonical.is_empty() => {
                    path.splice(..count, [symbol.canonical]);
                    return;
                }
                Ok(_) => {}
                Err(error) if matches!(error.code, "E0504" | "E0502") => {
                    self.error.get_or_insert(error);
                    return;
                }
                Err(_) => {}
            }
        }
    }
}

impl Rewrite<'_> {
    pub(super) fn name(&self, name: &mut String) {
        if !self.scopes.iter().rev().any(|scope| scope.contains(name)) {
            if let Some(symbol) = self.globals.get(name) {
                *name = symbol.canonical.clone();
            }
        }
    }
}
