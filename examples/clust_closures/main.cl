interface Read {
    call(self) -> i32;
}
fn make(value: i32) -> dyn Read {
    return fn() -> i32 { return value; };
}
fn main() -> i32 {
    let offset = 3;
    let add = fn(value: i32) -> i32 { return value + offset; };
    var count = 0;
    var next = fn() -> i32 { count += 1; return count; };
    assert(next() == 1);
    assert(next() == 2);
    let saved = make(4);
    return add(saved());
}
