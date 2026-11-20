use core.weak.{Weak};
use core.option.{Option};
class Animal {
    new() {}
    pub read(self)->i32 {return 1;}
}
class Cat:Animal {
    new() {super.new();}
    pub read(self)->i32 {return 7;}
    drop() {println(9);}
}
fn read(value:Animal)->i32 {return value.read();}
fn main()->i32 {
    let cat=Cat.new();
    let handle=Weak<Animal>.new(cat);
    let base:Animal=cat;
    assert(same_identity(base,cat));
    let through_base=Weak<Animal>.new(base);
    let a=match handle.with<i32>(read) {Option.Some(value)=>value,Option.None=>0};
    let b=match through_base.with<i32>(read) {Option.Some(value)=>value,Option.None=>0};
    println(1);
    return a+b;
}
