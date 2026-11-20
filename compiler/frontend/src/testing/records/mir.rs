use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_124" => Record {
            name: "S",
            fields: &[field!("a", "i32", false), field!("b", "i32", false)],
            copy: true,
            ..EMPTY
        },
        _ => return None,
    })
}
