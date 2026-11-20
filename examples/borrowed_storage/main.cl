class Resource {
    pub value: i32
    new(value: i32) { self.value = value; }
    drop() {}
}

class Holder {
    pub value: Resource
    new(value: Resource) { self.value = value; }
}

enum Choice {
    Some(Resource),
    None,
}

fn relay<T>(value: T) -> T {
    return value;
}

fn main() -> i32 {
    let owner = Resource.new(7);
    let ref view = owner;
    let holder = Holder.new(Resource.new(7));
    let array = {view, view};
    let tuple = (view, Resource.new(3));
    let wrapped = Choice.Some(view);
    let choice = relay(wrapped);
    let extracted = match choice {
        Choice.Some(value) => value.value,
        Choice.None => 0,
    };
    return holder.value.value + array[1].value + tuple.1.value + extracted;
}
