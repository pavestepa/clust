use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_109" => Record {
            name: "S",
            fields: &[field!("value", "i32", true)],
            methods: &[
                method!("make(n:i32) -> S {return S {value: n};}", false),
                method!("read(self) -> i32 {return value;}", false),
            ],
            ..EMPTY
        },
        "record_110" => Record {
            name: "S",
            fields: &[field!("value", "i32", true)],
            methods: &[
                method!("make(n:i32) -> S {return S {value: n};}", false),
                method!("read(self) -> i32 {let S {value:n}=self;return n;}", false),
            ],
            ..EMPTY
        },
        "record_111" => Record {
            name: "S",
            methods: &[
                method!("secret() -> i32 {return 7;}", true),
                method!("expose() -> ()=>i32 {return S.secret;}", false),
            ],
            ..EMPTY
        },
        "record_112" => Record {
            name: "S",
            generics: "<T:Copy>",
            fields: &[field!("value", "T", true)],
            methods: &[
                method!("make(n:T) -> S<T> {return S<T> {value: n};}", false),
                method!("secret<U:Copy>(x:U) -> U {return x;}", true),
                method!(
                    "pass<U:Copy>(x:U) -> U {let f:(U)=>U=S<T>.secret<U>;return f(x);}",
                    false
                ),
                method!("read(self) -> T {return value;}", false),
            ],
            ..EMPTY
        },
        "record_113" => Record {
            name: "S",
            methods: &[method!("read() -> i32 {return helper();}", false)],
            ..EMPTY
        },
        "record_114" => Record {
            name: "S",
            fields: &[
                field!("hidden", "i32", true),
                field!("visible", "i32", false),
            ],
            methods: &[method!(
                "make() -> S {return S {hidden: 3, visible: 7};}",
                false
            )],
            ..EMPTY
        },
        "record_115" => Record {
            name: "S",
            fields: &[field!("value", "i32", true)],
            ..EMPTY
        },
        "record_116" => Record {
            name: "S",
            fields: &[field!("value", "i32", true)],
            methods: &[method!("make() -> S {return S {value: 7};}", false)],
            ..EMPTY
        },
        "record_117" => Record {
            name: "D",
            methods: &[method!("read(s:S) -> i32 {return s.value;}", false)],
            ..EMPTY
        },
        _ => return None,
    })
}
