interface Read { read(self) -> i32; }

interface Bump { bump(var self); }

class Counter: Read, Bump {
    value: i32
    new(value: i32) { self.value = value; }
    pub read(self) -> i32 { return self.value; }
    pub bump(var self) { self.value += 1; }
}

fn advance<T: Read & Bump>(var value: T) -> i32 {
    value.bump();
    return value.read();
}

fn main() -> i32 {
    var counter = Counter.new(3);
    let current = advance(counter);
    return current + counter.read();
}
