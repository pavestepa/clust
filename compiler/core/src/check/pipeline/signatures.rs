use super::*;
pub(super) struct Declarations {
    pub signatures: BTreeMap<String, Signature>,
    pub declarations: Vec<a::Function>,
    pub templates: BTreeMap<String, a::Function>,
}
pub(super) fn collect(
    module: &a::Module,
    constants: &crate::constants::Constants,
    nominals: &records::Nominals,
    consuming: &Consuming,
) -> std::result::Result<Declarations, Vec<Diagnostic>> {
    let mut signatures = BTreeMap::new();
    let mut declarations = Vec::new();
    let mut errors = Vec::new();
    let mut templates = BTreeMap::new();
    for item in &module.items {
        if matches!(
            item.kind,
            a::ItemKind::Const { .. }
                | a::ItemKind::Aggregate { .. }
                | a::ItemKind::Enum { .. }
                | a::ItemKind::Interface { .. }
        ) {
            continue;
        }
        let a::ItemKind::Function(function) = &item.kind else {
            errors.push(error(
                item.span,
                "E0900",
                "this declaration is not yet implemented in the value checker",
            ));
            continue;
        };
        if constants.contains_key(&function.name) {
            errors.push(error(
                function.span,
                "E0301",
                "function conflicts with a constant declaration",
            ));
            continue;
        }
        if !function.generics.is_empty() && !function.c_abi {
            if function.name == "main" {
                errors.push(error(
                    item.span,
                    "E0303",
                    "main cannot have generic parameters",
                ));
            }

            if templates
                .insert(function.name.clone(), function.clone())
                .is_some()
                || signatures.contains_key(&function.name)
            {
                errors.push(error(item.span, "E0301", "duplicate function declaration"));
            }
            continue;
        }
        if function.c_abi {
            errors.push(error(
                item.span,
                "E0900",
                "foreign function checking is not implemented yet",
            ));
            continue;
        }
        let signature = (|| {
            let mut parameters = Vec::new();
            for param in &function.parameters {
                let ty = param
                    .ty
                    .as_ref()
                    .ok_or_else(|| error(param.span, "E0300", "unexpected receiver"))?;
                parameters.push(resolve_type_with(ty, constants, nominals)?);
            }
            Ok(Signature {
                modes: consume::parameter_modes(function, &parameters, consuming),
                id: declarations.len(),
                parameters,
                result: resolve_type_with(&function.result, constants, nominals)?,
            })
        })();
        match signature {
            Ok(signature) => {
                if templates.contains_key(&function.name) {
                    errors.push(error(item.span, "E0301", "duplicate function declaration"));
                }
                if signatures
                    .insert(function.name.clone(), signature)
                    .is_some()
                {
                    errors.push(error(
                        function.span,
                        "E0301",
                        "duplicate function declaration",
                    ));
                }
                declarations.push(function.clone());
            }
            Err(e) => errors.push(e),
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(Declarations {
        signatures,
        declarations,
        templates,
    })
}
