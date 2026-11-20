use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_96" => Record {
            name: "Holder",
            fields: &[field!("value", "R", false)],
            ..EMPTY
        },
        "record_97" => Record {
            name: "Counter",
            fields: &[field!("value", "i32", false)],
            methods: &[method!(
                "tap(var self,x:R) -> R {value+=1;return x;}",
                false
            )],
            ..EMPTY
        },
        "record_98" => Record {
            name: "G",
            generics: "<T>",
            fields: &[field!("value", "T", false)],
            ..EMPTY
        },
        "record_99" => Record {
            name: "Outer",
            fields: &[field!("h", "Holder", false)],
            ..EMPTY
        },
        "record_100" => Record {
            name: "S",
            fields: &[field!("value", "R", false)],
            methods: &[method!(
                "set(var self,_borrowed_x:R) -> () {let x=_borrowed_x;value=x;}",
                false
            )],
            ..EMPTY
        },
        "record_101" => Record {
            name: "A",
            fields: &[field!("xs", "{R;2}", false)],
            ..EMPTY
        },
        "record_102" => Record {
            name: "Mixed",
            fields: &[field!("view", "R", false), field!("owned", "R", false)],
            ..EMPTY
        },
        "record_103" => Record {
            name: "G",
            generics: "<T>",
            fields: &[field!("value", "T", false)],
            methods: &[method!("get(self) -> T {return value;}", false)],
            ..EMPTY
        },
        "record_104" => Record {
            name: "S",
            fields: &[field!("value", "R", false)],
            methods: &[
                method!("read(self) -> i32 {return value.value;}", false),
                method!("get(self) -> R {return value;}", false),
            ],
            ..EMPTY
        },
        "record_105" => Record {
            name: "Plain",
            fields: &[field!("value", "i32", false)],
            ..EMPTY
        },
        "record_106" => Record {
            name: "Box",
            fields: &[field!("value", "C", false)],
            ..EMPTY
        },
        "record_107" => Record {
            name: "F",
            fields: &[field!("value", "i32", false)],
            drops: &["let fail=1/value;"],
            ..EMPTY
        },
        "record_108" => Record {
            name: "Mix",
            fields: &[
                field!("owned", "R", false),
                field!("borrowed", "R", false),
                field!("fail", "i32", false),
            ],
            ..EMPTY
        },
        _ => return None,
    })
}
