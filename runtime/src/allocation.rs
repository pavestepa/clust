use std::{
    alloc::{alloc, dealloc, Layout},
    ptr,
};
/// Allocate target storage, returning null for invalid layout or OOM.
pub fn allocate(size: usize, alignment: usize) -> *mut u8 {
    let Ok(layout) = Layout::from_size_align(size, alignment) else {
        return ptr::null_mut();
    };
    if size == 0 {
        return alignment as *mut u8;
    }
    // SAFETY: layout was validated, and size is nonzero.
    let pointer = unsafe { alloc(layout) };
    #[cfg(feature = "diagnostic")]
    if !pointer.is_null() {
        registry::insert(pointer, size, alignment);
    }
    pointer
}
/// # Safety
/// Pointer must be a live allocation from allocate with the identical layout,
/// and no live access may remain after this call. ZST sentinels are not freed.
pub unsafe fn deallocate(pointer: *mut u8, size: usize, alignment: usize) {
    if size == 0 {
        return;
    }
    let layout = Layout::from_size_align(size, alignment).expect("invalid allocation layout");
    #[cfg(feature = "diagnostic")]
    registry::remove(pointer, size, alignment);
    unsafe {
        dealloc(pointer, layout);
    }
}

#[cfg(feature = "diagnostic")]
mod registry {
    use std::{collections::BTreeMap, sync::Mutex};
    static LIVE: Mutex<BTreeMap<usize, (usize, usize)>> = Mutex::new(BTreeMap::new());
    pub fn insert(pointer: *mut u8, size: usize, align: usize) {
        assert!(LIVE
            .lock()
            .unwrap()
            .insert(pointer as usize, (size, align))
            .is_none());
    }
    pub fn remove(pointer: *mut u8, size: usize, align: usize) {
        if LIVE.lock().unwrap().remove(&(pointer as usize)) != Some((size, align)) {
            std::process::abort();
        }
    }
    pub fn count() -> usize {
        LIVE.lock().unwrap().len()
    }
}
#[cfg(feature = "diagnostic")]
pub fn live_allocations() -> usize {
    registry::count()
}
