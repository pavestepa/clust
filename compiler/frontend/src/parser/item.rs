use super::*;

impl Parser {
    pub(super) fn item(&mut self) -> Result<Item> {
        self.unsupported()?;
        let start = self.current().span;
        let mut attributes = Vec::new();
        while self.eat("#") {
            self.expect("[")?;
            let name = self.name()?;
            let attribute = match name.as_str() {
                "Copy" => Attribute::Copy,
                "repr" => {
                    self.expect("(")?;
                    self.expect_word("C")?;
                    self.expect(")")?;
                    Attribute::ReprC
                }
                "link" => {
                    self.expect("(")?;
                    let value = self.string()?;
                    self.expect(")")?;
                    Attribute::Link(value)
                }
                "rust" | "safety" => {
                    self.expect("(")?;
                    let value = self.string()?;
                    self.expect(")")?;
                    if name == "rust" {
                        Attribute::Rust(value)
                    } else {
                        Attribute::Safety(value)
                    }
                }
                _ => return Err(Diagnostic::new("E0108", "unsupported attribute", self.last)),
            };
            self.expect("]")?;
            attributes.push(attribute);
            self.lines();
        }
        let public = self.eat_word("pub");
        let private = !public;
        if public && self.eat("(") {
            let mut scope = Vec::new();
            let qualified = self.eat_word("in");
            loop {
                let segment = if self.eat_word("self") {
                    "self".into()
                } else {
                    self.name()?
                };
                scope.push(segment);
                if !qualified || !self.eat(".") {
                    break;
                }
            }
            self.expect(")")?;
            if !matches!(
                scope.first().map(String::as_str),
                Some("crate" | "self" | "super")
            ) {
                return Err(self.error("visibility must start with crate, self or super"));
            }
            attributes.push(Attribute::Visibility(scope));
        }
        let kind = if self.word("fn") || self.word("unsafe") {
            ItemKind::Function(self.function(false, false, false)?)
        } else if self.word("class") {
            self.aggregate()?
        } else if self.eat_word("enum") {
            let name = self.name()?;
            let generics = self.generics()?;
            self.lines();
            self.expect("{")?;
            self.lines();
            let mut variants = Vec::new();
            while !self.at("}") {
                let start = self.current().span;
                let name = self.name()?;
                let fields = if self.eat("(") {
                    let mut fields = Vec::new();
                    self.lines();
                    while !self.at(")") {
                        fields.push(self.ty()?);
                        self.lines();
                        if !self.eat(",") {
                            break;
                        }
                        self.lines();
                    }
                    self.expect(")")?;
                    VariantFields::Tuple(fields)
                } else if self.eat("{") {
                    let mut fields = Vec::new();
                    self.lines();
                    while !self.at("}") {
                        let field = self.name()?;
                        self.expect(":")?;
                        self.lines();
                        let ty = self.ty()?;
                        fields.push((field, ty));
                        self.lines();
                        if !self.eat(",") {
                            break;
                        }
                        self.lines();
                    }
                    self.expect("}")?;
                    VariantFields::Named(fields)
                } else {
                    VariantFields::Unit
                };
                variants.push(Variant {
                    name,
                    fields,
                    span: start.join(self.last),
                });
                self.lines();
                if !self.eat(",") {
                    break;
                }
                self.lines();
            }
            self.expect("}")?;
            ItemKind::Enum {
                name,
                generics,
                variants,
            }
        } else if self.eat_word("interface") {
            let name = self.name()?;
            let generics = self.generics()?;
            self.lines();
            self.expect("{")?;
            let mut methods = Vec::new();
            self.separators();
            while !self.at("}") {
                methods.push(self.function(true, true, false)?);
                self.end()?;
                self.separators();
            }
            self.expect("}")?;
            ItemKind::Interface {
                name,
                generics,
                methods,
            }
        } else if self.eat_word("const") {
            let (name, ty, value) = self.constant()?;
            ItemKind::Const { name, ty, value }
        } else if self.eat_word("mod") {
            let name = self.name()?;
            self.lines();
            let body = if self.eat(";") {
                None
            } else {
                Some(self.nested(|parser| {
                    parser.expect("{")?;
                    parser.separators();
                    let mut items = Vec::new();
                    while !parser.at("}") {
                        items.push(parser.item()?);
                        parser.separators();
                    }
                    parser.expect("}")?;
                    Ok(Module { items })
                })?)
            };
            ItemKind::Module { name, body }
        } else if self.eat_word("use") {
            let mut bindings = Vec::new();
            self.use_tree(Vec::new(), &mut bindings)?;
            self.expect(";")?;
            ItemKind::Use(bindings)
        } else if self.eat_word("extern") {
            let abi = self.string()?;
            if !["C", "Rust"].contains(&abi.as_str()) {
                return Err(self.error("extern ABI must be C or Rust"));
            }
            let rust = attributes
                .iter()
                .any(|attribute| matches!(attribute, Attribute::Rust(_)));
            if (abi == "Rust") != rust {
                return Err(self.error("extern Rust requires a rust path attribute; that attribute cannot be used with extern C"));
            }
            self.lines();
            if self.word("fn") || self.word("unsafe") {
                ItemKind::Function(self.function(false, false, true)?)
            } else {
                self.expect("{")?;
                self.separators();
                let mut functions = Vec::new();
                while !self.at("}") {
                    functions.push(self.function(false, true, true)?);
                    self.end()?;
                    self.separators();
                }
                self.expect("}")?;
                ItemKind::Extern(functions)
            }
        } else {
            self.unsupported()?;
            return Err(self.error("expected declaration"));
        };
        Ok(Item {
            kind,
            private,
            attributes,
            span: start.join(self.last),
        })
    }
}

mod aggregates;
mod attributes;
mod constants;
mod functions;
mod imports;
