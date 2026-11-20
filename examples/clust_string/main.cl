use core.string.{String};

fn main() -> i32 {
    var message = String.new("Hello");
    message.append(", ");
    message.push('🌍');
    println(message);
    assert(message.len() == 11);
    assert(message.char_count() == 8);
    let expected = String.new("Hello, 🌍");
    assert(message.equals(expected));
    return 0;
}
