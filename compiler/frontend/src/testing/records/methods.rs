use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_10" => Record {
            name: "Cell",
            fields: &[field!("value", "i32", false)],
            methods: &[
                method!("bump(var self) -> () {value+=1;}", false),
                method!("read(self) -> i32 {return value;}", false),
                method!("twice(var self) -> () {bump();self.bump();}", false),
            ],
            ..EMPTY
        },
        "record_11" => Record {
            name: "Cell",
            fields: &[field!("value", "i32", false)],
            methods: &[
                method!("bump(var self) -> () {self.value+=1;}", false),
                method!("read(self) -> i32 {return self.value;}", false),
            ],
            copy: true,
            ..EMPTY
        },
        "record_12" => Record {
            name: "Box",
            generics: "<T>",
            fields: &[field!("value", "T", false)],
            methods: &[method!("read(self) -> i32 {return 7;}", false)],
            ..EMPTY
        },
        "record_13" => Record {
            name: "Box",
            generics: "<T:Copy>",
            fields: &[field!("value", "T", false)],
            methods: &[method!("get(self) -> T {return value;}", false)],
            copy: true,
            ..EMPTY
        },
        "record_14" => Record {
            name: "Cell",
            fields: &[field!("value", "i32", false)],
            methods: &[method!(
                "read(self) -> i32 {let value=4;return value+self.value;}",
                false
            )],
            ..EMPTY
        },
        "record_16" => Record {
            name: "Box",
            generics: "<T:Copy>",
            fields: &[field!("value", "T", false)],
            methods: &[
                method!(
                    "create(value:T) -> Box<T> {return Box<T> {value: value};}",
                    false
                ),
                method!("extract(value:Box<T>) -> T {return value.value;}", false),
            ],
            copy: true,
            ..EMPTY
        },
        "record_33" => Record {
            name: "Cell",
            fields: &[field!("value", "i32", false)],
            methods: &[method!("bump(self) -> () {value+=1;}", false)],
            ..EMPTY
        },
        "record_34" => Record {
            name: "Cell",
            fields: &[field!("value", "i32", false)],
            methods: &[method!("bump(var self) -> () {value+=1;}", false)],
            ..EMPTY
        },
        _ => return None,
    })
}
