use core.intrinsic.{weak_create, weak_exists, weak_read};
use core.option.{Option};

// Identity metadata never owns the target. Access is scoped to a readonly,
// noncapturing callback with a scalar result, so no target borrow can escape.
pub class Weak<T> {
    token: (usize, usize)
    new(target: T) { self.token = weak_create<T>(target); }
    pub is_alive(self) -> bool { return weak_exists<T>(self.token); }
    pub with<R: Copy>(self, callback: (T) => R) -> Option<R> {
        return weak_read<T, R>(self.token, callback);
    }
}
