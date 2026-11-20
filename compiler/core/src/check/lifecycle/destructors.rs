pub(in super::super) fn is_destructor(name: &str) -> bool {
    name.rsplit_once(".$drop").is_some_and(|(_, suffix)| {
        suffix.is_empty() || suffix.starts_with("$g") || suffix.starts_with("$entry")
    })
}
