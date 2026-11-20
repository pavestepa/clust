use super::*;

impl Parser {
    pub(in super::super) fn type_arguments(&mut self) -> Result<Vec<TypeArgument>> {
        self.expect("<")?;
        self.lines();
        let mut arguments = Vec::new();
        if self.at(">") || self.at(">>") {
            return Err(self.error("generic argument list cannot be empty"));
        }
        loop {
            let numeric = matches!(self.current().kind, TokenKind::Integer(_)) || self.at("-");
            let arithmetic_name = matches!(self.current().kind, TokenKind::Word(_))
                && self.tokens.get(self.pos + 1).is_some_and(|t| {
                    matches!(t.kind, TokenKind::Symbol("+" | "-" | "*" | "/" | "%"))
                });
            arguments.push(if numeric || arithmetic_name {
                TypeArgument::Const(self.expr(9, true, false)?)
            } else {
                TypeArgument::Type(self.ty()?)
            });
            self.lines();
            if !self.eat(",") {
                break;
            }
            self.lines();
            if self.at(">") || self.at(">>") {
                break;
            }
        }
        self.close_angle()?;
        Ok(arguments)
    }
}

impl Parser {
    pub(in super::super) fn generics(&mut self) -> Result<Vec<Generic>> {
        if !self.eat("<") {
            return Ok(Vec::new());
        }
        self.lines();
        let mut generics = Vec::new();
        let mut saw_default = false;
        loop {
            self.unsupported()?;
            let start = self.current().span;
            let constant = self.eat_word("const");
            let name = self.name()?;
            let kind = if constant {
                self.expect(":")?;
                let ty = self.ty()?;
                if !matches!(&ty.kind, TypeKind::Named { path, arguments } if path == &["usize"] && arguments.is_empty())
                {
                    return Err(self.error("const generic parameters must have type usize"));
                }
                let default = if self.eat("=") {
                    self.lines();
                    Some(self.expr(9, true, false)?)
                } else {
                    None
                };
                if default.is_none() && saw_default {
                    return Err(self.error("required generic parameter follows a default"));
                }
                saw_default |= default.is_some();
                GenericKind::Const { ty, default }
            } else {
                let mut constraints = Vec::new();
                if self.eat(":") {
                    loop {
                        self.lines();
                        constraints.push(self.ty()?);
                        if !self.eat("&") {
                            break;
                        }
                    }
                }
                let default = if self.eat("=") {
                    self.lines();
                    Some(self.ty()?)
                } else {
                    None
                };
                if default.is_none() && saw_default {
                    return Err(self.error("required generic parameter follows a default"));
                }
                saw_default |= default.is_some();
                GenericKind::Type {
                    constraints,
                    default,
                }
            };
            generics.push(Generic {
                name,
                kind,
                span: start.join(self.last),
            });
            self.lines();
            if !self.eat(",") {
                break;
            }
            self.lines();
            if self.at(">") || self.at(">>") {
                break;
            }
        }
        self.close_angle()?;
        Ok(generics)
    }
}
