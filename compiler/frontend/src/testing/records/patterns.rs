use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_25" => Record {
            name: "Pair",
            fields: &[field!("x", "i32", false), field!("y", "i32", false)],
            copy: true,
            ..EMPTY
        },
        _ => return None,
    })
}
