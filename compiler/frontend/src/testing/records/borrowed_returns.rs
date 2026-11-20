use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_5" => Record {
            name: "Cell",
            fields: &[field!("value", "i32", false)],
            ..EMPTY
        },
        "record_15" => Record {
            name: "Pair",
            fields: &[
                field!("a", "Cell", false),
                field!("b", "Cell", false),
            ],
            ..EMPTY
        },
        "record_26" => Record {
            name: "Pair",
            fields: &[
                field!("a", "Cell", false),
                field!("b", "Cell", false),
            ],
            methods: &[method!("left(var self) -> Cell {return a;}", false)],
            ..EMPTY
        },
        "record_27" => Record {
            name: "Holder",
            fields: &[field!("value", "E", false)],
            methods: &[method!("get(self) -> Cell {return match value {E.Some(x)=>x,E.None=>{while true {};return get();}};}", false)],
            ..EMPTY
        },
        "record_28" => Record {
            name: "Box",
            generics: "<T>",
            fields: &[field!("value", "T", false)],
            methods: &[method!("get(self) -> T {return id(value);}", false)],
            ..EMPTY
        },
        "record_29" => Record {
            name: "Pair",
            fields: &[
                field!("a", "Cell", false),
                field!("b", "Cell", false),
            ],
            methods: &[method!("get(var self) -> Cell {let ref x=a;return x;}", false)],
            ..EMPTY
        },
        "record_30" => Record {
            name: "Holder",
            fields: &[field!("a", "Cell", false)],
            methods: &[method!("get(self) -> Cell {return a;}", false)],
            ..EMPTY
        },
        "record_31" => Record {
            name: "R",
            fields: &[field!("value", "i32", false)],
            ..EMPTY
        },
        "record_32" => Record {
            name: "R",
            fields: &[field!("value", "i32", false)],
            methods: &[method!("get(self) -> R {return self;}", false)],
            ..EMPTY
        },
_=>return None,
})
}
