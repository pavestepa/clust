use super::*;

pub(super) fn ancestors(classes: &BTreeMap<String, Class>, actual: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut base = classes[actual].base.clone();
    while let Some(owner) = base {
        base = classes[&owner].base.clone();
        result.push(owner);
    }
    result
}
