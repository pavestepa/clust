//! Ownership effects of a resolved native Rust signature. An async result keeps
//! its argument borrows alive until its future completes or is dropped.
use super::super::{
    loans::{Access, Loans},
    Place,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Parameter {
    Value,
    Shared,
    Exclusive,
}
#[derive(Clone, Debug)]
pub struct Contract {
    pub inputs: Vec<Parameter>,
    pub result_origins: Vec<usize>,
}
impl Contract {
    /// Check simultaneous loans of known local places. Rust validates projections,
    /// two-phase method receivers, moves and unresolved generic substitutions.
    pub fn check_arguments(&self, args: &[Option<Place>]) -> Result<(), String> {
        let mut loans = Loans::default();
        for (arg, mode) in args.iter().zip(&self.inputs) {
            let Some(place) = arg else { continue };
            let access = match mode {
                Parameter::Value => continue,
                Parameter::Shared => Access::Readonly,
                Parameter::Exclusive => Access::Mutable,
            };
            loans
                .borrow(vec![place.clone()], access, Access::Mutable)
                .map_err(|_| "E0381: overlapping borrows in Rust call".to_owned())?;
        }
        Ok(())
    }
}
