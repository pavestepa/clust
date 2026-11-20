use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_125" => Record {
            name: "S",
            methods: &[method!(
                "f(self) -> i32 {requires_unsafe_caller();return 7;}",
                false
            )],
            ..EMPTY
        },
        "record_126" => Record {
            name: "S",
            interfaces: "I",
            methods: &[method!(
                "f(self) -> i32 {requires_unsafe_caller();return 1;}",
                false
            )],
            ..EMPTY
        },
        "record_127" => Record {
            name: "R",
            fields: &[field!("x", "i32", false)],
            ..EMPTY
        },
        "record_128" => Record {
            name: "H",
            fields: &[field!("r", "R", false)],
            ..EMPTY
        },
        "record_129" => Record {
            name: "S",
            methods: &[method!(
                "f(self) -> i32 {requires_unsafe_caller();return 9;}",
                false
            )],
            ..EMPTY
        },
        "record_130" => Record {
            name: "R",
            fields: &[field!("x", "i32", false)],
            drops: &[""],
            ..EMPTY
        },
        "record_131" => Record {
            name: "S",
            fields: &[field!("p", "unsafe (i32)=>i32", false)],
            ..EMPTY
        },
        "record_132" => Record {
            name: "S",
            methods: &[method!(
                "f(n:i32) -> i32 {requires_unsafe_caller();return n;}",
                false
            )],
            ..EMPTY
        },
        _ => return None,
    })
}
