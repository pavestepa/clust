use core.option.{Option, is_some, unwrap_or};
use core.result.{Result, is_ok};

fn divide(numerator: i32, denominator: i32) -> Result<i32, i32> {
    if denominator == 0 {
        return Result.Err(1);
    }
    return Result.Ok(numerator / denominator);
}

fn main() -> i32 {
    let value = Option<i32>.Some(14);
    let result = divide(unwrap_or(value, 0), 2);
    if !is_ok(result) {
        return 1;
    }
    return match result {
        Result.Ok(answer) => answer,
        Result.Err(code) => code,
    };
}
