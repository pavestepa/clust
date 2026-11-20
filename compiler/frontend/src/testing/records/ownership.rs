use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_80" => Record {
            name: "Invalid",
            fields: &[field!("cell", "Cell", false)],
            copy: true,
            ..EMPTY
        },
        _ => return None,
    })
}
