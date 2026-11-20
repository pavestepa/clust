use super::*;

impl Loader {
    pub(super) fn link(&self) -> Result<Module, Diagnostic> {
        let mut scopes = BTreeMap::new();
        for (index, (id, unit)) in self.units.iter().enumerate() {
            let mut symbols = unit.children.clone();
            if self.trusted.contains(id) {
                for intrinsic in crate::intrinsics::Intrinsic::ALL {
                    symbols.insert(
                        intrinsic.name().into(),
                        Symbol {
                            canonical: intrinsic.symbol().into(),

                            module: None,
                            restriction: Some(id.clone()),
                            origin: Origin::Declaration,
                        },
                    );
                }
            }
            for item in &unit.items {
                if let Some(name) = item_name(&item.kind) {
                    let canonical = if id.is_empty() && name == "main" {
                        "main".into()
                    } else {
                        format!("m{index}::{name}")
                    };
                    if symbols
                        .insert(
                            name.into(),
                            Symbol {
                                canonical,

                                module: None,
                                restriction: visibility(item, id)?,
                                origin: Origin::Declaration,
                            },
                        )
                        .is_some()
                    {
                        return Err(Diagnostic::new(
                            "E0301",
                            "duplicate module declaration",
                            item.span,
                        ));
                    }
                }
            }
            scopes.insert(id.clone(), symbols);
        }
        // Resolve use trees and re-exports to a fixed point, including cycles.
        loop {
            let mut progress = false;
            for (id, unit) in &self.units {
                for item in &unit.items {
                    let ItemKind::Use(bindings) = &item.kind else {
                        continue;
                    };
                    for binding in bindings {
                        let Ok(symbol) = resolve(&scopes, id, &binding.path, item.span) else {
                            continue;
                        };
                        let imported = if binding.glob {
                            let Some(target) = &symbol.module else {
                                continue;
                            };
                            scopes[target]
                                .iter()
                                .filter(|(_, s)| accessible(s, id))
                                .map(|(n, s)| (n.clone(), s.clone()))
                                .collect::<Vec<_>>()
                        } else {
                            vec![(
                                binding
                                    .alias
                                    .clone()
                                    .unwrap_or_else(|| binding.path.last().unwrap().clone()),
                                symbol,
                            )]
                        };
                        for (name, mut symbol) in imported {
                            if let Some(previous) = scopes[id].get(&name).cloned() {
                                let same = previous.canonical == symbol.canonical
                                    && (symbol.module.is_none()
                                        || previous.module == symbol.module);
                                if same {
                                    continue;
                                }
                                if binding.glob {
                                    if previous.origin == Origin::Glob {
                                        let mut ambiguous = previous;
                                        ambiguous.origin = Origin::Ambiguous;
                                        scopes.get_mut(id).unwrap().insert(name, ambiguous);
                                        progress = true;
                                    }
                                    continue;
                                }
                                if previous.origin != Origin::Glob
                                    && previous.origin != Origin::Ambiguous
                                {
                                    if previous.module.is_some()
                                        && previous.canonical.is_empty()
                                        && symbol.module.is_none()
                                    {
                                        let mut combined = previous;
                                        combined.canonical = symbol.canonical.clone();
                                        scopes.get_mut(id).unwrap().insert(name, combined);
                                        progress = true;
                                        continue;
                                    }
                                    return Err(Diagnostic::new(
                                        "E0502",
                                        "use conflicts with an existing name",
                                        item.span,
                                    ));
                                }
                            }
                            let restriction = visibility(item, id)?;
                            if !item.private
                                && symbol.restriction.as_ref().is_some_and(|limit| {
                                    restriction
                                        .as_ref()
                                        .is_none_or(|scope| !scope.starts_with(limit))
                                })
                            {
                                return Err(Diagnostic::new(
                                    "E0504",
                                    "re-export exceeds the declaration's visibility",
                                    item.span,
                                ));
                            }
                            symbol.origin = if symbol.origin == Origin::Ambiguous {
                                Origin::Ambiguous
                            } else if binding.glob {
                                Origin::Glob
                            } else {
                                Origin::Explicit
                            };
                            symbol.restriction = restriction;
                            scopes.get_mut(id).unwrap().insert(name, symbol);
                            progress = true;
                        }
                    }
                }
            }
            if !progress {
                break;
            }
        }
        let mut result = vec![];
        for (id, unit) in &self.units {
            for item in &unit.items {
                if let ItemKind::Use(bindings) = &item.kind {
                    for binding in bindings {
                        let symbol = resolve(&scopes, id, &binding.path, item.span)?;
                        if binding.glob && symbol.module.is_none() {
                            return Err(Diagnostic::new(
                                "E0503",
                                "glob use requires a module",
                                item.span,
                            ));
                        }
                    }
                }
            }
            let mut rewrite = Rewrite {
                globals: &scopes[id],
                scopes: vec![BTreeSet::new()],
                modules: &scopes,
                current: id,
                error: None,
            };
            for item in &unit.items {
                if matches!(item.kind, ItemKind::Use(_)) {
                    continue;
                }
                let mut item = item.clone();
                rewrite.item(&mut item);
                // Module visibility is checked while resolving paths. The flattened
                // checker still enforces lexical privacy for class members.
                if matches!(item.kind, ItemKind::Function(_)) {
                    item.private = false;
                }
                result.push(item);
            }
            if let Some(error) = rewrite.error {
                return Err(error);
            }
        }
        Ok(Module { items: result })
    }
}
