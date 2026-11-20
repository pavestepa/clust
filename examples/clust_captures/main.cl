interface Read { call(self) -> i32; }
class Item {
    value: i32
    new(value: i32) { self.value = value; }
    pub read(self) -> i32 { return self.value; }
    drop() { println(self.value); }
}
fn make() -> dyn Read {
    let item = Item.new(7);
    return fn() -> i32 { return item.read(); };
}
fn main() -> i32 {
    var count = 0;
    var next = fn() -> i32 { count += 1; return count; };
    assert(next() == 1);
    assert(next() == 2);
    assert(count == 2);
    let item = Item.new(4);
    let read = fn() -> i32 { return item.read(); };
    assert(read() + item.read() == 8);
    let predicate = count == 2;
    let test = fn() -> bool { return predicate; };
    assert(test());
    let escaped = make();
    return escaped();
}
