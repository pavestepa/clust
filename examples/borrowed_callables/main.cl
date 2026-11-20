class Item {
    pub value: i32
    new(value: i32) { self.value = value; }
    drop() {}
}

fn identity<T>(value: T) -> T { return value; }

fn apply<T>(function: (T) => T, value: T) -> T { return function(value); }

fn main() -> i32 {
    let item = Item.new(7);
    let function: (Item) => Item = identity;
    let result = apply(function, item);
    return item.value + result.value;
}
