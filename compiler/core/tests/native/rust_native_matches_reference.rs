use super::*;

#[test]
fn rust_native_matches_reference_at_both_optimization_levels() {
    let cases=[
        "pub fn main() -> i32 {let a={1,7};let n:isize=1;if trusted_region() {return *(&a[0]+n);}}",
        "pub fn main() -> i32 {let n=7;let p=&n;let ref v=p;let q:*i32=*v;if trusted_region() {return *q;}}",
        " pub fn read(p:*i32) -> i32 {requires_unsafe_caller();if trusted_region() {return *p;}} pub fn main() -> i32 {let n=7;let f:unsafe (*i32)=>i32=read;if trusted_region() {return f(&n);}}",

        include_str!("../fixtures/raw_pointers.fixture"),
        "pub fn main() -> i32 {var n=2;let p=&var n;let q=p;if trusted_region() {*q=7;return *p;}}",
        "pub fn identity<T:Copy>(p:*T) -> *T {return p;} pub fn main() -> i32 {let n=7;let p=identity(&n);if trusted_region() {return *p;}}",
        "use compiler_fixtures.record_0.{S}; pub fn main() -> i32 {let n=7;let s=S {p: &n};if trusted_region() {return *s.p;}}",
        "pub fn main() -> i32 {let a={7,3};let p=&a[0];if trusted_region() {let end=p+2;let first=end+(-2);return *first;}}",
        "use compiler_fixtures.record_1.{S}; pub fn main() -> i32 {var s=S {n: 1};let p=&var s.n;if trusted_region() {*p=7};return s.n;}",
        "pub class C {pub n:i32;new() {n=1;}} pub fn main() -> i32 {var c=C.new();let p=&var c.n;if trusted_region() {*p=7};return c.n;}",
        "pub fn main() -> i32 {var n=1;let ref var v=n;let p=&var v;if trusted_region() {*p=7};return n;}",
        "pub fn main() -> i32 {var b=false;let p=&var b;if trusted_region() {*p=true;if *p {return 7;}};return 0;}",
        include_str!("../fixtures/unsafe_callable.fixture"),
        "pub fn main() -> i32 {loop {loop {break;};return 7;}}",
        "use compiler_fixtures.record_2.{R}; pub fn main() -> i32 {var n=0;loop {let r=R {v: n};n+=1;if n<7 {continue;};break;};return n;}",
        "use compiler_fixtures.record_3.{R};  pub fn view(x:R) -> R {requires_unsafe_caller();return x;} pub fn main() -> i32 {let r=R {v: 7};let f:unsafe (R)=>R=view;if trusted_region() {let a=f(r);return a.v;}}",
        "pub fn fib(n:i32) -> i32 {if n<2 {return n;};return fib(n-1)+fib(n-2);} pub fn main() -> i32 {return fib(12);}",
        "pub fn main() -> i32 {var n:u64=27;var steps=0;while n!=1 {if n%2==0 {n=n/2} else {n=n*3+1};steps+=1;};return steps;}",
        "pub fn odd(n:i32) -> bool {if n==0 {return false;};return even(n-1);} pub fn even(n:i32) -> bool {if n==0 {return true;};return odd(n-1);} pub fn main() -> i32 {if odd(13) {return 7;};return 0;}",
        "pub fn main() -> i32 {var n:i8=127;n+=1;return n as i32;}",
        "pub fn main() -> i32 {let n:u128=340282366920938463463374607431768211455;return (n+1) as i32;}",
        "pub class C {pub value:i32;new(n:i32) {value=n;}} pub fn main() -> i32 {var c=C.new(1);var n=0;while n<3 {c=C.new(n);n+=1;};return c.value;}",
        "pub class C {pub value:i32;new(n:i32) {value=n;}} pub fn main() -> i32 {var n=0;var sum=0;while n<9 {let c=C.new(n);n+=1;if n==2 {continue;};sum+=c.value;if n==7 {break;}};return sum;}",
        "pub class C {pub value:i32;new(n:i32) {value=12/n;}} pub fn main() -> i32 {let c=C.new(0);return c.value;}",
        // NMM §25.1: sequential class reborrows restore each parent.
        "pub class Counter {pub value:i32;new(initial:i32) {self.value=initial;};pub read(self) -> i32 {return self.value;};pub increment(var self) -> () {self.value+=1;}} pub fn advance(var counter:Counter) -> () {counter.increment();} pub fn main() -> i32 {var owner=Counter.new(10);let initial=owner.read();var first=owner;first.increment();var second=first;advance(second);first.increment();owner.increment();let result=owner;return result.read()-initial;}",
        // N13: rebinding a view must not destroy its previous target.
        "use compiler_fixtures.record_4.{R}; pub fn main() -> i32 {var a=R {value: 1};var b=R {value: 2};var alias=a;alias=b;alias.value+=1;return a.value+b.value;}",
        "pub fn identity<T>(x:T) -> T {return x;} pub fn main() -> i32 {let f:(i32)=>i32=identity;return f(9);}",
        "pub fn identity<T>(x:T) -> T {return x;} pub fn main() -> i32 {let f:(i32)=>i32=identity<i32>;return f(9);}",
        "pub fn first<T:Copy,const N:usize>(xs:{T;N}) -> T {return xs[0];} pub fn main() -> i32 {let f:({i32;2})=>i32=first;return f({8,9});}",
        "pub fn identity<T>(x:T) -> T {return x;} pub fn select() -> (i32)=>i32 {return identity;} pub fn main() -> i32 {return select()(6);}",
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn take(x:Cell) -> i32 {let held=Hold {cell: x};return held.cell.value+1;} pub fn main() -> i32 {var x=Cell {value: 1};let n=take(x);x=Cell {value: 4};return n+x.value;}",
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn take(x:Cell) -> i32 {let held=Hold {cell: x};return held.cell.value+1;} pub fn main() -> i32 {var x=Cell {value: 1};let n=take(x);if true {x=Cell {value: 4}} else {x=Cell {value: 5}};return n+x.value;}",
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn take(x:Cell) -> i32 {let held=Hold {cell: x};return held.cell.value+1;} pub fn main() -> i32 {var x=Cell {value: 1};var n=0;while n<6 {n+=take(x);x=Cell {value: 1};};return n+x.value;}",
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn take(x:Cell) -> i32 {let held=Hold {cell: x};return held.cell.value+1;} pub fn main() -> i32 {var x=Cell {value: 1};var n=0;while n<6 {n+=take(x);x=Cell {value: 1};};return n+x.value;}",
        "use compiler_fixtures.record_7.{Math}; pub fn main() -> i32 {let f=Math.add;return f(3);}",
        "use compiler_fixtures.record_8.{Handler}; pub fn add(n:i32) -> i32 {return n+2;} pub fn main() -> i32 {let h=Handler {run: add};return h.invoke();}",
        "pub fn add(n:i32) -> i32 {return n+2;} pub fn main() -> i32 {let f:(i32)=>i32=add;return f(3);}",
        "pub fn one() -> i32 {return 1;} pub fn two() -> i32 {return 2;} pub fn choose(b:bool) -> ()=>i32 {return if b {one} else {two};} pub fn main() -> i32 {return choose(false)();}",
        "pub fn bump(var n:i32) -> i32 {n+=1;return n;} pub fn apply(f:(var i32)=>i32,n:i32) -> i32 {return f(n);} pub fn main() -> i32 {let n=3;return apply(bump,n)+n;}",
        "pub fn one() -> i32 {return 1;} pub fn two() -> i32 {return 2;} pub fn main() -> i32 {var f=one;let copy=f;f=two;return f()*10+copy();}",
        "use compiler_fixtures.record_9.{Handler}; pub fn add(n:i32) -> i32 {return n+2;} pub fn main() -> i32 {let h=Handler {run: add};return h.run(3);}",
        "pub fn add(n:i32) -> i32 {return n+2;} pub fn apply<T:Copy>(f:(T)=>T,n:T) -> T {return f(n);} pub fn main() -> i32 {return apply(add,3);}",
        "pub fn add(n:i32) -> i32 {return n+2;} pub fn invoke<T:Copy>(f:()=>T) -> T {return f();} pub fn five() -> i32 {return 5;} pub fn main() -> i32 {return invoke(five);}",
        "pub fn yes(n:i32) -> bool {return n>0;} pub fn main() -> i32 {let predicate=yes;return match 7 {x if predicate(x)=>x,_=>0};}",
        "pub fn fail() -> i32 {return 1/0;} pub fn main() -> i32 {let f=fail;return f();}",
        "pub fn fail() -> i32 {\nlet n=1;\nlet text=\"hello\";return n/0;\n}\npub fn main() -> i32 {return fail();}",
        "pub fn main() -> i32 {\nlet a={1};\nreturn a[2];\n}",
        "use compiler_fixtures.record_5.{Cell}; pub fn bump(var x:Cell) -> () {x.value+=1;} pub fn read(x:Cell) -> i32 {return x.value;} pub fn main() -> i32 {var x=Cell {value: 1};bump(x);return read(x)+x.value;}",
        "use compiler_fixtures.record_5.{Cell}; pub fn bump(var x:Cell,n:i32) -> () {if n==0 {return;};x.value+=1;bump(x,n-1);} pub fn main() -> i32 {var x=Cell {value: 1};bump(x,5);return x.value;}",
        "use compiler_fixtures.record_5.{Cell}; pub fn read(x:Cell) -> i32 {return x.value;} pub fn main() -> i32 {let x=Cell {value: 3};return read(x)+read(x)+x.value;}",
        "use compiler_fixtures.record_5.{Cell}; pub fn select<T>(x:T,n:i32) -> i32 {return n;} pub fn main() -> i32 {let x=Cell {value: 1};return select(x,7)+x.value;}",
        "use compiler_fixtures.record_5.{Cell}; pub fn change(var x:Cell,var y:Cell) -> () {x=y;x.value=9;} pub fn main() -> i32 {var a=Cell {value: 1};var b=Cell {value: 2};change(a,b);return a.value*10+b.value;}",
        "use compiler_fixtures.record_5.{Cell}; pub fn bump(var x:Cell) -> () {x.value+=1;} pub fn main() -> i32 {var pair=(Cell {value: 1},Cell {value: 2});bump(pair.0);return pair.0.value+pair.1.value;}",
        "use compiler_fixtures.record_10.{Cell}; pub fn main() -> i32 {var c=Cell {value: 1};c.twice();return c.read();}",
        "use compiler_fixtures.record_11.{Cell}; pub fn main() -> i32 {var c=Cell {value: 1};c.bump();return c.read()+c.value;}",
        "use compiler_fixtures.record_12.{Box}; pub fn main() -> i32 {let b=Box<i32> {value: 3};return b.read();}",
        "use compiler_fixtures.record_13.{Box}; pub fn main() -> i32 {return (Box<i32> {value: 9}).get();}",
        "use compiler_fixtures.record_14.{Cell}; pub fn main() -> i32 {let c=Cell {value: 3};return c.read();}",
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn main() -> i32 {var x=Cell {value: 1};let y=Hold {cell: x};if true {x=Cell {value: 2}} else {x=Cell {value: 3}};return y.cell.value+x.value;}",
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn main() -> i32 {var x=(Cell {value: 1},Cell {value: 2});let y=Hold {cell: x.0};let z=Hold {cell: x.1};x.0=Cell {value: 3};return y.cell.value+z.cell.value+x.0.value;}",
        "use compiler_fixtures.record_5.{Cell}; pub fn identity(x:Cell) -> Cell {return x;} pub fn main() -> i32 {let x=Cell {value: 7};let y=identity(x);return y.value;}",
        "use compiler_fixtures.record_5.{Cell}; pub fn main() -> i32 {let x=Cell {value: 7};let y=x;return y.value+x.value;}",
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; use compiler_fixtures.record_15.{Pair}; pub fn main() -> i32 {var x=Pair {a: Cell {value: 1}, b: Cell {value: 2}};let y=Hold {cell: x.a};x.a=Cell {value: 4};let z=x;return y.cell.value+z.a.value+z.b.value;}",
        "pub enum E {A(i32),B} pub fn main() -> i32 {let e=E.A(7);return match e {E.A(x) if x>2=>x,E.A(_)=>1,E.B=>0};}",
        "use compiler_fixtures.record_5.{Cell}; pub fn identity<T>(x:T) -> T {return x;} pub fn main() -> i32 {let x=Cell {value: 5};let y=identity(x);return y.value;}",
        "use compiler_fixtures.record_5.{Cell}; pub fn take(x:Cell) -> i32 {return x.value;} pub fn main() -> i32 {var x=Cell {value: 1};var n=0;while n<3 {n+=take(x);x=Cell {value: 1};};return n;}",

        "pub fn main() -> i32 {var a=1;var b=2;let ref var x=a;*x=if true {x=b;7} else {0};return a*10+b;}",
        "pub fn main() -> i32 {var a=1;let ref var x=a;*x=x+2;return a;}",
        "pub fn main() -> i32 {var a={1};let ref x=a[2];return x;}",
        "pub fn main() -> i32 {var a={1};match a[2] {ref x=>{return x;}}}",
        "pub fn main() -> i32 {var a=(1,2);let (ref var x,ref y)=a;*x+=y;return a.0;}",

        "#[Copy] pub enum E {A(i32),B} pub fn main() -> i32 {var a=E.A(3);match a {E.A(ref var x) if x>1=>{*x+=4},_=>{}};return match a {E.A(x)=>x,E.B=>0};}",
        "#[Copy] pub enum E {A(i32),B} pub fn main() -> i32 {var a=E.A(3);match a {E.A(ref var x) if false=>{*x=99},E.A(ref var x)=>{*x+=1},_=>{}};return match a {E.A(x)=>x,E.B=>0};}",
        "pub fn main() -> i32 {var a=(1,2);match a {(ref var x,_) if a.1==2=>{*x=8},_=>{}};return a.0;}",
        "pub fn main() -> i32 {let a=(1,2);let x=match a {(x,_)=>x};return x+a.1;}",
        "pub fn main() -> i32 {let pair=(1,2);var n=0;while n<2 {let (x,_)=pair;n+=x;};return n;}",
        "pub fn main() -> i32 {var pair=(1,2);let (x,_)=pair;pair.1=9;return x+pair.1;}",

        "pub fn main() -> i32 {var a={1,2};let {ref var x,..}=a;*x=7;*x+=1;return a[0];}",
        "pub fn main() -> i32 {var a=1;var b=2;let ref var x=a;*x=3;x=b;*x=7;return a*10+b;}",
        "pub fn main() -> i32 {var a=(1,2);let (ref var x,ref var y)=a;*x+=3;*y+=5;return a.0+a.1;}",
        "pub fn main() -> i32 {var a={1,2};var i:usize=0;let ref var x=a[i];i=1;*x=8;return a[0]+a[1];}",
        "pub fn main() -> i32 {var a=1;let ref var parent=a;let ref child=parent;let copy=child;*parent=5;return a+copy;}",
        "pub fn main() -> i32 {var a=1;let ref x=a;let copy=x;a=4;return copy+a;}",
        "pub fn main() -> i32 {var a=1;let ref var x=a;var n=0;while n<3 {*x+=1;n+=1;};return a;}",

        include_str!("../../../../examples/arithmetic/main.cl"),
        include_str!("../fixtures/unsafe.fixture"),
        "use compiler_fixtures.record_16.{Box}; pub fn main() -> i32 {let boxed=Box<i32>.create(17);return Box.extract(boxed);}",
        "pub fn predicate(var x:i32) -> bool {x+=1;return x==4;} pub fn main() -> i32 {let n=3;return match n {x if predicate(x)=>x,_=>0};}",

        "#[Copy] pub enum Option<T> {None,Some(T)} pub fn unwrap<T:Copy>(value:Option<T>,fallback:T) -> T {return match value {Option.Some(x)=>x,Option.None=>fallback};} pub fn main() -> i32 {let value=Option<i32>.Some(8);return unwrap(value,1);}",

        "pub fn identity<T>(x:T) -> T {return x;} pub fn main() -> i32 {let a:u8=identity(7);return identity<i32>(a as i32);}",
        "pub fn last<T,const N:usize>(values:{T;N}) -> T {return values[N-1];} pub fn main() -> i32 {return last({1,2,3});}",
        "pub fn pick<T>(n:i32,x:T) -> T {if n==0 {return x;};return pick<T>(n-1,x);} pub fn main() -> i32 {return pick(5,120);}",

        "#[Copy] pub enum E {None,Some(i32)} pub fn main() -> i32 {let a=E.None;return match a {E.Some(x)=>x,E.None=>7};}",
        "#[Copy] pub enum E {A {x:i32,y:i32},B(u128),C} pub fn main() -> i32 {let a=E.B(340282366920938463463374607431768211455);return match a {E.A {x,y}=>x+y,E.B(x)=> (x/3) as i32,E.C=>0};}",
        "#[Copy] pub enum Inner {A(i32),B} #[Copy] pub enum Outer {Some(Inner),None} pub fn main() -> i32 {let x=Outer.Some(Inner.B);return match x {Outer.Some(Inner.A(v))=>v,Outer.Some(_)=>2,Outer.None=>3};}",

        "pub fn main() -> i32 {return match (3,true) {(x,false)=>x,(x,true) if x>4=>9,(x,_)=>x+10};}",
        "pub fn main() -> i32 {match true {false=>{return 4;},true=>{return 7;}}}",
        "pub fn main() -> i32 {return match {1,2,3,4} {{x,..,y}=>x+y};}",
        "use compiler_fixtures.record_17.{Pair}; pub fn main() -> i32 {var n=0;var p=Pair {right:if true {n+=1;n} else {0},left:if true {n+=1;n} else {0}};let c=p;p.left+=10;return c.left+p.left;}",

        "pub fn main() -> i32 {return if true {return 9;} else {return 4;};}",
        "pub fn main() -> i32 {var i=0;var sum=0;while i<10 {i+=1;if i==3 {continue;};if i==6 {break;};sum+=i;};return sum;}",
        "pub fn main() -> i32 {var pair=(1,true);pair.0+=2;return pair.0;}",
        "pub fn main() -> i32 {var a={1,2};var i:usize=0;a[if true {i+=1;i} else {0}]+=3;return a[1];}",
        "pub fn main() -> i32 {var a={1};a[2]=1/0;return a[0];}",
        "pub fn main() -> i32 {return if false && 1/0==0 {1} else {2};}",
        "pub fn main() -> i32 {return if true || 1/0==0 {3} else {2};}",
        "pub fn main() -> i32 {let x:i8=127;return (x+1) as i32;}",
        "pub fn main() -> i32 {let x:i128=-170141183460469231731687303715884105728;return (x % -1) as i32;}",
        "pub fn main() -> i32 {let x:i128=1;let count:i8=-1;return (x << count) as i32;}",
        "pub fn main() -> i32 {let x:u8=255;let count:u128=256;return (x >> count) as i32;}",
        "pub fn main() -> i32 {let x:f32=1.5;return x as i32;}",
        "pub fn main() -> i32 {let x=0.0/0.0;return x as i32;}",
        "pub fn main() -> i32 {let x=1.0/0.0;return x as i32;}",
        "pub fn main() -> i32 {let x:f32=1.5;let wide=x as f64;return wide as i32;}",
        "pub fn main() -> i32 {let x:u128=340282366920938463463374607431768211455;return (x / 3) as i32;}",
        "pub fn main() -> i32 {let x=19.0;return (x % 4.0) as i32;}",
    ];
    let dir = std::env::temp_dir().join(format!("clust-native-rust-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let compiler = std::env::var("CLUST_RUSTC").unwrap_or_else(|_| "rustc".into());
    let mut count = 0;
    for checks in [OverflowChecks::Checked, OverflowChecks::Wrapping] {
        let mut generated = String::new();
        let mut expected_cases = Vec::new();
        for (id, source) in cases.iter().enumerate() {
            let parsed = parse(SourceId(0), source);
            assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
            let program = check(&parsed.module.unwrap())
                .unwrap_or_else(|errors| panic!("{source}: {errors:?}"));
            let cfg = clust_core::mir::lower(&program);
            let expected = execute(&program, program.main.unwrap(), vec![], checks, 100_000);
            let mir_value = clust_core::mir::interpret::execute(
                &cfg,
                cfg.main.unwrap(),
                vec![],
                checks,
                100_000,
            );
            assert_eq!(
                expected.as_ref().map_err(|e| &e.kind),
                mir_value.as_ref().map_err(|e| &e.kind),
                "{source}"
            );
            let code = match &expected {
                Ok(Value::Integer(v)) => (v.bits() & 255) as i32,
                Ok(Value::Unit) => 0,
                Err(_) => 101,
                other => panic!("{other:?}"),
            };
            let original = Source::new("native.cl", *source);
            let location = expected
                .err()
                .map(|trap| original.location(trap.span.start).unwrap());
            let rust =
                emit_with_sources(&program, checks, std::slice::from_ref(&original)).unwrap();
            generated.push_str(&format!(
                "mod case{id} {{\n{}\n}}\n",
                rust.replace("fn main() {", "pub fn run() {")
            ));
            expected_cases.push((code, location));
        }
        generated.push_str(
            "fn main() { match std::env::args().nth(1).unwrap().parse::<usize>().unwrap() {\n",
        );
        for id in 0..cases.len() {
            generated.push_str(&format!("{id} => case{id}::run(),\n"));
        }
        generated.push_str("_ => panic!(\"invalid test case\"), } }\n");
        fs::write(dir.join("case.rs"), &generated).unwrap();
        for opt in ["opt-level=0", "opt-level=3"] {
            let output = Command::new(&compiler)
                .args(["--edition=2021", "-C", opt])
                .arg(dir.join("case.rs"))
                .arg("-o")
                .arg(dir.join("case"))
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}; source: {}",
                String::from_utf8_lossy(&output.stderr),
                dir.join("case.rs").display()
            );
            for (id, (code, location)) in expected_cases.iter().enumerate() {
                let output = Command::new(dir.join("case"))
                    .arg(id.to_string())
                    .output()
                    .unwrap();
                assert_eq!(
                    output.status.code(),
                    Some(*code),
                    "{}: {}",
                    cases[id],
                    String::from_utf8_lossy(&output.stderr)
                );
                if let Some((line, column)) = location {
                    assert!(
                        String::from_utf8_lossy(&output.stderr)
                            .contains(&format!("native.cl:{line}:{column}:")),
                        "{}: {}",
                        cases[id],
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                count += 1;
            }
        }
    }
    eprintln!("{count} native Rust comparisons passed");
    fs::remove_dir_all(dir).unwrap();
}
