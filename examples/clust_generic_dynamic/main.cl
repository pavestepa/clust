interface Read<T:Copy> { read(self)->T; }
class A<T:Copy>:Read<T> {value:T;new(value:T) {self.value=value;} pub read(self)->T {return self.value;}}
class B:Read<u8> {new() {} pub read(self)->u8 {return 3;}}
fn make()->dyn Read<i32> {return A<i32>.new(7);}
fn small()->dyn Read<u8> {return B.new();}
fn main()->i32 {let a=make();let b=small();return a.read()+b.read() as i32;}
