use super::*;

/// All entries of a consuming function require O; borrowed storage must use a
/// genuinely non-consuming API, never an alternate ABI for the same sink.
pub(in super::super) fn parameter_modes(
    declaration: &a::Function,
    types: &[Type],
    consuming: &Consuming,
) -> Vec<Option<Access>> {
    let mut modes = methods::parameter_modes(declaration, types);
    if let Some(flags) = consuming.borrow().get(base_name(&declaration.name)) {
        for (mode, consumes) in modes.iter_mut().zip(flags) {
            if *consumes {
                *mode = None;
            }
        }
    }
    modes
}
