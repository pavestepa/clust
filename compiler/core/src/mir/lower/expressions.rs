mod control_flow;
use super::*;

impl Builder {
    pub(super) fn expr_mode(&mut self, expr: &Expr, borrowed: Option<loans::Access>) -> Operand {
        if self.current.is_none() {
            return Operand::Constant(Value::Unit);
        }
        if let Some(access) = borrowed {
            if !matches!(
                expr.kind,
                ExprKind::Block(_) | ExprKind::If { .. } | ExprKind::Match(_)
            ) {
                let source = self.place(expr);
                let id = self.temp(&expr.ty, expr.span);
                self.function.locals[id].view = Some(access);
                self.evaluate_into(Operation::Borrow(source), Place::local(id), expr.span);
                return Operand::Local(id);
            }
        }
        let operation = match &expr.kind {
            ExprKind::StoreView(value) => Operation::StoreView(self.place(value)),
            ExprKind::StoredView(_) => {
                let place = self.place(expr);
                if expr.ty.copyable() {
                    Operation::Copy(place)
                } else {
                    Operation::Move(place)
                }
            }
            ExprKind::StorageCall {
                function,
                arguments,
                contract,
            } => Operation::CallStorage {
                function: *function,
                arguments: arguments.iter().map(|a| self.expr(a)).collect(),
                contract: contract.clone(),
            },
            ExprKind::BorrowedCall {
                function,
                arguments,
                contract,
            } => {
                let arguments = arguments.iter().map(|a| self.expr(a)).collect();
                let id = self.temp(&expr.ty, expr.span);
                self.function.locals[id].view = Some(contract.access);
                self.evaluate_into(
                    Operation::CallView {
                        function: *function,
                        arguments,
                        contract: contract.clone(),
                    },
                    Place::local(id),
                    expr.span,
                );
                return Operand::Local(id);
            }
            ExprKind::Identity(left, right) => {
                let mut places = Vec::new();
                for value in [left, right] {
                    let place = self.place(value);
                    let id = self.temp(&value.ty, value.span);
                    self.function.locals[id].view = Some(loans::Access::Readonly);
                    self.evaluate_into(Operation::Borrow(place), Place::local(id), value.span);
                    places.push(Place {
                        local: id,
                        projections: vec![Projection::Dereference],
                    });
                }
                Operation::Identity(places.remove(0), places.remove(0))
            }
            ExprKind::Take(value) => Operation::Move(self.place(value)),
            ExprKind::ArgumentView { place, access } => {
                let source = self.place(place);
                let id = self.temp(&place.ty, expr.span);
                self.function.locals[id].view = Some(*access);
                self.evaluate_into(Operation::Borrow(source), Place::local(id), expr.span);
                return Operand::Address(Place {
                    local: id,
                    projections: vec![Projection::Dereference],
                });
            }
            ExprKind::New {
                constructor,
                arguments,
            } => {
                let args = arguments.iter().map(|v| self.expr(v)).collect::<Vec<_>>();
                let object = self.evaluate(
                    Operation::AllocateClass(expr.ty.class_key()),
                    &expr.ty,
                    expr.span,
                );
                let Operand::Local(local) = object else {
                    unreachable!()
                };
                let mut supplied = vec![Operand::Address(Place::local(local))];
                supplied.extend(args);
                self.evaluate(
                    Operation::Call(*constructor, supplied),
                    &Type::Unit,
                    expr.span,
                );
                Operation::CompleteClass(Operand::Local(local))
            }
            ExprKind::Value(value) => Operation::Use(Operand::Constant(value.clone())),
            ExprKind::Borrow { local, place } => {
                let place = self.place(place);
                self.evaluate_into(Operation::Borrow(place), Place::local(*local), expr.span);
                return Operand::Constant(Value::Unit);
            }
            ExprKind::View(_)
            | ExprKind::Local(_)
            | ExprKind::Field(_, _)
            | ExprKind::Index(_, _)
            | ExprKind::Payload(_, _) => {
                let place = self.place(expr);
                if expr.ty.copyable() {
                    Operation::Copy(place)
                } else {
                    Operation::Move(place)
                }
            }
            ExprKind::Block(body) => return self.block_mode(body, borrowed),
            ExprKind::Tuple(values) | ExprKind::Array(values) => {
                Operation::Aggregate(self.aggregate_fields(values))
            }
            ExprKind::Enum { variant, fields } => {
                Operation::Enum(*variant, self.aggregate_fields(fields))
            }
            ExprKind::Tag(value) => Operation::TagPlace(self.place(value)),
            ExprKind::Unary(op @ ("storage_view" | "storage_view_mut"), values) => {
                let ExprKind::Tuple(args) = &values.kind else {
                    unreachable!()
                };
                let arguments = vec![Operand::Address(self.place(&args[0])), self.expr(&args[1])];
                let access = if *op == "storage_view" {
                    loans::Access::Readonly
                } else {
                    loans::Access::Mutable
                };
                let contract = returns::Contract {
                    access,
                    origins: std::collections::BTreeSet::from([returns::Origin {
                        parameter: 0,
                        path: vec![],
                    }]),
                };
                let id = self.temp(&expr.ty, expr.span);
                self.function.locals[id].view = Some(access);
                self.evaluate_into(
                    Operation::StorageView {
                        arguments,
                        contract,
                    },
                    Place::local(id),
                    expr.span,
                );
                return Operand::Local(id);
            }
            ExprKind::Unary(operator @ ("raw_addr" | "raw_addr_mut"), value) => {
                Operation::Unary(operator, Operand::Address(self.place(value)))
            }
            ExprKind::Unary(operator, value) => Operation::Unary(operator, self.expr(value)),
            ExprKind::Cast(value) => Operation::Cast(self.expr(value), expr.ty.clone()),
            ExprKind::Binary(operator, left, right) if ["&&", "||"].contains(operator) => {
                return self.short_circuit(expr, operator, left, right)
            }
            ExprKind::Binary(operator, left, right) => {
                let a = self.expr(left);
                let b = self.expr(right);
                Operation::Binary(operator, a, b)
            }
            ExprKind::Call(id, args) => {
                Operation::Call(*id, args.iter().map(|v| self.expr(v)).collect())
            }
            ExprKind::CallIndirect(callee, args, contract) => {
                let callee = self.expr(callee);
                let operation = Operation::CallIndirect(
                    callee,
                    args.iter().map(|v| self.expr(v)).collect(),
                    contract.clone(),
                );
                if let Some(view) = &contract.view {
                    let id = self.temp(&expr.ty, expr.span);
                    self.function.locals[id].view = Some(view.access);
                    self.evaluate_into(operation, Place::local(id), expr.span);
                    return Operand::Local(id);
                }
                operation
            }
            ExprKind::Assign {
                place,
                operator,
                value,
            } => return self.assignment(expr, place, operator, value),
            ExprKind::If { condition, yes, no } => {
                return self.conditional(expr, condition, yes, no, borrowed)
            }
            ExprKind::While { condition, body } => return self.while_loop(expr, condition, body),
            ExprKind::Match(arms) => return self.match_arms(expr, arms, borrowed),
        };
        self.evaluate(operation, &expr.ty, expr.span)
    }
}
