use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_40" => Record {
            name: "Cell",
            fields: &[field!("value", "i32", false)],
            methods: &[method!(
                "reset(var self) -> () {*self=Cell {value: 2};}",
                false
            )],
            ..EMPTY
        },
        "record_41" => Record {
            name: "Pair",
            fields: &[field!("cell", "Cell", false)],
            methods: &[method!(
                "change(var self) -> () {self.cell.reset();}",
                false
            )],
            ..EMPTY
        },
        "record_133" => Record {
            name: "Node",
            fields: &[field!("next", "Node", false)],
            ..EMPTY
        },
        "record_134" => Record {
            name: "Root",
            fields: &[field!("link", "Link<i32>", false)],
            ..EMPTY
        },
        "record_135" => Record {
            name: "Root",
            fields: &[field!("node", "Node<i32>", false)],
            ..EMPTY
        },
        "record_136" => Record {
            name: "Holder",
            fields: &[
                field!("small", "Cell<i32>", false),
                field!("large", "Cell<i64>", false),
            ],
            ..EMPTY
        },
        "record_137" => Record {
            name: "R",
            drops: &[""],
            copy: true,
            ..EMPTY
        },
        "record_138" => Record {
            name: "R",
            drops: &["", ""],
            ..EMPTY
        },
        "record_139" => Record {
            name: "R",
            fields: &[field!("cell", "Cell", false)],
            drops: &[""],
            ..EMPTY
        },
        "record_140" => Record {
            name: "H",
            fields: &[field!("c", "Cell", false)],
            ..EMPTY
        },
        "record_141" => Record {
            name: "R",
            fields: &[field!("value", "i32", false)],
            methods: &[method!(
                "replace(var self) -> () {*self=R {value: 3};}",
                false
            )],
            drops: &["value+=10;"],
            ..EMPTY
        },
        "record_142" => Record {
            name: "Pair",
            fields: &[field!("a", "R", false), field!("b", "R", false)],
            ..EMPTY
        },
        "record_143" => Record {
            name: "Parent",
            fields: &[
                field!("a", "R", false),
                field!("b", "R", false),
                field!("c", "i32", false),
            ],
            drops: &["let fail=1/0;"],
            ..EMPTY
        },
        "record_144" => Record {
            name: "Fail",
            fields: &[field!("value", "i32", false)],
            drops: &["let failed=1/value;"],
            ..EMPTY
        },
        "record_145" => Record {
            name: "R",
            generics: "<T:Copy>",
            fields: &[field!("value", "T", false)],
            drops: &["let copy=value;"],
            ..EMPTY
        },
        "record_146" => Record {
            name: "R",
            generics: "<T>",
            fields: &[field!("value", "T", false)],
            drops: &["value+=1;"],
            ..EMPTY
        },
        "record_147" => Record {
            name: "P",
            fields: &[field!("a", "R", false), field!("b", "R", false)],
            methods: &[method!("get(self) -> R {return a;}", false)],
            ..EMPTY
        },
        "record_148" => Record {
            name: "S",
            fields: &[field!("c", "C", false)],
            copy: true,
            ..EMPTY
        },
        "record_149" => Record {
            name: "P",
            fields: &[
                field!("value", "i32", false),
                field!("a", "R", false),
                field!("b", "R", false),
            ],
            drops: &["a.value+=10;"],
            ..EMPTY
        },
        "record_150" => Record {
            name: "P",
            fields: &[
                field!("value", "i32", false),
                field!("a", "R", false),
                field!("b", "R", false),
            ],
            drops: &["let fail=1/0;"],
            ..EMPTY
        },
        "record_151" => Record {
            name: "R",
            fields: &[field!("value", "i32", false)],
            drops: &["let fail=1/value;"],
            ..EMPTY
        },
        "record_152" => Record {
            name: "P",
            fields: &[field!("a", "R", false), field!("b", "R", false)],
            ..EMPTY
        },
        "record_153" => Record {
            name: "R",
            fields: &[field!("value", "i32", false)],
            drops: &["value+=10;"],
            ..EMPTY
        },
        "record_154" => Record {
            name: "R",
            fields: &[field!("value", "i32", false)],
            methods: &[method!(
                "replace(var self) -> () {*self=R {value: 3};}",
                false
            )],
            drops: &[""],
            ..EMPTY
        },
        "record_155" => Record {
            name: "P",
            fields: &[
                field!("a", "R", false),
                field!("b", "R", false),
                field!("c", "i32", false),
            ],
            ..EMPTY
        },
        "record_156" => Record {
            name: "F",
            fields: &[field!("value", "i32", false)],
            drops: &["let failure=1/value;"],
            ..EMPTY
        },
        "record_157" => Record {
            name: "A",
            methods: &[method!("read(self) -> i32 {return 1;}", false)],
            ..EMPTY
        },
        "record_158" => Record {
            name: "B",
            interfaces: "A",
            ..EMPTY
        },
        _ => return None,
    })
}
