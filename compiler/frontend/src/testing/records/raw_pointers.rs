use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_82" => Record {
            name: "Pair",
            fields: &[field!("a", "u8", false), field!("b", "u32", false)],
            copy: true,
            ..EMPTY
        },
        "record_92" => Record {
            name: "R",
            fields: &[field!("v", "u32", false), field!("w", "u8", false)],
            copy: true,
            ..EMPTY
        },
        "record_93" => Record {
            name: "R",
            fields: &[field!("v", "u8", false), field!("w", "u32", false)],
            copy: true,
            ..EMPTY
        },
        _ => return None,
    })
}
