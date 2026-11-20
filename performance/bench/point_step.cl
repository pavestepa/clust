class Point {
    pub x: i64
    pub y: i64
    new(x: i64, y: i64) { self.x = x; self.y = y; }
}

fn step(var p: Point, i: i64) -> () {
    p.x = p.x + i;
    p.y = p.y ^ p.x;
}

fn main() -> i32 {
    var p = Point.new(1, 2);
    var i: i64 = 0;
    while i < 400000000 {
        step(p, i);
        i += 1;
    }
    return ((p.x ^ p.y) % 251) as i32;
}
