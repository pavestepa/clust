use core.vec.{Vec};
use core.option.{Option};

fn main() -> i32 {
    var values = Vec<i32>.new();
    var n = 0;
    while n < 10 {
        values.push(n);
        n += 1;
    }
    assert(values.len() == 10);
    var index: usize = 0;
    var total = 0;
    while index < values.len() {
        total += values.at(index);
        index += 1;
    }
    assert(total == 45);
    return match values.pop() {
        Option.Some(last) => last,
        Option.None => 0,
    };
}
