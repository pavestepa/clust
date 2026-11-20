class Vault<T: Copy> {
    value: T
    new(initial: T) { self.value = initial; }
    identity<U: Copy>(input: U) -> U { return input; }
    pub read(self) -> T { return self.value; }
    pub transformer() -> (i32) => i32 { return Vault<T>.identity<i32>; }
    drop() { let final_value = self.value; }
}

fn main() -> i32 {
    let vault = Vault<i32>.new(3);
    let transform = Vault<i32>.transformer();
    return vault.read() + transform(4);
}
