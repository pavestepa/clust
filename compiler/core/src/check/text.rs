//! Checked UTF-8 byte primitives used by the owned String library.
use super::*;
impl Checker<'_> {
    pub(super) fn text_builtin(
        &mut self,
        name: &str,
        types: &[a::TypeArgument],
        args: &[a::Expr],
        span: Span,
    ) -> Result<Option<h::Expr>> {
        if !matches!(name, "str_byte_len" | "str_byte_at" | "write_byte")
            || self.lookup(name).is_some()
            || self.signatures.contains_key(name)
            || self.mono.borrow().templates.contains_key(name)
        {
            return Ok(None);
        }
        if !types.is_empty() || args.len() != if name == "str_byte_at" { 2 } else { 1 } {
            return Err(error(span, "E0313", "invalid text primitive arguments"));
        }
        let parameter = if name == "write_byte" {
            Type::Integer(I::U8)
        } else {
            Type::Str
        };
        let first = self.expr(&args[0], Some(&parameter), true)?;
        let (kind, ty) = match name {
            "str_byte_len" => (
                h::ExprKind::Unary("str_byte_len", Box::new(first)),
                Type::Integer(I::Usize),
            ),
            "write_byte" => (
                h::ExprKind::Unary("write_byte", Box::new(first)),
                Type::Unit,
            ),
            _ => {
                let index = self.expr(&args[1], Some(&Type::Integer(I::Usize)), true)?;
                (
                    h::ExprKind::Binary("str_byte_at", Box::new(first), Box::new(index)),
                    Type::Integer(I::U8),
                )
            }
        };
        Ok(Some(h::Expr { kind, ty, span }))
    }
}
