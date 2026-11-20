class Resource<T: Copy> {
    value: T
    new(value: T) { self.value = value; }
    drop() {
        let final_value = self.value;
    }
}

fn main() -> i32 {
    var first = Resource<i32>.new(1);
    let second = Resource<i32>.new(2);
    first = Resource<i32>.new(3);
    // Replacement destroys 1. Scope exit destroys 3, then 2.
    return 0;
}
