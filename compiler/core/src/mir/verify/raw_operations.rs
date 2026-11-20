use super::*;

pub(super) fn verify_raw(
    f: &Function,
    op: &Operation,
    destination: &Place,
) -> Result<(), &'static str> {
    let result = place_type(f, destination);
    let valid = match op {
        Operation::StorageView {
            arguments,
            contract,
        } => {
            let [Operand::Address(storage), index] = arguments.as_slice() else {
                return Err("invalid MIR storage view arguments");
            };
            let Some(Type::Record { fields, .. }) = place_type(f, storage) else {
                return Err("storage view requires a storage capability");
            };
            (matches!(&result, Some(Type::Class { copy: false, .. }))
                || result.as_ref().is_some_and(Type::raw_copy_value))
                && fields.len() == 2
                && fields[0].name == "data"
                && fields[0].private
                && fields[1].name == "cap"
                && fields[1].private
                && fields[1].ty == Type::Integer(clust_semantics::IntegerType::Usize)
                && matches!(&fields[0].ty,Type::Pointer {mutable:true,pointee} if result.as_ref()==Some(&**pointee))
                && operand_type(f, index)
                    == Some(Type::Integer(clust_semantics::IntegerType::Usize))
                && contract.origins
                    == BTreeSet::from([returns::Origin {
                        parameter: 0,
                        path: vec![],
                    }])
                && (contract.access == loans::Access::Readonly || writable(f, storage))
        }
        Operation::Unary(op @ ("storage_address" | "storage_address_mut"), args) => {
            let Some(Type::Tuple(fields)) = operand_type(f, args) else {
                return Err("invalid storage address arguments");
            };
            fields.len() == 3
                && fields[1..]
                    .iter()
                    .all(|t| *t == Type::Integer(clust_semantics::IntegerType::Usize))
                && matches!((&fields[0],&result), (Type::Pointer {mutable:true,pointee},Some(Type::Pointer {mutable,pointee:target})) if pointee.raw_storage_value() && pointee==target && *mutable==(*op=="storage_address_mut"))
        }
        Operation::Unary("storage_move_slots", args) => {
            let Some(Type::Tuple(fields)) = operand_type(f, args) else {
                return Err("invalid MIR range move");
            };
            fields.len() == 7
                && fields[0] == fields[3]
                && matches!(&fields[0],Type::Pointer {mutable:true,pointee} if pointee.raw_storage_value())
                && [1, 2, 4, 5, 6]
                    .iter()
                    .all(|i| fields[*i] == Type::Integer(clust_semantics::IntegerType::Usize))
                && result == Some(Type::Unit)
        }
        Operation::Unary("storage_detach", value) => {
            let Some(Type::Record { fields, .. }) = operand_type(f, value) else {
                return Err("detach requires RawStorage");
            };
            fields.len() == 2
                && fields[0].private
                && fields[0].name == "data"
                && fields[1].private
                && fields[1].name == "cap"
                && matches!(&fields[0].ty,Type::Pointer {mutable:true,pointee} if pointee.raw_storage_value())
                && fields[1].ty == Type::Integer(clust_semantics::IntegerType::Usize)
                && result == Some(Type::Tuple(fields.iter().map(|f| f.ty.clone()).collect()))
        }
        Operation::Unary(op @ ("raw_expose" | "raw_expose_mut"), value) => {
            matches!(operand_type(f,value),Some(Type::Pointer {mutable,pointee}) if mutable==(*op=="raw_expose_mut") && pointee.raw_storage_value() && crate::layout::layout(&pointee).is_ok_and(|l|l.size()!=0))
                && matches!(
                    result,
                    Some(Type::Integer(
                        clust_semantics::IntegerType::Usize | clust_semantics::IntegerType::Isize
                    ))
                )
        }
        Operation::Unary("raw_recover", value) => {
            matches!(
                operand_type(f, value),
                Some(Type::Integer(
                    clust_semantics::IntegerType::Usize | clust_semantics::IntegerType::Isize
                ))
            ) && matches!(&result,Some(Type::Pointer {pointee,..}) if pointee.raw_storage_value() && crate::layout::layout(pointee).is_ok_and(|l|l.size()!=0))
        }
        Operation::Unary("storage_allocate", count) => {
            operand_type(f, count) == Some(Type::Integer(clust_semantics::IntegerType::Usize))
                && matches!(&result, Some(Type::Pointer {mutable:true,pointee}) if pointee.raw_storage_value())
        }
        Operation::Unary(
            op @ ("raw_initialize"
            | "storage_initialize"
            | "storage_replace"
            | "storage_take"
            | "raw_take"
            | "raw_drop"
            | "storage_drop"
            | "storage_drop_range"
            | "storage_adopt"
            | "storage_release"
            | "storage_check_range"),
            args,
        ) => {
            let Some(Type::Tuple(fields)) = operand_type(f, args) else {
                return Err("invalid MIR storage operands");
            };
            let Some(Type::Pointer {
                mutable: true,
                pointee,
            }) = fields.first()
            else {
                return Err("invalid MIR storage capability");
            };
            let count = if matches!(*op, "storage_check_range" | "storage_drop_range") {
                4
            } else if matches!(
                *op,
                "raw_initialize" | "storage_initialize" | "storage_replace"
            ) {
                3
            } else {
                2
            };
            fields.len() == count
                && fields.get(1) == Some(&Type::Integer(clust_semantics::IntegerType::Usize))
                && (!matches!(*op, "storage_check_range" | "storage_drop_range")
                    || fields[2..]
                        .iter()
                        .all(|t| *t == Type::Integer(clust_semantics::IntegerType::Usize)))
                && pointee.raw_storage_value()
                && (!matches!(
                    *op,
                    "raw_initialize" | "storage_initialize" | "storage_replace"
                ) || fields.get(2) == Some(&**pointee))
                && result
                    == Some(
                        if matches!(*op, "raw_take" | "storage_take" | "storage_replace") {
                            (**pointee).clone()
                        } else {
                            Type::Unit
                        },
                    )
        }

        Operation::Unary(name @ ("raw_addr" | "raw_addr_mut"), Operand::Address(p)) => {
            let source = place_type(f, p);
            source.as_ref().is_some_and(Type::raw_copy_value)
                && result
                    == source.map(|t| Type::Pointer {
                        mutable: *name == "raw_addr_mut",
                        pointee: Box::new(t),
                    })
                && (*name != "raw_addr_mut" || writable(f, p))
        }
        Operation::Unary("raw_addr" | "raw_addr_mut", _) => false,
        Operation::Unary("raw_read", p) => {
            matches!(operand_type(f,p), Some(Type::Pointer { pointee, .. }) if pointee.raw_copy_value() && result.as_ref() == Some(&*pointee))
        }
        Operation::Unary("io_print" | "io_println", value) => {
            result == Some(Type::Unit)
                && matches!(
                    operand_type(f, value),
                    Some(
                        Type::Str
                            | Type::Bool
                            | Type::Char
                            | Type::Integer(_)
                            | Type::F32
                            | Type::F64
                    )
                )
        }
        Operation::Unary("weak_identity", value) => {
            matches!(value, Operand::Address(_))
                && matches!(value,Operand::Address(place) if matches!(place_type(f,place),Some(Type::Class {copy:false,..})))
                && result
                    == Some(Type::Tuple(vec![
                        Type::Integer(
                            clust_semantics::IntegerType::Usize
                        );
                        2
                    ]))
        }
        Operation::Unary("weak_alive", value) => {
            operand_type(f, value)
                == Some(Type::Tuple(vec![
                    Type::Integer(
                        clust_semantics::IntegerType::Usize
                    );
                    2
                ]))
                && result == Some(Type::Bool)
        }
        Operation::Binary("weak_access", token, callback) => {
            let valid_token = operand_type(f, token)
                == Some(Type::Tuple(vec![
                    Type::Integer(
                        clust_semantics::IntegerType::Usize
                    );
                    2
                ]));
            valid_token
                && match (operand_type(f, callback), &result) {
                    (
                        Some(Type::Function {
                            unsafe_: false,
                            parameters,
                            result: callback_result,
                            contract,
                        }),
                        Some(Type::Enum { variants, .. }),
                    ) => {
                        parameters.len() == 1
                            && !parameters[0].0
                            && matches!(parameters[0].1, Type::Class { copy: false, .. })
                            && !parameters[0].1.contains_views()
                            && contract.as_ref().is_none_or(|c| {
                                c.modes == [Some(loans::Access::Readonly)] && c.view.is_none()
                            })
                            && crate::check::weak::scalar_result(&callback_result)
                            && variants.len() == 2
                            && variants[0].name == "None"
                            && variants[0].fields.is_empty()
                            && variants[1].name == "Some"
                            && variants[1].fields == [*callback_result]
                    }
                    _ => false,
                }
        }
        Operation::Unary("str_byte_len", value) => {
            result == Some(Type::Integer(clust_semantics::IntegerType::Usize))
                && operand_type(f, value) == Some(Type::Str)
        }
        Operation::Unary("write_byte", value) => {
            result == Some(Type::Unit)
                && operand_type(f, value) == Some(Type::Integer(clust_semantics::IntegerType::U8))
        }
        Operation::Binary("str_byte_at", value, index) => {
            result == Some(Type::Integer(clust_semantics::IntegerType::U8))
                && operand_type(f, value) == Some(Type::Str)
                && operand_type(f, index)
                    == Some(Type::Integer(clust_semantics::IntegerType::Usize))
        }
        Operation::Unary("assert", value) => {
            result == Some(Type::Unit) && operand_type(f, value) == Some(Type::Bool)
        }
        Operation::Binary("raw_write", p, v) => {
            matches!(operand_type(f,p), Some(Type::Pointer { mutable:true, pointee }) if pointee.raw_copy_value() && operand_type(f,v).as_ref() == Some(&*pointee) && result == Some(Type::Unit))
        }
        Operation::Binary("checked_index", index, length) => {
            let expected = Some(Type::Integer(clust_semantics::IntegerType::Usize));
            result == expected
                && operand_type(f, index) == expected
                && operand_type(f, length) == expected
        }
        Operation::Binary("raw_difference", a, b) => {
            matches!((operand_type(f,a),operand_type(f,b)),(Some(Type::Pointer {pointee:a,..}),Some(Type::Pointer {pointee:b,..})) if a==b && a.raw_storage_value() && crate::layout::layout(&a).is_ok_and(|layout|layout.size()!=0))
                && result == Some(Type::Integer(clust_semantics::IntegerType::Isize))
        }
        Operation::Binary("raw_offset" | "raw_offset_sub", p, n) => {
            matches!(operand_type(f,p), Some(Type::Pointer { pointee, .. }) if pointee.raw_storage_value())
                && operand_type(f, p) == result
                && operand_type(f, n) == Some(Type::Integer(clust_semantics::IntegerType::Isize))
        }
        Operation::Cast(p, target)
            if matches!(target, Type::Pointer { .. })
                || matches!(operand_type(f, p), Some(Type::Pointer { .. })) =>
        {
            matches!((operand_type(f,p),target), (Some(Type::Pointer { mutable: a, pointee: x }), Type::Pointer { mutable: b, pointee: y }) if (!*b || a) && (x == *y || x.raw_reinterpret_to(y)) && x.raw_storage_value() && result.as_ref() == Some(target))
        }
        _ => true,
    };
    if valid {
        Ok(())
    } else {
        Err("invalid MIR raw pointer type or capability")
    }
}
