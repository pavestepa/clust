//! Backend tests also exercise IR operations unavailable to source programs.
//! Parse with the production grammar first, then construct those nodes in Rust.
//! Module linking uses the same pass for test inputs, never for bundled modules.
use crate::{ast::*, modules, parser, source::SourceId};
use std::path::Path;

pub fn parse(source: SourceId, text: &str) -> parser::Parsed {
    parse_records(source, text)
}

/// Construct backend-only unsafe IR probes after parsing their fixture syntax.
/// Source conformance tests use `parse` or the production parser.
pub fn parse_ir(source: SourceId, text: &str) -> parser::Parsed {
    let mut parsed = parse_records(source, text);
    if let Some(module) = &mut parsed.module {
        lower(module);
    }
    parsed
}

/// Supply backend nominal nodes explicitly from Rust, without source attributes.
pub fn parse_with_items(source: SourceId, text: &str, items: Vec<Item>) -> parser::Parsed {
    let mut parsed = parse_records(source, text);
    if let Some(module) = &mut parsed.module {
        module.items.extend(items);
    }
    parsed
}

pub fn load(entry: &Path, package: &str) -> modules::Loaded {
    modules::load_with_parser(entry, package, parse_records)
}

/// Load a backend IR fixture containing unsafe-operation probes.
pub fn load_ir(entry: &Path, package: &str) -> modules::Loaded {
    let mut loaded = load(entry, package);
    if let Some(module) = &mut loaded.module {
        lower(module);
    }
    loaded
}

pub mod record;
mod records;

fn parse_records(source: SourceId, text: &str) -> parser::Parsed {
    let mut parsed = parser::parse(source, text);
    if let Some(module) = &mut parsed.module {
        let mut result = Vec::new();
        for item in std::mem::take(&mut module.items) {
            if let ItemKind::Use(bindings) = &item.kind {
                let mut remaining = vec![];
                for binding in bindings {
                    if binding
                        .path
                        .first()
                        .is_some_and(|name| name == "compiler_fixtures")
                    {
                        assert_eq!(
                            binding.path.len(),
                            3,
                            "fixture path must identify one Rust record factory"
                        );
                        let record =
                            records::get(&binding.path[1]).expect("unknown Rust record fixture");
                        assert_eq!(record.name, binding.path[2]);
                        result.push(record.build(item.span));
                    } else {
                        remaining.push(binding.clone());
                    }
                }
                if !remaining.is_empty() {
                    result.push(Item {
                        kind: ItemKind::Use(remaining),
                        ..item
                    });
                }
            } else {
                result.push(item);
            }
        }
        module.items = result;
    }
    parsed
}

fn lower(module: &mut Module) {
    BackendFixtures.module(module);
}
struct BackendFixtures;
impl BackendFixtures {
    fn module(&self, module: &mut Module) {
        for item in &mut module.items {
            match &mut item.kind {
                ItemKind::Function(f) => self.function(f),
                ItemKind::Aggregate { members, .. } => {
                    for member in members {
                        match member {
                            Member::Method { function, .. } => self.function(function),
                            Member::Constructor { body, .. } | Member::Destructor(body) => {
                                self.block(body)
                            }
                            _ => {}
                        }
                    }
                }
                ItemKind::Interface { methods, .. } => {
                    methods.iter_mut().for_each(|f| self.function(f))
                }
                _ => {}
            }
        }
    }
    fn function(&self, f: &mut Function) {
        if let Some(body) = &mut f.body {
            if body.statements.first().is_some_and(|s| matches!(&s.kind, StatementKind::Expression(e) if marker(e, "requires_unsafe_caller"))) {
            body.statements.remove(0);
            f.unsafe_ = true;
        }
            self.block(body);
        }
    }
    fn block(&self, body: &mut Block) {
        for stmt in &mut body.statements {
            match &mut stmt.kind {
                StatementKind::Expression(Expr {
                    kind:
                        ExprKind::If {
                            condition,
                            then_block,
                            else_branch: None,
                        },
                    ..
                }) if marker(condition, "trusted_region") => {
                    self.block(then_block);
                    stmt.kind = StatementKind::Unsafe(Block {
                        statements: std::mem::take(&mut then_block.statements),
                        span: then_block.span,
                    });
                }
                StatementKind::Expression(e)
                | StatementKind::Binding { value: e, .. }
                | StatementKind::Return(Some(e)) => self.expr(e),
                _ => {}
            }
        }
    }
    fn expr(&self, e: &mut Expr) {
        match &mut e.kind {
            ExprKind::Construct { fields, .. } => {
                for (_, value) in fields {
                    self.expr(value);
                }
            }
            ExprKind::Group(v)
            | ExprKind::Unary { value: v, .. }
            | ExprKind::Member { value: v, .. }
            | ExprKind::Cast { value: v, .. }
            | ExprKind::Specialize { value: v, .. } => self.expr(v),
            ExprKind::Call {
                callee, arguments, ..
            } => {
                self.expr(callee);
                arguments.iter_mut().for_each(|v| self.expr(v));
            }
            ExprKind::Index {
                value: left,
                index: right,
            }
            | ExprKind::Binary { left, right, .. }
            | ExprKind::Assign {
                place: left,
                value: right,
                ..
            } => {
                self.expr(left);
                self.expr(right);
            }
            ExprKind::Tuple(v) | ExprKind::Array(v) | ExprKind::New { arguments: v, .. } => {
                v.iter_mut().for_each(|v| self.expr(v))
            }
            ExprKind::If {
                condition,
                then_block,
                else_branch,
            } => {
                self.expr(condition);
                self.block(then_block);
                if let Some(v) = else_branch {
                    self.expr(v);
                }
            }
            ExprKind::Match { value, arms } => {
                self.expr(value);
                for arm in arms {
                    if let Some(guard) = &mut arm.guard {
                        self.expr(guard);
                    }
                    self.expr(&mut arm.value);
                }
            }
            ExprKind::Branch(b) => self.block(b),
            ExprKind::While { condition, body } => {
                self.expr(condition);
                self.block(body);
            }
            ExprKind::Closure(f) => self.function(f),
            ExprKind::Name(_) | ExprKind::Literal(_) => {}
        }
    }
}

fn marker(value: &Expr, name: &str) -> bool {
    matches!(&value.kind, ExprKind::Call { callee, types, arguments } if types.is_empty() && arguments.is_empty() && matches!(&callee.kind, ExprKind::Name(n) if n == name))
}
