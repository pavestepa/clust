# expressions

Check an AST expression with its expected type and value context. The dispatcher routes operators, calls, projections and assignments to focused checks. Shared finalization applies the storage read rule and verifies the result type.

Module entry: [expressions](../expressions.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [assign_expression](assign_expression.rs) | `check_assign`, `explicit` |
| [binary_expression](binary_expression.rs) | `check_binary` |
| [call_expression](call_expression.rs) | `check_call` |
| [cast_expression](cast_expression.rs) | `check_cast` |
| [if_expression](if_expression.rs) | `check_if` |
| [member_expression](member_expression.rs) | `check_member` |
| [unary_expression](unary_expression.rs) | `check_unary`, `view_name` |
