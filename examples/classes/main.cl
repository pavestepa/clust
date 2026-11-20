class Counter {
    value: i32
    new(initial: i32) { self.value = initial; }
    pub increment(var self) -> () { self.value += 1; }
    pub read(self) -> i32 { return self.value; }
    drop() { self.value = 0; }
}

#[Copy]
class Number {
    pub value: i32
    new(initial: i32) { self.value = initial; }
}

fn relay(counter: Counter) -> Counter { return counter; }

fn main() -> i32 {
    var counter = Counter.new(3);
    counter.increment();
    let moved = relay(counter);
    let original = Number.new(5);
    var independent = original;
    independent.value = 9;
    if same_identity(original, independent) { return 1; }
    return moved.read() + original.value + independent.value;
}
