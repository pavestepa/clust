use core.vec.{Vec};
use core.string.{String};
class Item {
    pub value: i32
    new(value: i32) { self.value = value; }
    drop() { println(self.value); }
}
fn main() -> i32 {
    var values = Vec<i32>.new();
    values.push(1);
    values.push(2);
    values.push(3);
    values[1] = 4;
    var total = 0;
    for value in values {
        if value == 1 { continue; }
        total += value;
        if value == 3 { break; }
    }
    assert(values[1] == 4);
    var items = Vec<Item>.new();
    items.push(Item.new(8));
    items[0] = Item.new(9);
    for item in items { total += item.value; }
    var text = String.new("a");
    text.push('é');
    text.push('中');
    text.push('🌍');
    assert(text.len() == 10);
    assert(text.char_count() == 4);
    let expected = String.new("aé中🌍");
    assert(text.equals(expected));
    assert(text[0] == 97);
    return total;
}
