use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_81" => Record {
            name: "Item",
            fields: &[field!("v", "i32", false)],
            drops: &[""],
            ..EMPTY
        },
        "record_83" => Record {
            name: "Pair",
            fields: &[field!("a", "i32", false), field!("b", "i32", false)],
            copy: true,
            ..EMPTY
        },
        "record_84" => Record {
            name: "Pair",
            fields: &[field!("a", "u32", false), field!("b", "u32", false)],
            copy: true,
            ..EMPTY
        },
        "record_85" => Record {
            name: "Set",
            fields: &[field!("p", "*var i32", false)],
            drops: &["if trusted_region() {*p=7}"],
            ..EMPTY
        },
        "record_86" => Record {
            name: "R",
            fields: &[field!("v", "i32", false)],
            drops: &["let bad=1/0;"],
            ..EMPTY
        },
        "record_87" => Record {
            name: "Z",
            drops: &[""],
            ..EMPTY
        },
        "record_88" => Record {
            name: "R",
            fields: &[field!("n", "i32", false)],
            ..EMPTY
        },
        "record_89" => Record {
            name: "S",
            fields: &[field!("r", "R", false)],
            ..EMPTY
        },
        "record_90" => Record {
            name: "Item",
            fields: &[field!("payload", "RawStorage<i32>", false)],
            ..EMPTY
        },
        "record_91" => Record {
            name: "Item",
            fields: &[field!("v", "i32", false)],
            ..EMPTY
        },
        _ => return None,
    })
}
