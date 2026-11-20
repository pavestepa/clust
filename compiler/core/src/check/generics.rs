mod arguments;
use super::*;
pub(super) mod substitute;
use substitute::Substitution;
#[derive(Clone, Default)]
pub(super) struct Monomorphization {
    pub result_contracts: ReturnContracts,
    pub consuming: Consuming,
    pub ownership_errors: Vec<Diagnostic>,
    pub entry_keys: BTreeMap<usize, String>,
    pub result_requests: BTreeMap<String, Type>,
    pub pending_results: std::collections::BTreeSet<usize>,
    pub bodies: BTreeMap<usize, a::Function>,
    pub entries: BTreeMap<String, Signature>,
    pub templates: BTreeMap<String, a::Function>,
    pub pending: Vec<(Signature, a::Function)>,
    pub next_id: usize,
    pub auditing: bool,
    pub audits: Vec<(Signature, a::Function)>,
    pub audit_keys: std::collections::BTreeSet<String>,
    pub instances: BTreeMap<String, Signature>,
    pub class_destructor_payloads: std::collections::BTreeSet<String>,
    pub destructor_types: std::collections::BTreeSet<String>,
}
impl Checker<'_> {
    pub(super) fn generic_call(
        &mut self,
        name: &str,
        types: &[a::TypeArgument],
        arguments: &[a::Expr],
        expected: Option<&Type>,
        span: Span,
    ) -> Result<h::Expr> {
        let template = self.mono.borrow().templates[name].clone();
        let (mut substitutions, checked) =
            self.generic_arguments(&template, types, arguments, expected, span)?;
        if !self.mono.borrow().auditing {
            let audit_key = format!("{name}:{:?}", substitutions.constants);
            if self.mono.borrow_mut().audit_keys.insert(audit_key.clone()) {
                let mut abstract_substitution = Substitution::default();
                abstract_substitution.constants = substitutions.constants.clone();
                for generic in &template.generics {
                    if let a::GenericKind::Type { constraints, .. } = &generic.kind {
                        let marker = self.nominals.marker(
                            format!("<generic {audit_key}:{}>", generic.name),
                            constraints.iter().any(interfaces::copy),
                        );
                        abstract_substitution
                            .types
                            .insert(generic.name.clone(), marker);
                    }
                }
                self.nominals.interfaces.prepare_markers(
                    &template.generics,
                    &abstract_substitution,
                    self.constants,
                    self.nominals,
                )?;
                for marker in abstract_substitution.types.values() {
                    for method in self.nominals.interfaces.marker_methods(
                        marker,
                        self.constants,
                        self.nominals,
                    )? {
                        self.mono
                            .borrow_mut()
                            .templates
                            .insert(method.name.clone(), method);
                    }
                }
                let mut declaration = template.clone();
                abstract_substitution.function(&mut declaration);
                declaration.generics.clear();
                let parameters = declaration
                    .parameters
                    .iter()
                    .map(|p| {
                        resolve_type_with(p.ty.as_ref().unwrap(), self.constants, self.nominals)
                    })
                    .collect::<Result<Vec<_>>>()?;
                let result = resolve_type_with(&declaration.result, self.constants, self.nominals)?;
                // Audit generic bodies through the owned entry of consuming
                // parameters: the checks must hold for an owner caller.
                let mut audit_modes = methods::parameter_modes(&declaration, &parameters);
                for (index, mode) in audit_modes.iter_mut().enumerate() {
                    if self.consuming_parameter(&declaration, index) {
                        *mode = None;
                    }
                }
                self.mono.borrow_mut().audits.push((
                    Signature {
                        modes: audit_modes,
                        id: 0,
                        parameters,
                        result,
                    },
                    declaration,
                ));
            }
        }
        let key = format!(
            "{name}:{:?}:{:?}",
            substitutions.types, substitutions.constants
        );
        let mut mono = self.mono.borrow_mut();
        let signature = if let Some(signature) = mono.instances.get(&key) {
            signature.clone()
        } else {
            if mono.instances.len() >= 256 {
                return Err(error(
                    span,
                    "E0372",
                    "specialization limit exceeded; possible infinitely growing generic recursion",
                ));
            }
            let mut function = template;
            substitutions.function(&mut function);
            function.generics.clear();
            let parameters = function
                .parameters
                .iter()
                .map(|p| resolve_type_with(p.ty.as_ref().unwrap(), self.constants, self.nominals))
                .collect::<Result<Vec<_>>>()?;
            let result = resolve_type_with(&function.result, self.constants, self.nominals)?;
            let id = mono.next_id;
            mono.next_id += 1;
            function.name = format!("{name}$g{id}");
            let signature = Signature {
                modes: consume::parameter_modes(&function, &parameters, &mono.consuming),
                id,
                parameters,
                result,
            };
            mono.instances.insert(key, signature.clone());
            mono.bodies.insert(id, function.clone());
            if !mono.auditing || function.body.is_some() {
                mono.pending.push((signature.clone(), function));
            }
            signature
        };
        for (arg, ty) in checked.iter().zip(&signature.parameters) {
            if !self.class_subtype(ty, &arg.ty) {
                same(ty, &arg.ty, arg.span)?;
            }
        }
        if let Some(expected) = expected {
            same(expected, &signature.result, span)?;
        }
        drop(mono);
        self.call_entry(signature, arguments, checked, span)
    }
}

mod complexity;
mod inference;
use self::complexity::validate_complexity;
use self::inference::infer;
