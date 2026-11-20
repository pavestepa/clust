use super::*;

impl Parser {
    pub(super) fn use_tree(
        &mut self,
        path: Vec<String>,
        bindings: &mut Vec<UseBinding>,
    ) -> Result<()> {
        self.nested(|parser| parser.use_tree_inner(path, bindings))
    }
}

impl Parser {
    pub(super) fn use_tree_inner(
        &mut self,
        mut path: Vec<String>,
        bindings: &mut Vec<UseBinding>,
    ) -> Result<()> {
        self.lines();
        if self.eat("{") {
            self.lines();
            while !self.at("}") {
                self.use_tree(path.clone(), bindings)?;
                self.lines();
                if !self.eat(",") {
                    break;
                }
                self.lines();
            }
            self.expect("}")?;
        } else if self.eat("*") {
            bindings.push(UseBinding {
                path,
                alias: None,
                glob: true,
            });
        } else {
            let segment = if self.eat_word("weak") {
                "weak".into()
            } else if self.eat_word("self") {
                "self".into()
            } else {
                self.name()?
            };
            if segment != "self" || path.is_empty() {
                path.push(segment);
            }
            if self.eat(".") {
                self.use_tree(path, bindings)?;
            } else {
                let alias = if self.eat_word("as") {
                    Some(self.name()?)
                } else {
                    None
                };
                bindings.push(UseBinding {
                    path,
                    alias,
                    glob: false,
                });
            }
        }
        Ok(())
    }
}
