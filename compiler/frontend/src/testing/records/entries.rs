use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_6" => Record {
            name: "Hold",
            fields: &[field!("cell", "Cell", false)],
            ..EMPTY
        },
        _ => return None,
    })
}
