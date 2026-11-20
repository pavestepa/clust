use core.vec.{Vec};
use core.option.{Option};

fn main() -> i32 {
    var buffer = Vec<i32>.new();
    buffer.reserve(3);
    buffer.push(3);
    buffer.push(7);
    return match buffer.pop() {
        Option.Some(value) => value,
        Option.None => 0,
    };
}
