use super::*;

pub(in super::super) fn freeze_constants(
    method: &mut a::Function,
    generics: &[a::Generic],
    constants: &crate::constants::Constants,
) {
    let bindings = Bindings::module_constants(generics, constants);
    bindings.method(method);
    method.body = None;
}
