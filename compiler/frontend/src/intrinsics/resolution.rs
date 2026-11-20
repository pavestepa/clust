use super::*;

impl Intrinsic {
    pub fn resolve(symbol: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|i| i.symbol() == symbol)
    }
}
