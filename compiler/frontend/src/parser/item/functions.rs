use super::*;

impl Parser {
    pub(in super::super) fn function(
        &mut self,
        method: bool,
        signature: bool,
        c_abi: bool,
    ) -> Result<Function> {
        let start = self.current().span;
        let unsafe_ = self.eat_word("unsafe");
        if unsafe_ && !c_abi {
            return Err(self.error("unsafe functions must be implemented in Rust and imported"));
        }
        if method {
            if self.word("fn") {
                return Err(self.error("Clust methods omit fn"));
            }
        } else {
            self.expect_word("fn")?;
        }
        let name = self.name()?;
        let generics = self.generics()?;
        if c_abi && !generics.is_empty() {
            return Err(self.error("C functions cannot be generic"));
        }
        let parameters = self.parameters(method)?;
        let result = if self.eat("->") {
            self.lines();
            self.ty()?
        } else {
            Type {
                kind: TypeKind::Tuple(Vec::new()),
                span: self.last,
            }
        };
        let body = if (signature && !self.at("{")) || c_abi {
            None
        } else {
            Some(self.block()?)
        };
        Ok(Function {
            name,
            generics,
            parameters,
            result,
            body,
            c_abi,
            unsafe_,
            span: start.join(self.last),
        })
    }
}

impl Parser {
    pub(in super::super) fn parameters(&mut self, method: bool) -> Result<Vec<Parameter>> {
        self.expect("(")?;
        self.lines();
        let mut parameters = Vec::new();
        while !self.at(")") {
            self.unsupported()?;
            let start = self.current().span;
            let mutable = self.eat_word("var");
            let receiver = self.eat_word("self");
            let (pattern, ty) = if receiver {
                if !method || !parameters.is_empty() {
                    return Err(self.error("receiver must be the first parameter of a method"));
                }
                (
                    Pattern {
                        kind: PatternKind::Binding {
                            name: "this".into(),
                            by_ref: true,
                            mutable,
                        },
                        span: self.last,
                    },
                    None,
                )
            } else {
                let pattern = self.pattern()?;
                self.expect(":")?;
                self.lines();
                let ty = self.ty()?;
                (pattern, Some(ty))
            };
            parameters.push(Parameter {
                pattern,
                mutable,
                ty,
                receiver,
                span: start.join(self.last),
            });
            self.lines();
            if !self.eat(",") {
                break;
            }
            self.lines();
        }
        self.expect(")")?;
        Ok(parameters)
    }
}
