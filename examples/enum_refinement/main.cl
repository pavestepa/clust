class Resource {
    pub value: i32
    new(value: i32) { self.value = value; }
    drop() {}
}

enum Choice {
    Item(Resource),
    Empty,
}

fn read(choice: Choice, fallback: i32) -> i32 {
    return match choice {
        Choice.Item(value) => value.value,
        Choice.Empty => fallback,
    };
}

fn main() -> i32 {
    var owner = Resource.new(7);
    let ref view = owner;
    var choice = Choice.Item(view);
    choice = Choice.Empty;
    owner = Resource.new(3);
    return read(choice, owner.value);
}
