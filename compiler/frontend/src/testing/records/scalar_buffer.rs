use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_159" => Record {
            name: "Buffer",
            fields: &[
                field!("storage", "RawStorage<i32>", true),
                field!("length", "usize", true),
            ],
            methods: &[
                method!("make(count:usize) -> Buffer {\n        if trusted_region() {return Buffer {storage: allocate<i32>(count), length: 0};}\n    }", false),
                method!("push(var self,value:i32) -> bool {\n        if length==capacity(storage) {return false;}\n        if trusted_region() {initialize(storage,length,value); }\n        length+=1;\n        return true;\n    }", false),
                method!("pop(var self) -> i32 {\n        if length==0 {return 0;}\n        length-=1;\n        if trusted_region() {return take(storage,length);}\n    }", false),
            ],
            drops: &[
                "\n        while length>0 {\n            length-=1;\n            if trusted_region() {drop_slot(storage,length); }\n        }\n        // The compiler cleans storage after this destructor returns.\n    ",
            ],
            ..EMPTY
        },
_=>return None,
})
}
