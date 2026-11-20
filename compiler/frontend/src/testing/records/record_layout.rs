use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_18" => Record {
            name: "Boxed",
            fields: &[field!("pair", "Pair", false)],
            copy: true,
            ..EMPTY
        },
        "record_19" => Record {
            name: "A",
            fields: &[field!("a", "A", false)],
            copy: true,
            ..EMPTY
        },
        "record_20" => Record {
            name: "A",
            fields: &[field!("b", "B", false)],
            copy: true,
            ..EMPTY
        },
        "record_21" => Record {
            name: "B",
            fields: &[field!("a", "{A;1}", false)],
            copy: true,
            ..EMPTY
        },
        "record_22" => Record {
            name: "A",
            fields: &[field!("x", "i32", false), field!("x", "i32", false)],
            copy: true,
            ..EMPTY
        },
        "record_23" => Record {
            name: "A",
            fields: &[field!("x", "i32", false)],
            copy: true,
            ..EMPTY
        },
        "record_24" => Record {
            name: "B",
            fields: &[field!("x", "i32", false)],
            copy: true,
            ..EMPTY
        },
        _ => return None,
    })
}
