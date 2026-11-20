use super::*;

pub(in super::super) fn compatible(
    required: &a::Function,
    actual: &a::Function,
    generics: &[a::Generic],
    constants: &crate::constants::Constants,
) -> Result<bool> {
    let mut scoped = constants.clone();
    for generic in generics {
        scoped.remove(&generic.name);
    }
    let keys = Keys { constants: &scoped };
    Ok(keys.signature(required)? == keys.signature(actual)?)
}
