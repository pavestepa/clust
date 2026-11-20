use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_119" => Record {
            name: "H",
            fields: &[field!("x", "R", false)],
            ..EMPTY
        },
        _ => return None,
    })
}
