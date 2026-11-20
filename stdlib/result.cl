// Recoverable failure carries an explicit error value.
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

pub fn is_ok<T, E>(value: Result<T, E>) -> bool {
    return match value {
        Result.Ok(_) => true,
        Result.Err(_) => false,
    };
}
