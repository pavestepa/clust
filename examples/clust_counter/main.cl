class Counter {
    value: i32
    new(value: i32) {
        self.value = value;
    }
    pub read(self) -> i32 {
        return self.value;
    }
    pub add(var self, amount: i32) {
        self.value += amount;
    }
}
fn main() -> i32 {
    var counter = Counter.new(2);
    counter.add(3);
    return counter.read();
}
