use core.vec.{Vec};
class Animal<T:Copy> {
    pub value:T
    new(value:T) {self.value=value;}
    hidden(self)->T {return self.value;}
    pub read(self)->T {return self.hidden();}
}
class Cat<U:Copy>:Animal<U> {
    new(value:U) {super.new(value);}
    pub read(self)->U {return super.read();}
}
class Dog<V:Copy>:Animal<V> {
    new(value:V) {super.new(value);}
}
fn read<T:Copy>(value:Animal<T>)->T {return value.read();}
fn main()->i32 {
    var animals=Vec<Animal<i32>>.new();
    animals.push(Cat<i32>.new(3));
    animals.push(Dog<i32>.new(4));
    let concrete=Cat<i32>.new(2);let copied=read(concrete);assert(copied==2);var result=0;
    for animal in animals {result+=read(animal);}
    return result;
}
