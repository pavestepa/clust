use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_118" => Record {
            name: "W",
            fields: &[field!("r", "R", false)],
            ..EMPTY
        },
        _ => return None,
    })
}
