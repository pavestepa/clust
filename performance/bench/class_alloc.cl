class Node {
    value: i32
    new(initial: i32) { self.value = initial; }
    pub read(self) -> i32 { return self.value; }
}

fn main() -> i32 {
    var total: i64 = 0;
    var i: i32 = 0;
    while i < 20000000 {
        let node = Node.new(i);
        total += node.read() as i64;
        i += 1;
    }
    return (total % 251) as i32;
}
