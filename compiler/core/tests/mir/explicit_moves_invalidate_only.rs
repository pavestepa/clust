use super::*;

#[test]
fn explicit_moves_invalidate_only_the_selected_path_and_backend_rejects_later_reads() {
    use clust_semantics::{Integer, IntegerType};
    use mir::{Block, Function, Operand as V, Operation as O, Place, Projection, Terminator as T};
    let span = Span {
        source: SourceId(0),
        start: 0,
        end: 1,
    };
    let integer = hir::Type::Integer(IntegerType::I32);
    let pair = hir::Type::Tuple(vec![integer.clone(), integer.clone()]);
    let number = |n| hir::Value::Integer(Integer::from_bits(IntegerType::I32, n));
    let field = |n| Place {
        local: 0,
        projections: vec![Projection::Field(n)],
    };
    let function = Function {
        rust_adapter: None,
        storage_writes: std::sync::Arc::new(vec![vec![]]),
        drop_reads: Default::default(),
        result_view: None,
        classes: Default::default(),
        cleanup: mir::cleanup::Plan::new(3),
        name: "partial".into(),
        parameters: vec![0],
        locals: [pair.clone(), integer.clone(), integer]
            .into_iter()
            .enumerate()
            .map(|(id, ty)| hir::Local {
                view: None,
                name: id.to_string(),
                ty,
                mutable: true,
                span,
            })
            .collect(),
        result: pair,
        entry: 0,
        span,
        blocks: vec![
            Block {
                span,
                terminator: T::Evaluate {
                    operation: O::Move(field(0)),
                    destination: Place::local(1),
                    next: 1,
                    unwind: 4,
                },
            },
            Block {
                span,
                terminator: T::Evaluate {
                    operation: O::Copy(field(1)),
                    destination: Place::local(2),
                    next: 2,
                    unwind: 4,
                },
            },
            Block {
                span,
                terminator: T::Evaluate {
                    operation: O::Use(V::Constant(number(9))),
                    destination: field(0),
                    next: 3,
                    unwind: 4,
                },
            },
            Block {
                span,
                terminator: T::Return(V::Local(0)),
            },
            Block {
                span,
                terminator: T::ResumePanic,
            },
        ],
    };
    let mut program = mir::Program {
        classes: Default::default(),
        destructors: Default::default(),
        functions: vec![function],
        main: None,
    };
    let args = vec![hir::Value::Aggregate(vec![number(1), number(2)])];
    assert!(mir::dataflow::initialization_errors(&program.functions[0]).is_empty());
    assert_eq!(
        mir::interpret::execute(&program, 0, args.clone(), OverflowChecks::Checked, 1000).unwrap(),
        hir::Value::Aggregate(vec![number(9), number(2)])
    );
    assert!(clust_core::rust::emit_mir(&program, OverflowChecks::Checked).is_ok());
    program.functions[0].blocks[2].terminator = T::Goto(3);
    assert_eq!(
        mir::dataflow::initialization_errors(&program.functions[0]),
        vec![(3, 0)]
    );
    assert_eq!(
        mir::interpret::execute(&program, 0, args, OverflowChecks::Checked, 1000)
            .unwrap_err()
            .kind,
        interpret::TrapKind::InvalidIr
    );
    assert_eq!(
        clust_core::rust::emit_mir(&program, OverflowChecks::Checked)
            .unwrap_err()
            .code,
        "E0401"
    );
}
