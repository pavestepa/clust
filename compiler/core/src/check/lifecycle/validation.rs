use super::*;

pub(in super::super) fn validate(program: &h::Program) -> Result<Destructors> {
    if !program.functions.iter().any(|f| is_destructor(&f.name)) {
        return Ok(Destructors::default());
    }
    let mir = crate::mir::lower(program);
    let summaries = effects::infer(&mir);
    let mut registry = Destructors::default();
    for (id, function) in program.functions.iter().enumerate() {
        if !is_destructor(&function.name) {
            continue;
        }
        let receiver = function.parameters[0];
        if !effects::permits_finalizing(&summaries[id], 0) {
            return Err(error(function.span,"E0391","destructor cannot replace its finalizing target, including through aliases or calls"));
        }
        registry
            .insert(&function.locals[receiver].ty, id)
            .map_err(|message| error(function.span, "E0390", message))?;
    }
    // A user destructor may read all fields. Extracting an owned subfield would
    // leave its body with an invalid target. Whole moves remain permitted.
    for function in &mir.functions {
        for block in &function.blocks {
            let crate::mir::Terminator::Evaluate { operation, .. } = &block.terminator else {
                continue;
            };
            for place in crate::mir::dataflow::consumed(function, operation) {
                let mut ty = function.locals[place.local].ty.clone();
                for projection in &place.projections {
                    if registry.entry(&ty).is_some()
                        && !matches!(projection, crate::mir::Projection::Dereference)
                    {
                        return Err(error(
                            block.span,
                            "E0392",
                            "cannot move an owned field out of a type with a destructor",
                        ));
                    }
                    ty = match (projection, &ty) {
                        (crate::mir::Projection::Dereference, _) => ty.clone(),
                        (crate::mir::Projection::Field(i), Type::Class { .. }) => {
                            program.classes[&ty.class_key()].fields[*i].ty.clone()
                        }
                        (crate::mir::Projection::Field(i), Type::Record { fields, .. }) => {
                            fields[*i].ty.clone()
                        }
                        (crate::mir::Projection::Field(i), Type::Tuple(fields)) => {
                            fields[*i].clone()
                        }
                        (crate::mir::Projection::Variant(i), Type::Enum { variants, .. }) => {
                            Type::Tuple(variants[*i].fields.clone())
                        }
                        (
                            crate::mir::Projection::Index(_)
                            | crate::mir::Projection::ConstantIndex(_),
                            Type::Array(element, _),
                        ) => (**element).clone(),
                        _ => {
                            return Err(error(block.span, "E0392", "invalid destructor move path"))
                        }
                    };
                }
            }
        }
    }
    Ok(registry)
}
