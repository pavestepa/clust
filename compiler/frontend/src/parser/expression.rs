use super::*;

impl Parser {
    pub(super) fn literal(&mut self) -> Option<Literal> {
        let value = match self.current().kind.clone() {
            TokenKind::Integer(s) => Literal::Integer(s),
            TokenKind::Float(s) => Literal::Float(s),
            TokenKind::String(s) => Literal::String(s),
            TokenKind::Char(c) => Literal::Char(c),
            TokenKind::Word(s) if s == "true" || s == "false" => Literal::Bool(s == "true"),
            _ => return None,
        };
        self.bump();
        Some(value)
    }

    pub(super) fn expr(&mut self, minimum: u8, soft: bool, before_body: bool) -> Result<Expr> {
        self.nested(|p| p.expression(minimum, soft, before_body))
    }

    fn expression(&mut self, minimum: u8, soft: bool, before_body: bool) -> Result<Expr> {
        if soft {
            self.lines();
        }
        let mut left = self.prefix(soft, before_body)?;
        let mut operations = 0;
        loop {
            operations += 1;
            if operations > 512 {
                return Err(Diagnostic::new(
                    "E0201",
                    "expression chain limit exceeded",
                    self.current().span,
                ));
            }
            if soft {
                self.lines();
            } else if self.newline() {
                let (saved, last) = (self.pos, self.last);
                self.lines();
                if !self.at(".") {
                    self.pos = saved;
                    self.last = last;
                    break;
                }
            }
            if minimum <= 13 && self.eat(".") {
                self.lines();
                let name = if self.eat_word("new") {
                    "new".to_owned()
                } else if let TokenKind::Integer(s) = &self.current().kind {
                    let s = s.clone();
                    self.bump();
                    s
                } else {
                    self.name()?
                };
                let span = left.span.join(self.last);
                left = Expr {
                    kind: ExprKind::Member {
                        value: Box::new(left),
                        name,
                    },
                    span,
                };
                continue;
            }
            if minimum <= 13
                && self.at("<")
                && matches!(left.kind, ExprKind::Name(_) | ExprKind::Member { .. })
            {
                let (saved, last, edits) = (self.pos, self.last, self.edits.len());
                if let Ok(arguments) = self.type_arguments() {
                    if self.at("(")
                        || self.at(".")
                        || (!before_body && self.at("{"))
                        || self.at_eof()
                        || self.newline()
                        || [")", "]", "}", ",", ";"].iter().any(|token| self.at(token))
                    {
                        let span = left.span.join(self.last);
                        left = Expr {
                            kind: ExprKind::Specialize {
                                value: Box::new(left),
                                arguments,
                            },
                            span,
                        };
                        continue;
                    }
                }
                for (index, token) in self.edits.drain(edits..).rev() {
                    self.tokens[index] = token;
                }
                self.pos = saved;
                self.last = last;
            }
            if minimum <= 13 && self.eat("(") {
                let arguments = self.expression_list(")")?;
                let span = left.span.join(self.last);
                let (callee, types) = match left.kind {
                    ExprKind::Specialize { value, arguments } => (value, arguments),
                    _ => (Box::new(left), Vec::new()),
                };
                if let ExprKind::Member { value, name } = &callee.kind {
                    if name == "new" && !matches!(&value.kind, ExprKind::Name(n) if n == "super") {
                        let ty = expression_type(value)
                            .ok_or_else(|| self.error("constructor requires a class name"))?;
                        left = Expr {
                            kind: ExprKind::New { ty, arguments },
                            span,
                        };
                        continue;
                    }
                }
                left = Expr {
                    kind: ExprKind::Call {
                        callee,
                        types,
                        arguments,
                    },
                    span,
                };
                continue;
            }
            if minimum <= 13 && self.eat("[") {
                let index = self.expr(0, true, false)?;
                self.expect("]")?;
                let span = left.span.join(self.last);
                left = Expr {
                    kind: ExprKind::Index {
                        value: Box::new(left),
                        index: Box::new(index),
                    },
                    span,
                };
                continue;
            }
            if minimum <= 13 && !before_body && self.at("{") {
                if let Some(ty) = expression_type(&left) {
                    self.bump();
                    self.lines();
                    let mut fields = Vec::new();
                    while !self.at("}") {
                        let start = self.current().span;
                        let name = self.name()?;
                        let value = if self.eat(":") {
                            self.expr(0, true, false)?
                        } else {
                            Expr {
                                kind: ExprKind::Name(name.clone()),
                                span: start,
                            }
                        };
                        fields.push((name, value));
                        self.lines();
                        if !self.eat(",") {
                            break;
                        }
                        self.lines();
                    }
                    self.expect("}")?;
                    let span = left.span.join(self.last);
                    left = Expr {
                        kind: ExprKind::Construct { ty, fields },
                        span,
                    };
                    continue;
                }
            }
            if minimum <= 11 && self.eat_word("as") {
                self.lines();
                let ty = self.ty()?;
                let span = left.span.join(self.last);
                left = Expr {
                    kind: ExprKind::Cast {
                        value: Box::new(left),
                        ty,
                    },
                    span,
                };
                continue;
            }
            if self.at("?") || self.at("...") || self.at("..") || self.at("..=") {
                self.unsupported()?;
            }
            let Some((operator, precedence, assignment)) = infix(&self.current().kind) else {
                break;
            };
            if precedence < minimum {
                break;
            }
            if precedence == 4
                && matches!(&left.kind, ExprKind::Binary {operator,..} if ["==","!=","<",">","<=",">="].contains(operator))
            {
                return Err(self.error("comparison chains require parentheses"));
            }
            self.bump();
            self.lines();
            let right = self.expr(
                if assignment {
                    precedence
                } else {
                    precedence + 1
                },
                soft,
                before_body,
            )?;
            let span = left.span.join(right.span);
            left = Expr {
                kind: if assignment {
                    ExprKind::Assign {
                        operator,
                        place: Box::new(left),
                        value: Box::new(right),
                    }
                } else {
                    ExprKind::Binary {
                        operator,
                        left: Box::new(left),
                        right: Box::new(right),
                    }
                },
                span,
            };
        }
        Ok(left)
    }
}

mod operands;
mod prefix;
use self::operands::expression_type;
use self::operands::infix;
