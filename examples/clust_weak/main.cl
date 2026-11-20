use core.weak.{Weak};
use core.option.{Option};

class Item {
    pub value: i32
    new(value: i32) {
        self.value = value;
        let during_construction = Weak<Item>.new(self);
        assert(during_construction.is_alive());
    }
    drop() {
        let during_destruction = Weak<Item>.new(self);
        assert(!during_destruction.is_alive());
        println(self.value);
    }
}
fn read(item: Item) -> i32 { return item.value; }
fn expired() -> Weak<Item> {
    let item = Item.new(9);
    return Weak<Item>.new(item);
}
fn main() -> i32 {
    let item = Item.new(7);
    let handle = Weak<Item>.new(item);
    assert(handle.is_alive());
    let value = match handle.with<i32>(read) {
        Option.Some(value) => value,
        Option.None => 0,
    };
    let gone = expired();
    assert(!gone.is_alive());
    assert(match gone.with<i32>(read) {
        Option.Some(_) => false,
        Option.None => true,
    });
    return value;
}
