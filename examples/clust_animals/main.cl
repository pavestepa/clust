use core.vec.{Vec};
class Animal {
    new() {}
    pub sound(self)->i32 {return 1;}
    drop() {println(10);}
}
class Cat:Animal {
    new() {super.new();}
    pub sound(self)->i32 {return 2;}
    drop() {println(20);}
}
class Dog:Animal {
    new() {super.new();}
    pub sound(self)->i32 {return 3;}
    drop() {println(30);}
}
fn make(cat:bool)->Animal {
    return if cat {Cat.new()} else {Dog.new()};
}
fn main()->i32 {
    var animals=Vec<Animal>.new();
    animals.push(make(true));
    animals.push(make(false));
    return animals.at(0).sound()+animals.at(1).sound();
}
