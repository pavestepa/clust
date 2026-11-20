use super::*;

impl Loader {
    pub(super) fn builtin_requests(&self) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        for source in &self.sources {
            let tokens = crate::lexer::lex(SourceId(0), &source.text).tokens;
            for window in tokens.windows(3) {
                use crate::lexer::TokenKind;
                if let [crate::lexer::Token {
                    kind: TokenKind::Word(root),
                    ..
                }, crate::lexer::Token {
                    kind: TokenKind::Symbol("."),
                    ..
                }, crate::lexer::Token {
                    kind: TokenKind::Word(name),
                    ..
                }] = window
                {
                    if root == "core" {
                        names.insert(name.clone());
                    }
                }
            }
        }
        for unit in self.units.values() {
            for item in &unit.items {
                if let ItemKind::Use(bindings) = &item.kind {
                    for binding in bindings {
                        if binding.path.first().is_some_and(|p| p == "core") {
                            names
                                .insert(binding.path.get(1).cloned().unwrap_or_else(|| "*".into()));
                        }
                    }
                }
            }
        }
        names
    }
}

impl Loader {
    pub(super) fn builtins(&mut self) {
        let sources = BTreeMap::from([
            ("intrinsic", crate::intrinsics::SOURCE),
            ("storage", include_str!("../storage.cl")),
            ("option", include_str!("../../../../stdlib/option.cl")),
            ("result", include_str!("../../../../stdlib/result.cl")),
            ("vec", include_str!("../../../../stdlib/vec.cl")),
            ("string", include_str!("../../../../stdlib/string.cl")),
            ("weak", include_str!("../../../../stdlib/weak.cl")),
        ]);
        let mut children = Symbols::new();
        loop {
            let requested = self.builtin_requests();
            let needed = sources
                .keys()
                .filter(|name| {
                    (requested.contains(**name) || requested.contains("*"))
                        && !children.contains_key(**name)
                })
                .map(|name| (*name).to_owned())
                .collect::<BTreeSet<_>>();
            if needed.is_empty() {
                break;
            }
            for name in needed {
                let id = vec!["core".into(), name.clone()];
                children.insert(
                    name.clone(),
                    Symbol {
                        canonical: String::new(),

                        module: Some(id.clone()),
                        restriction: None,
                        origin: Origin::Declaration,
                    },
                );
                self.file(
                    id,
                    self.root.join(format!("\0core.{name}")),
                    self.root.clone(),
                    sources[name.as_str()].into(),
                    Some(&name),
                );
            }
        }
        self.units.insert(
            vec!["core".into()],
            Unit {
                items: vec![],
                children,
            },
        );
    }
}
