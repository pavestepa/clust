mod results;
use super::*;
use crate::hir::{Expr, ExprKind, Statement};
pub fn lower(program: &h::Program) -> Program {
    let classes = std::sync::Arc::new(program.classes.clone());
    let writes = std::sync::Arc::new(
        program
            .functions
            .iter()
            .map(|f| f.storage_writes.clone())
            .collect::<Vec<_>>(),
    );
    let destructors = drop_plan::Destructors::from_functions(&program.functions)
        .expect("checked destructor declarations");
    Program {
        classes: program.classes.clone(),
        destructors: destructors.clone(),
        functions: program
            .functions
            .iter()
            .map(|f| function(f, classes.clone(), writes.clone(), &destructors))
            .collect(),
        main: program.main,
    }
}
fn function(
    source: &h::Function,
    classes: std::sync::Arc<h::Classes>,
    writes: std::sync::Arc<Vec<storage_writes::Contract>>,
    destructors: &drop_plan::Destructors,
) -> Function {
    let mut builder = Builder {
        function: Function {
            rust_adapter: source.rust_adapter.clone(),
            drop_reads: Default::default(),
            storage_writes: writes,
            result_view: source.result_view.clone(),
            classes,
            cleanup: cleanup::Plan::new(source.locals.len()),
            name: source.name.clone(),
            parameters: source.parameters.clone(),
            locals: source.locals.clone(),
            result: source.result.clone(),
            blocks: vec![],
            entry: 0,
            span: source.span,
        },
        current: None,
        unwind: 0,
        loops: vec![],
        scopes: vec![],
        temporary_start: source.locals.len(),
        cleanup_scopes: vec![0],
    };
    let entry = builder.block_id(source.span);
    builder.function.entry = entry;
    builder.current = Some(entry);
    builder.unwind = builder.block_id(source.span);
    builder.function.blocks[builder.unwind].terminator = Terminator::ResumePanic;
    builder.block(&source.body);
    if builder.current.is_some() {
        builder.end(Terminator::Return(Operand::Constant(Value::Unit)));
    }
    // Every failure exits the frame through an explicit storage boundary. Drop
    // elaboration can use this edge for initialized roots and argument temporaries.
    let resume = builder.block_id(source.span);
    builder.function.blocks[resume].terminator = Terminator::ResumePanic;
    builder.function.blocks[builder.unwind].terminator = Terminator::EndStorage {
        locals: (0..builder.function.locals.len()).collect(),
        next: resume,
    };
    builder.function.drop_reads = builder
        .function
        .locals
        .iter()
        .enumerate()
        .filter(|(_, local)| local.view.is_none() && drop_borrows::reads(&local.ty, destructors))
        .map(|(id, _)| id)
        .collect();
    reachability::prune(&mut builder.function);
    builder.function
}
struct Builder {
    function: Function,
    current: Option<BlockId>,
    unwind: BlockId,
    loops: Vec<(BlockId, BlockId, usize, LocalId)>,
    scopes: Vec<Vec<LocalId>>,
    temporary_start: LocalId,
    cleanup_scopes: Vec<cleanup::ScopeId>,
}
impl Builder {
    // Completed field values are construction-owned until the final aggregate
    // operation transfers them. Failure never invokes the unfinished parent's
    // destructor, and its completed fields clean in declaration/index order.
    // Value contexts materialize Copy views. place() and direct returns keep
    // their descriptor so explicit ref bindings and result provenance survive.
}

mod aggregates;
mod builder;
mod expressions;
mod places;
mod scopes;
