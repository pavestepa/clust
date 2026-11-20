use super::*;

impl Parser {
    pub(super) fn expression_list(&mut self, close: &str) -> Result<Vec<Expr>> {
        self.lines();
        let mut values = Vec::new();
        while !self.at(close) {
            values.push(self.expr(0, true, false)?);
            self.lines();
            if !self.eat(",") {
                break;
            }
            self.lines();
        }
        self.expect(close)?;
        Ok(values)
    }
}

pub(super) fn infix(kind: &TokenKind) -> Option<(&'static str, u8, bool)> {
    let TokenKind::Symbol(operator) = kind else {
        return None;
    };
    let (precedence, assignment) = match *operator {
        "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>=" => (1, true),
        "||" => (2, false),
        "&&" => (3, false),
        "==" | "!=" | "<" | ">" | "<=" | ">=" => (4, false),
        "|" => (5, false),
        "^" => (6, false),
        "&" => (7, false),
        "<<" | ">>" => (8, false),
        "+" | "-" => (9, false),
        "*" | "/" | "%" => (10, false),
        _ => return None,
    };
    Some((operator, precedence, assignment))
}

pub(super) fn expression_type(expr: &Expr) -> Option<Type> {
    let (path, arguments) = match &expr.kind {
        ExprKind::Name(name) => (vec![name.clone()], Vec::new()),
        ExprKind::Member { value, name } => {
            let base = expression_type(value)?;
            let TypeKind::Named {
                mut path,
                arguments,
            } = base.kind
            else {
                return None;
            };
            path.push(name.clone());
            (path, arguments)
        }
        ExprKind::Specialize { value, arguments } => {
            let base = expression_type(value)?;
            let TypeKind::Named { path, .. } = base.kind else {
                return None;
            };
            (path, arguments.clone())
        }
        _ => return None,
    };
    Some(Type {
        kind: TypeKind::Named { path, arguments },
        span: expr.span,
    })
}
