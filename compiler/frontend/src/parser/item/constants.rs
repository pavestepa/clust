use super::*;

impl Parser {
    pub(super) fn constant(&mut self) -> Result<(String, Type, Expr)> {
        let name = self.name()?;
        self.expect(":")?;
        self.lines();
        let ty = self.ty()?;
        self.expect("=")?;
        self.lines();
        let value = self.expr(0, false, false)?;
        self.end()?;
        Ok((name, ty, value))
    }
}
