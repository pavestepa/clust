use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_120" => Record { name: "U", ..EMPTY },
        "record_121" => Record {
            name: "Holder",
            generics: "<T>",
            fields: &[field!("value", "T", false)],
            ..EMPTY
        },
        "record_122" => Record {
            name: "Box",
            generics: "<T:Copy>",
            fields: &[field!("value", "T", false)],
            methods: &[
                method!(
                    "create(x:T) -> Box<T> {let ignored=value();return Box<T> {value: x};}",
                    false
                ),
                method!("secret() -> i32 {return 4;}", true),
            ],
            copy: true,
            ..EMPTY
        },
        "record_123" => Record {
            name: "R",
            generics: "<T:Copy>",
            fields: &[field!("value", "T", true)],
            methods: &[
                method!("make(value:T) -> R<T> {return R<T> {value: value};}", false),
                method!("read(self) -> T {return value;}", false),
            ],
            drops: &["let final_value=value;"],
            ..EMPTY
        },
        _ => return None,
    })
}
