#[rust("i32.unchecked_add")]
#[safety("The sum must be representable as i32.")]
extern "Rust" unsafe fn unchecked_add(left: i32, right: i32) -> i32;

fn main() -> i32 {
    let add = unchecked_add;
    return add(3, 4);
}
