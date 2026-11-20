use core.vec.{Vec};
use core.option.{Option};
class Item {
    pub value: i32
    new(n: i32) { self.value = n; }
    drop() {}
}

fn fetch() -> Option<Item> {
    var buffer = Vec<Item>.new();
    buffer.push(Item.new(3));
    buffer.push(Item.new(7));
    return buffer.pop();
}

fn main() -> i32 {
    return match fetch() {
        Option.Some(item) => item.value,
        Option.None => 0,
    };
}
