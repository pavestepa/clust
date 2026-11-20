interface Counter {
    read(self) -> i32;
    add(var self, amount: i32);

    advance(var self) -> i32 {
        self.add(2);
        return self.read();
    }
}

class Count: Counter {
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
    var count = Count.new(5);
    return count.advance();
}
