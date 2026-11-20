use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_0" => Record {
            name: "S",
            fields: &[field!("p", "*i32", false)],
            ..EMPTY
        },
        "record_1" => Record {
            name: "S",
            fields: &[field!("n", "i32", false)],
            ..EMPTY
        },
        "record_2" => Record {
            name: "R",
            fields: &[field!("v", "i32", false)],
            drops: &[""],
            ..EMPTY
        },
        "record_3" => Record {
            name: "R",
            fields: &[field!("v", "i32", false)],
            ..EMPTY
        },
        "record_17" => Record {
            name: "Pair",
            fields: &[field!("left", "i32", false), field!("right", "i32", false)],
            copy: true,
            ..EMPTY
        },
        _ => return None,
    })
}
