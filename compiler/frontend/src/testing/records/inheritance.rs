use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_95" => Record { name: "A", ..EMPTY },
        _ => return None,
    })
}
