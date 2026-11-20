class Cell {
    pub value: i32
    new(value: i32) { self.value = value; }
}

class Pair {
    pub left: Cell
    pub right: Cell
    new(left: Cell, right: Cell) { self.left = left; self.right = right; }

    pub first(var self) -> Cell {
        return self.left;
    }
}

fn identity<T>(value: T) -> T {
    return value;
}

fn relay<T>(value: T) -> T {
    return identity(value);
}

fn main() -> i32 {
    var pair = Pair.new(Cell.new(5), Cell.new(9));
    let ref var first = pair.first();
    first.value += 2;

    let ref source = pair.left;
    let result = relay(source);
    return result.value + pair.right.value;
}
