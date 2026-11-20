use super::*;

#[test]
fn cfg_execution_preserves_evaluation_order_and_control_flow() {
    for source in [
  include_str!("../../../../examples/arithmetic/main.cl"),
  "pub fn main() -> i32 {var a=1;return a+if true {a=5;a} else {0};}",
  "pub fn main() -> i32 {var a={1,2};var i:usize=0;a[if true {i+=1;i} else {0}]+=if true {a[1]=10;3} else {0};return a[1];}",
  "pub fn main() -> i32 {var a={1};return a[if true {a[0]=3;0} else {0}];}",
  "pub fn main() -> i32 {var x=0;while x<10 {x+=1;if x==2 {continue;};if x==5 {break;}};return x;}",
  "pub fn main() -> i32 {return if false && 1/0==0 {1} else {7};}",
  "pub fn main() -> i32 {return if true {return 4;} else {return 8;};}",
  "pub fn main() -> i32 {var a={1};a[2]=1/0;return 0;}",
  "#[Copy] pub enum E {A(i32),B} pub fn main() -> i32 {return match E.B {E.A(v)=>v,E.B=>5};}",
  "use compiler_fixtures.record_124.{S}; pub fn main() -> i32 {var S {a,b}=S {a: 1, b: 2};a+=3;return a+b;}",
  "pub fn main() -> i32 {let a:i8=127;return (a+1) as i32;}",
 ]{
  let hir=compile(source);let cfg=mir::lower(&hir);
  for function in &cfg.functions {
   assert!(mir::dataflow::initialization_errors(function).is_empty(),"{source}: {:?}",mir::dataflow::initialization_errors(function));
   let _=mir::dataflow::liveness(function);
  }
  for mode in [OverflowChecks::Checked,OverflowChecks::Wrapping] {
   let reference=interpret::execute(&hir,hir.main.unwrap(),vec![],mode,100000).map_err(|t|t.kind);
   let result=mir::interpret::execute(&cfg,cfg.main.unwrap(),vec![],mode,100000).map_err(|t|t.kind);
   assert_eq!(result,reference,"{source}");
  }
 }
}

#[test]
fn liveness_keeps_loop_and_panic_uses_and_init_intersects_branches() {
    use mir::{Block, Function, Operand as V, Operation as O, Place, Terminator as T};
    let span = Span {
        source: SourceId(0),
        start: 0,
        end: 1,
    };
    let mut function = Function {
        rust_adapter: None,
        storage_writes: Default::default(),
        drop_reads: Default::default(),
        result_view: None,
        classes: Default::default(),
        cleanup: mir::cleanup::Plan::new(2),
        name: "test".into(),
        parameters: vec![0],
        locals: (0..2)
            .map(|id| hir::Local {
                view: None,
                name: id.to_string(),
                ty: hir::Type::Bool,
                mutable: true,
                span,
            })
            .collect(),
        result: hir::Type::Bool,
        entry: 0,
        span,
        blocks: vec![
            Block {
                span,
                terminator: T::Branch {
                    condition: V::Local(0),
                    yes: 1,
                    no: 3,
                },
            },
            Block {
                span,
                terminator: T::Evaluate {
                    operation: O::Use(V::Constant(hir::Value::Bool(true))),
                    destination: Place::local(1),
                    next: 2,
                    unwind: 4,
                },
            },
            Block {
                span,
                terminator: T::Goto(0),
            },
            Block {
                span,
                terminator: T::Return(V::Local(1)),
            },
            Block {
                span,
                terminator: T::Return(V::Local(1)),
            },
        ],
    };
    let live = mir::dataflow::liveness(&function);
    assert!(
        live.live_in[1].contains(&1),
        "failed write must preserve old value for panic edge"
    );
    assert!(
        live.live_in[2].contains(&0),
        "condition remains live across loop back edge"
    );
    assert_eq!(
        mir::dataflow::initialization_errors(&function),
        vec![(3, 1), (4, 1)]
    );
    function.parameters.push(1);
    assert!(mir::dataflow::initialization_errors(&function).is_empty());
}
