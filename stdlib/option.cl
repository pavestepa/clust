// Absence is explicit; Some owns its payload subject to ordinary inference.
pub enum Option<T> {
    None,
    Some(T),
}

pub fn is_some<T>(value: Option<T>) -> bool {
    return match value {
        Option.Some(_) => true,
        Option.None => false,
    };
}

pub fn unwrap_or<T>(value: Option<T>, fallback: T) -> T {
    return match value {
        Option.Some(value) => value,
        Option.None => fallback,
    };
}
