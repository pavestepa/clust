use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_42" => Record {
            name: "S",
            interfaces: "Read",
            fields: &[field!("value", "i32", false)],
            methods: &[method!("read(self) -> i32 {return value;}", false)],
            ..EMPTY
        },
        "record_43" => Record {
            name: "S",
            interfaces: "Read,Other",
            methods: &[
                method!("read(self) -> i32 {return 3;}", false),
                method!("other(self) -> i32 {return 4;}", false),
            ],
            ..EMPTY
        },
        "record_44" => Record {
            name: "S",
            interfaces: "Read",
            fields: &[field!("value", "i32", false)],
            methods: &[method!("read(self) -> i32 {return value;}", false)],
            copy: true,
            ..EMPTY
        },
        "record_45" => Record {
            name: "S",
            interfaces: "Read,Bump",
            fields: &[field!("value", "i32", false)],
            methods: &[
                method!("read(self) -> i32 {return value;}", false),
                method!("bump(var self) -> () {value+=1;}", false),
            ],
            ..EMPTY
        },
        "record_46" => Record {
            name: "S",
            interfaces: "Map<i32,bool>",
            methods: &[method!("map(self,x:i32) -> bool {return x>3;}", false)],
            ..EMPTY
        },
        "record_47" => Record {
            name: "Holder",
            generics: "<T:Read>",
            fields: &[field!("value", "T", false)],
            methods: &[method!("read(self) -> i32 {return value.read();}", false)],
            ..EMPTY
        },
        "record_48" => Record {
            name: "S",
            interfaces: "Read",
            methods: &[method!("read(self) -> i32 {return 7;}", false)],
            ..EMPTY
        },
        "record_49" => Record {
            name: "S",
            interfaces: "Identity",
            methods: &[method!("id<U:Copy>(self,x:U) -> U {return x;}", false)],
            ..EMPTY
        },
        "record_50" => Record {
            name: "S",
            interfaces: "First<2>",
            methods: &[method!("first(self,xs:{i32;2}) -> i32 {return xs[0];}", false)],
            ..EMPTY
        },
        "record_51" => Record {
            name: "S",
            interfaces: "Factory",
            methods: &[method!("make() -> i32 {return 7;}", false)],
            ..EMPTY
        },
        "record_52" => Record {
            name: "S",
            interfaces: "Get<R>",
            fields: &[field!("value", "R", false)],
            methods: &[method!("get(self) -> R {return value;}", false)],
            ..EMPTY
        },
        "record_53" => Record {
            name: "S",
            generics: "<T:Copy>",
            interfaces: "Read",
            fields: &[field!("value", "T", false)],
            methods: &[method!("read(self) -> i32 {return 7;}", false)],
            ..EMPTY
        },
        "record_54" => Record {
            name: "S",
            interfaces: "Get",
            methods: &[method!("get(self) -> i32 {return 7;}", false)],
            ..EMPTY
        },
        "record_55" => Record {
            name: "S",
            interfaces: "Identity",
            methods: &[method!("id<const M:usize>(self,x:{i32;M}) -> i32 {return x[0];}", false)],
            ..EMPTY
        },
        "record_56" => Record {
            name: "S",
            generics: "<T>",
            interfaces: "Factory",
            fields: &[field!("value", "T", false)],
            methods: &[method!("make() -> i32 {return 7;}", false)],
            ..EMPTY
        },
        "record_57" => Record {
            name: "S",
            interfaces: "I,J",
            methods: &[method!("read(self) -> i32 {return 7;}", false)],
            ..EMPTY
        },
        "record_58" => Record {
            name: "Adapter",
            generics: "<T:Get<R>>",
            fields: &[field!("value", "T", false)],
            methods: &[method!("get(self) -> R {return value.get();}", false)],
            ..EMPTY
        },
        "record_59" => Record {
            name: "H",
            generics: "<T:Read>",
            fields: &[field!("value", "T", false)],
            ..EMPTY
        },
        "record_60" => Record {
            name: "U",
            interfaces: "Use<H<S>>",
            methods: &[method!("apply(self,value:H<S>) -> i32 {return value.value.read();}", false)],
            ..EMPTY
        },
        "record_61" => Record {
            name: "Runner",
            interfaces: "Run",
            methods: &[method!("run<A:Use<H<B>>,B:Read>(self,x:A,value:B) -> i32 {return x.apply(H<B> {value: value});}", false)],
            ..EMPTY
        },
        "record_62" => Record {
            name: "S",
            interfaces: "I",
            methods: &[method!("read<const M:usize>(self,xs:{i32;M*M}) -> i32 {return xs[0];}", false)],
            ..EMPTY
        },
        "record_63" => Record {
            name: "S",
            interfaces: "I",
            methods: &[method!("read<U=bool>(self,x:U) -> U {return x;}", false)],
            ..EMPTY
        },
        "record_64" => Record {
            name: "S",
            interfaces: "I",
            ..EMPTY
        },
        "record_65" => Record {
            name: "S",
            interfaces: "I",
            methods: &[method!("read(self) -> i32 {return 1;}", true)],
            ..EMPTY
        },
        "record_66" => Record {
            name: "S",
            interfaces: "I",
            methods: &[method!("read(var self) -> i32 {return 1;}", false)],
            ..EMPTY
        },
        "record_67" => Record {
            name: "S",
            interfaces: "I",
            methods: &[method!("read(self) -> i32 {return 1;}", false)],
            ..EMPTY
        },
        "record_68" => Record {
            name: "S",
            interfaces: "I",
            methods: &[method!("read() -> i32 {return 1;}", false)],
            ..EMPTY
        },
        "record_69" => Record {
            name: "S",
            interfaces: "I",
            methods: &[method!("read(self) -> bool {return true;}", false)],
            ..EMPTY
        },
        "record_70" => Record {
            name: "S",
            interfaces: "I",
            methods: &[method!("read(self,x:i32) -> i32 {return x;}", false)],
            ..EMPTY
        },
        "record_71" => Record {
            name: "S",
            methods: &[method!("read(self) -> i32 {return 1;}", false)],
            ..EMPTY
        },
        "record_72" => Record {
            name: "S",
            interfaces: "I<bool>",
            methods: &[method!("get(self) -> i32 {return 1;}", false)],
            ..EMPTY
        },
        "record_73" => Record {
            name: "S",
            interfaces: "I",
            methods: &[method!("read<U>(self,x:U) -> U {return x;}", false)],
            ..EMPTY
        },
        "record_74" => Record {
            name: "I",
            ..EMPTY
        },
        "record_75" => Record {
            name: "S",
            interfaces: "I,I",
            ..EMPTY
        },
        "record_76" => Record {
            name: "S",
            interfaces: "Missing",
            ..EMPTY
        },
        "record_77" => Record {
            name: "S",
            interfaces: "I",
            methods: &[method!("bump(var self) -> () {}", false)],
            ..EMPTY
        },
        "record_78" => Record {
            name: "S",
            interfaces: "I",
            methods: &[
                method!("read(self) -> i32 {return 1;}", false),
                method!("extra(self) -> i32 {return 2;}", false),
            ],
            ..EMPTY
        },
        "record_79" => Record {
            name: "S",
            interfaces: "I",
            methods: &[method!("read(self) -> i32 {return 1;}", false)],
            copy: true,
            ..EMPTY
        },
_=>return None,
})
}
