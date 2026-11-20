use super::*;

impl Parser {
    pub(super) fn ty(&mut self) -> Result<Type> {
        self.nested(|p| p.type_inner())
    }

    fn type_inner(&mut self) -> Result<Type> {
        if !matches!(&self.current().kind,TokenKind::Word(word) if word=="dyn") {
            self.unsupported()?;
        }
        let start = self.current().span;
        let kind = if self.eat_word("dyn") {
            let interface = self.ty()?;
            TypeKind::Named {
                path: vec!["$dyn".into()],
                arguments: vec![TypeArgument::Type(interface)],
            }
        } else if self.eat_word("unsafe") {
            let mut ty = self.ty()?;
            let TypeKind::Function { unsafe_, .. } = &mut ty.kind else {
                return Err(self.error("unsafe type requires a function signature"));
            };
            *unsafe_ = true;
            ty.span = start.join(self.last);
            return Ok(ty);
        } else if self.eat("*") {
            let mutable = self.eat_word("var");
            self.lines();
            TypeKind::Pointer {
                mutable,
                pointee: Box::new(self.ty()?),
            }
        } else if self.eat("{") {
            self.lines();
            let element = Box::new(self.ty()?);
            self.expect(";")?;
            self.lines();
            let length = Box::new(self.expr(0, true, false)?);
            self.lines();
            self.expect("}")?;
            TypeKind::Array { element, length }
        } else if self.eat("(") {
            self.lines();
            let mut parameters = Vec::new();
            let mut comma = false;
            while !self.at(")") {
                let mutable = self.eat_word("var");
                let ty = self.ty()?;
                parameters.push((mutable, ty));
                self.lines();
                if !self.eat(",") {
                    break;
                }
                comma = true;
                self.lines();
            }
            self.expect(")")?;
            if self.eat("=>") {
                self.lines();
                TypeKind::Function {
                    unsafe_: false,
                    parameters,
                    result: Box::new(self.ty()?),
                }
            } else {
                if parameters.iter().any(|(mutable, _)| *mutable) {
                    return Err(self.error("var in a type requires a function parameter"));
                }
                if parameters.len() == 1 && !comma {
                    let mut ty = parameters.pop().unwrap().1;
                    ty.span = start.join(self.last);
                    return Ok(ty);
                }
                TypeKind::Tuple(parameters.into_iter().map(|(_, ty)| ty).collect())
            }
        } else {
            let mut path = vec![self.path_segment()?];
            while self.eat(".") {
                path.push(self.path_segment()?);
            }
            let primitive = path.len() == 1
                && [
                    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128",
                    "usize", "f32", "f64", "bool", "char", "str",
                ]
                .contains(&path[0].as_str());
            let arguments = if self.at("<") && !primitive {
                self.type_arguments()?
            } else {
                Vec::new()
            };
            TypeKind::Named { path, arguments }
        };
        Ok(Type {
            kind,
            span: start.join(self.last),
        })
    }

    pub(super) fn close_angle(&mut self) -> Result<()> {
        if self.eat(">") {
            return Ok(());
        }
        if self.at(">>") || self.at(">=") || self.at(">>=") {
            let span = self.current().span;
            self.last = Span {
                end: span.start + 1,
                ..span
            };
            self.edits.push((self.pos, self.tokens[self.pos].clone()));
            let remaining = match self.current().kind {
                TokenKind::Symbol(">>") => ">",
                TokenKind::Symbol(">=") => "=",
                _ => ">=",
            };
            self.tokens[self.pos] = Token {
                kind: TokenKind::Symbol(remaining),
                span: Span {
                    start: span.start + 1,
                    ..span
                },
            };
            Ok(())
        } else {
            Err(self.error("expected '>'"))
        }
    }
}

mod arguments;
