// Explicit import from the Rust standard library; no Cargo dependency needed.
#[rust("std.cmp.max")]
extern "Rust" fn maximum(left: i32, right: i32) -> i32;

fn main() -> i32 {
    return maximum(3, 7);
}
