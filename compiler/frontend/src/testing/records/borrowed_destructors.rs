use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_4" => Record {
            name: "R",
            fields: &[field!("value", "i32", false)],
            drops: &[""],
            ..EMPTY
        },
        "record_35" => Record {
            name: "D",
            fields: &[field!("value", "R", false)],
            methods: &[method!("get(self) -> R {return value;}", false)],
            drops: &["let valid=1/(value.value-6);"],
            ..EMPTY
        },
        "record_36" => Record {
            name: "H",
            fields: &[field!("value", "R", false)],
            ..EMPTY
        },
        "record_37" => Record {
            name: "S",
            fields: &[field!("value", "R", false)],
            methods: &[method!("get(self) -> R {return value;}", false)],
            ..EMPTY
        },
        "record_38" => Record {
            name: "G",
            generics: "<T>",
            fields: &[field!("tag", "T", false), field!("value", "R", false)],
            drops: &["let valid=1/(value.value-6);"],
            ..EMPTY
        },
        "record_39" => Record {
            name: "Outer",
            fields: &[field!("d", "D", false)],
            ..EMPTY
        },
        _ => return None,
    })
}
