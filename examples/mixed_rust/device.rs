use std::sync::atomic::{AtomicUsize, Ordering};
static RELEASED: AtomicUsize = AtomicUsize::new(0);
pub struct Handle(String);
impl Drop for Handle {
    fn drop(&mut self) {
        RELEASED.fetch_add(1, Ordering::SeqCst);
    }
}
pub fn make() -> Handle {
    Handle("Clust".into())
}
pub fn read(h: &Handle) -> &str {
    &h.0
}
pub fn append(h: &mut Handle) {
    h.0.push('!');
}
pub fn close(_: Handle) {}
pub fn released() -> usize {
    RELEASED.load(Ordering::SeqCst)
}
