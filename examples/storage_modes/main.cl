class Resource {
    pub value: i32
    new(value: i32) { self.value = value; }
    drop() {}
}

class Holder {
    pub value: Resource
    new(value: Resource) { self.value = value; }
}

fn main() -> i32 {
    let first = Resource.new(7);
    let next = Resource.new(3);
    let ref view = next;
    var holder = Holder.new(first);
    holder.value = view;
    let third = Resource.new(5);
    let array = {third, holder.value};
    return holder.value.value + array[0].value + array[1].value;
}
