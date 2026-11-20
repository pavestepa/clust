class Resource {
    pub value: i32
    new(value: i32) { self.value = value; }
    drop() {}
}

class Holder<T> {
    pub value: T
    new(value: T) { self.value = value; }
}

fn set<T>(var holder: Holder<T>, source: T) -> () {
    let value = source;
    holder.value = value;
}

fn main() -> i32 {
    let first = Resource.new(7);
    let second = Resource.new(3);
    let ref second_view = second;
    var holder = Holder<Resource>.new(first);
    let old_value = holder.value.value;
    set(holder, second_view);
    return old_value + holder.value.value;
}
