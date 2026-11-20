use super::*;

impl Parser {
    pub(super) fn aggregate(&mut self) -> Result<ItemKind> {
        self.expect_word("class")?;
        let name = self.name()?;
        let generics = self.generics()?;
        let mut interfaces = Vec::new();
        if self.eat(":") {
            loop {
                self.lines();
                interfaces.push(self.ty()?);
                if !self.eat(",") {
                    break;
                }
            }
        }
        self.lines();
        self.expect("{")?;
        self.separators();
        let mut members = Vec::new();
        while !self.at("}") {
            self.unsupported()?;
            let start = self.current().span;
            let public = self.eat_word("pub");
            let private = !public;
            self.unsupported()?;
            if self.word("fn") || self.word("unsafe") {
                return Err(self.error("Clust methods omit fn and unsafe"));
            }
            let bare_method = matches!(&self.current().kind, TokenKind::Word(name) if name != "new" && name != "drop")
                && self
                    .tokens
                    .get(self.pos + 1)
                    .is_some_and(|t| matches!(t.kind, TokenKind::Symbol("(" | "<")));
            let member = if bare_method {
                let function = self.function(true, false, false)?;
                let dispatch =
                    if !private && function.parameters.first().is_some_and(|p| p.receiver) {
                        MethodDispatch::Inferred
                    } else {
                        MethodDispatch::Ordinary
                    };
                Member::Method {
                    function,
                    private,
                    dispatch,
                }
            } else if self.eat_word("new") {
                let parameters = self.parameters(false)?;
                self.constructor = true;
                let body = self.block();
                self.constructor = false;
                let body = body?;
                Member::Constructor {
                    parameters,
                    body,
                    span: start.join(self.last),
                }
            } else if self.eat_word("drop") {
                self.expect("(")?;
                self.expect(")")?;
                Member::Destructor(self.block()?)
            } else if self.eat_word("const") {
                let (name, ty, value) = self.constant()?;
                Member::Const {
                    name,
                    ty,
                    value,
                    span: start.join(self.last),
                }
            } else {
                let weak = self.eat_word("weak");
                let name = self.name()?;
                self.expect(":")?;
                self.lines();
                let ty = self.ty()?;
                if !self.eat(",") {
                    self.end()?;
                }
                Member::Field {
                    name,
                    ty,
                    weak,
                    private,
                    span: start.join(self.last),
                }
            };
            members.push(member);
            self.separators();
        }
        self.expect("}")?;
        if members
            .iter()
            .filter(|member| matches!(member, Member::Constructor { .. }))
            .count()
            != 1
        {
            return Err(self.error("a class requires exactly one explicit new constructor"));
        }
        Ok(ItemKind::Aggregate {
            class: true,
            name,
            generics,
            interfaces,
            members,
        })
    }
}
