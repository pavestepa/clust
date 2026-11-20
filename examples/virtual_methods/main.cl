class Counter {
    value: i32
    new(initial: i32) { self.value = initial; }
    pub read(self) -> i32 { return self.value; }
}

class DoubleCounter: Counter {
    new(initial: i32) { super.new(initial); }
    pub read(self) -> i32 { return super.read() * 2; }
}

fn read_counter(counter: Counter) -> i32 {
    return counter.read();
}

fn main() -> i32 {
    return read_counter(DoubleCounter.new(4));
}
