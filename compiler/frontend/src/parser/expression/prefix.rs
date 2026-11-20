use super::*;

impl Parser {
    pub(super) fn prefix(&mut self, soft: bool, before_body: bool) -> Result<Expr> {
        self.unsupported()?;
        let start = self.current().span;
        let kind = if self.eat_word("fn") {
            let parameters = self.parameters(false)?;
            let result = if self.eat("->") {
                self.ty()?
            } else {
                Type {
                    kind: TypeKind::Tuple(vec![]),
                    span: self.last,
                }
            };
            let constructor = self.constructor;
            self.constructor = false;
            let body = self.block()?;
            self.constructor = constructor;
            ExprKind::Closure(Box::new(Function {
                name: "$closure".into(),
                generics: vec![],
                parameters,
                result,
                span: start.join(body.span),
                body: Some(body),
                c_abi: false,
                unsafe_: false,
            }))
        } else if let Some(value) = self.literal() {
            ExprKind::Literal(value)
        } else if let TokenKind::Symbol(operator @ ("-" | "!" | "*" | "&")) = self.current().kind {
            self.bump();
            let operator = if operator == "&" && self.eat_word("var") {
                "&var"
            } else {
                operator
            };
            self.lines();
            let value = self.expr(12, soft, before_body)?;
            ExprKind::Unary {
                operator,
                value: Box::new(value),
            }
        } else if self.eat("(") {
            self.lines();
            if self.eat(")") {
                ExprKind::Tuple(Vec::new())
            } else {
                let first = self.expr(0, true, false)?;
                if self.eat(",") {
                    let mut values = vec![first];
                    values.extend(self.expression_list(")")?);
                    ExprKind::Tuple(values)
                } else {
                    self.expect(")")?;
                    ExprKind::Group(Box::new(first))
                }
            }
        } else if self.eat("{") {
            ExprKind::Array(self.expression_list("}")?)
        } else if self.at("[") {
            return Err(Diagnostic::new(
                "E0105",
                "dynamic array literals are not supported",
                start,
            ));
        } else if self.eat_word("new") {
            return Err(self.error("use Type.new(...) to construct a class"));
        } else if self.eat_word("if") {
            if self.word("let") {
                return Err(Diagnostic::new(
                    "E0107",
                    "if let is not supported; use match",
                    self.current().span,
                ));
            }
            let condition = Box::new(self.expr(0, false, true)?);
            let then_block = self.block_with_tail(true)?;
            let (saved, last) = (self.pos, self.last);
            self.lines();
            let else_branch = if self.eat_word("else") {
                self.lines();
                Some(Box::new(if self.word("if") {
                    self.expr(0, false, before_body)?
                } else {
                    let block = self.block_with_tail(true)?;
                    Expr {
                        span: block.span,
                        kind: ExprKind::Branch(block),
                    }
                }))
            } else {
                self.pos = saved;
                self.last = last;
                None
            };
            ExprKind::If {
                condition,
                then_block,
                else_branch,
            }
        } else if self.eat_word("loop") {
            let condition = Box::new(Expr {
                kind: ExprKind::Literal(Literal::Bool(true)),
                span: self.last,
            });
            let body = self.block()?;
            ExprKind::While { condition, body }
        } else if self.eat_word("for") {
            return self.for_loop(start);
        } else if self.eat_word("while") {
            if self.word("let") {
                return Err(Diagnostic::new(
                    "E0107",
                    "while let is not supported; use match",
                    self.current().span,
                ));
            }
            let condition = Box::new(self.expr(0, false, true)?);
            let body = self.block()?;
            ExprKind::While { condition, body }
        } else if self.eat_word("match") {
            let value = Box::new(self.expr(0, false, true)?);
            self.lines();
            self.expect("{")?;
            self.lines();
            let mut arms = Vec::new();
            while !self.at("}") {
                let start = self.current().span;
                let pattern = self.pattern()?;
                let guard = if self.eat_word("if") {
                    Some(self.expr(0, false, true)?)
                } else {
                    None
                };
                self.expect("=>")?;
                self.lines();
                let value = if self.at("{") {
                    let block = self.block_with_tail(true)?;
                    Expr {
                        span: block.span,
                        kind: ExprKind::Branch(block),
                    }
                } else {
                    self.expr(0, false, false)?
                };
                arms.push(Arm {
                    pattern,
                    guard,
                    value,
                    span: start.join(self.last),
                });
                self.lines();
                if !self.eat(",") {
                    break;
                }
                self.lines();
            }
            self.expect("}")?;
            ExprKind::Match { value, arms }
        } else if self.eat_word("self") {
            ExprKind::Name("this".into())
        } else {
            ExprKind::Name(self.name()?)
        };
        Ok(Expr {
            kind,
            span: start.join(self.last),
        })
    }
}
