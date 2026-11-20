use core.vec.{Vec};

interface Read {
    read(self) -> i32;
    twice(self) -> i32 { return self.read() + self.read(); }
}
class First: Read {
    new() {}
    pub read(self) -> i32 { return 3; }
}
class Second: Read {
    new() {}
    pub read(self) -> i32 { return 4; }
}
fn make(first: bool) -> dyn Read {
    return if first { First.new() } else { Second.new() };
}
class Parent: Read {
    new() {}
    pub read(self) -> i32 { return 1; }
}
class Child: Parent {
    new() { super.new(); }
    pub read(self) -> i32 { return 5; }
}
fn through_base() -> dyn Read {
    let value: Parent = Child.new();
    return value;
}
fn main() -> i32 {
    let inherited = through_base();
    assert(inherited.twice() == 10);
    var values = Vec<dyn Read>.new();
    values.push(make(true));
    values.push(make(false));
    return values.at(0).twice() + values.at(1).twice();
}
