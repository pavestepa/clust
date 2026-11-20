class Item {
    pub value: i32
    new(value: i32) { self.value = value; }
    drop() {}
}

class Holder<T> {
    value: T
    new(source: T) { let value = source; self.value = value; }
    pub get(self) -> T { return self.value; }
    drop() {}
}

fn main() -> i32 {
    let item = Item.new(7);
    let ref view = item;
    let holder = Holder<Item>.new(view);
    let result = holder.get();
    return item.value + result.value;
}
