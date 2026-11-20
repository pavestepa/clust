use super::*;

pub(crate) fn constant_value(
    value: &a::Expr,
    ty: &Type,
    constants: &crate::constants::Constants,
) -> Result<Value> {
    let signatures = BTreeMap::new();
    let mut checker = Checker {
        unsafe_depth: 0,
        callable_parameters: Default::default(),
        member_owner: None,
        storage_hints: Default::default(),
        storage_requests: Default::default(),
        storage_roots: Default::default(),
        storage_sources: Default::default(),
        storage_parameters: Default::default(),
        signatures: &signatures,
        mono: &std::cell::RefCell::new(generics::Monomorphization::default()),
        constants,
        pattern_views: false,
        continuations: vec![],
        value_uses: BTreeMap::new(),
        nominals: &records::Nominals::default(),
        private_functions: &BTreeMap::new(),
        scopes: vec![BTreeMap::new()],
        locals: Vec::new(),
        result: ty.clone(),
        actual_result: None,
        loops: 0,
    };
    let expression = checker.expr(value, Some(ty), true)?;
    let function = h::Function {
        rust_adapter: None,
        storage_writes: Vec::new(),
        result_view: None,
        name: "constant".into(),
        parameters: Vec::new(),
        locals: checker.locals,
        result: ty.clone(),
        body: h::Block {
            construction: vec![],
            storage: vec![],
            statements: vec![h::Statement::Return(Some(expression))],
            result: Type::Unit,
            diverges: true,
            span: value.span,
        },
        span: value.span,
    };
    let program = h::Program {
        classes: Default::default(),
        functions: vec![function],
        main: None,
    };
    crate::interpret::execute(
        &program,
        0,
        Vec::new(),
        clust_semantics::OverflowChecks::Checked,
        100_000,
    )
    .map_err(|trap| {
        error(
            trap.span,
            "E0332",
            format!("constant evaluation failed: {:?}", trap.kind),
        )
    })
}

pub(super) fn literal(
    value: &a::Literal,
    expected: Option<&Type>,
    negative: bool,
    span: Span,
) -> Result<(h::ExprKind, Type)> {
    let (value, ty) = match value {
        a::Literal::Integer(text) => {
            let ty = match expected {
                Some(Type::Integer(ty)) => *ty,
                None => I::I32,
                _ => {
                    return Err(error(
                        span,
                        "E0319",
                        "integer literal requires an integer context",
                    ))
                }
            };
            let value = Integer::literal(ty, magnitude(text, span)?, negative)
                .map_err(|_| error(span, "E0319", "integer literal out of range"))?;
            (Value::Integer(value), Type::Integer(ty))
        }
        a::Literal::Float(text) => {
            let text = text.replace('_', "");
            if expected == Some(&Type::F32) {
                (
                    Value::F32(
                        text.parse()
                            .map_err(|_| error(span, "E0319", "invalid f32 literal"))?,
                    ),
                    Type::F32,
                )
            } else {
                (
                    Value::F64(
                        text.parse()
                            .map_err(|_| error(span, "E0319", "invalid f64 literal"))?,
                    ),
                    Type::F64,
                )
            }
        }
        a::Literal::Bool(value) => (Value::Bool(*value), Type::Bool),
        a::Literal::Char(value) => (Value::Char(*value), Type::Char),
        a::Literal::String(value) => (Value::Str(value.clone()), Type::Str),
    };
    if let Some(expected) = expected {
        same(expected, &ty, span)?;
    }
    Ok((h::ExprKind::Value(value), ty))
}

pub(super) fn magnitude(text: &str, span: Span) -> Result<u128> {
    let text = text.replace('_', "");
    let (digits, base) = if let Some(s) = text.strip_prefix("0x") {
        (s, 16)
    } else if let Some(s) = text.strip_prefix("0o") {
        (s, 8)
    } else if let Some(s) = text.strip_prefix("0b") {
        (s, 2)
    } else {
        (text.as_str(), 10)
    };
    u128::from_str_radix(digits, base)
        .map_err(|_| error(span, "E0319", "integer literal exceeds 128 bits"))
}
