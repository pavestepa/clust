use super::*;
pub(super) fn get(id: &str) -> Option<Record> {
    Some(match id {
        "record_94" => Record {
            name: "Fixed",
            generics: "<T,const N:usize>",
            fields: &[field!("values", "{T;N}", false)],
            copy: true,
            ..EMPTY
        },
        _ => return None,
    })
}
