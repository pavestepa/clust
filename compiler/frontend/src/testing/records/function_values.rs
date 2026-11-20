use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_7" => Record {
            name: "Math",
            methods: &[method!("add(n:i32) -> i32 {return n+2;}", false)],
            ..EMPTY
        },
        "record_8" => Record {
            name: "Handler",
            fields: &[field!("run", "(i32)=>i32", false)],
            methods: &[method!("invoke(self) -> i32 {return run(4);}", false)],
            ..EMPTY
        },
        "record_9" => Record {
            name: "Handler",
            fields: &[field!("run", "(i32)=>i32", false)],
            copy: true,
            ..EMPTY
        },
        _ => return None,
    })
}
