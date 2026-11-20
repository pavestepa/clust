use core.weak.{Weak};
use core.option.{Option};
class Parent {
    pub value:i32
    child:Option<Child>
    new() {self.value=7;self.child=Option.None;}
    pub attach(var self) {self.child=Option.Some(Child.new(Weak<Parent>.new(self)));}
    pub read(self)->i32 {return match self.child {Option.Some(child)=>child.read(),Option.None=>0};}
    drop() {println(1);}
}
class Child {
    parent:Weak<Parent>
    new(parent:Weak<Parent>) {self.parent=parent;}
    pub read(self)->i32 {return match self.parent.with<i32>(read_parent) {Option.Some(value)=>value,Option.None=>0};}
    drop() {assert(!self.parent.is_alive());println(2);}
}
fn read_parent(parent:Parent)->i32 {return parent.value;}
fn main()->i32 {var parent=Parent.new();parent.attach();return parent.read();}
