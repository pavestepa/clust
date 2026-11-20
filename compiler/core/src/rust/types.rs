use super::*;

impl Emitter {
    pub(super) fn class_payload(&self, name: &str) -> Type {
        Type::Record {
            name: format!("<payload {name}>"),
            copy: false,
            fields: self.classes[name].fields.clone(),
        }
    }
}

impl Emitter {
    pub(super) fn register(&mut self, ty: &Type) {
        if self.types.contains(ty) {
            return;
        }
        self.types.push(ty.clone());
        match ty {
            Type::Class { .. } => self.register(&self.class_payload(&ty.class_key())),
            Type::Record { fields, .. } => {
                for f in fields {
                    self.register(&f.ty);
                }
            }
            Type::Enum { variants, .. } => {
                for v in variants {
                    self.register(&Type::Tuple(v.fields.clone()));
                }
            }
            Type::Tuple(fields) => {
                for t in fields {
                    self.register(t);
                }
            }
            Type::Array(t, _) | Type::StoredView(t) | Type::Pointer { pointee: t, .. } => {
                self.register(t)
            }
            Type::Function {
                parameters, result, ..
            } => {
                for (_, t) in parameters {
                    self.register(t);
                }
                self.register(result);
            }
            _ => {}
        }
    }
}

impl Emitter {
    pub(super) fn ty(&self, ty: &Type) -> String {
        match ty {
            Type::Pointer { mutable, pointee } => format!(
                "*{} {}",
                if *mutable { "mut" } else { "const" },
                self.ty(pointee)
            ),
            Type::StoredView(target) => {
                if self.resource(target) {
                    format!("V{}", self.type_id(target))
                } else {
                    format!("*mut {}", self.ty(target))
                }
            }
            Type::Class { .. } => {
                format!("*mut H{}", self.types.iter().position(|t| t == ty).unwrap())
            }
            Type::Unit => "()".into(),
            Type::Bool => "bool".into(),
            Type::Char => "char".into(),
            Type::Str => "&'static str".into(),
            Type::F32 => "f32".into(),
            Type::F64 => "f64".into(),
            Type::Integer(t) => format!("{t:?}").to_lowercase(),
            Type::Record { .. } | Type::Enum { .. } | Type::Tuple(_) => {
                format!("T{}", self.types.iter().position(|t| t == ty).unwrap())
            }
            Type::Array(element, count) => format!("[{}; {count}]", self.ty(element)),
            Type::Function {
                parameters,
                result,
                contract,
                ..
            } => format!(
                "unsafe fn({}{}) -> Result<{}, Fault>",
                if self.resources { "&mut Context," } else { "" },
                parameters
                    .iter()
                    .enumerate()
                    .map(|(i, (_, ty))| {
                        // Without a known entry, non-Copy parameters are borrowed.
                        if contract
                            .as_ref()
                            .map_or(!ty.copyable(), |c| c.modes[i].is_some())
                        {
                            self.borrow_ty(ty)
                        } else {
                            self.ty(ty)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(","),
                if contract.as_ref().is_some_and(|c| c.view.is_some()) {
                    self.borrow_ty(result)
                } else {
                    self.ty(result)
                }
            ),
        }
    }
}

impl Emitter {
    pub(super) fn borrow_ty(&self, ty: &Type) -> String {
        if self.resource(ty) {
            format!("V{}", self.type_id(ty))
        } else {
            format!("*mut {}", self.ty(ty))
        }
    }
}

impl Emitter {
    pub(super) fn captured_slot(&self, f: &Function, local: usize) -> bool {
        f.blocks.iter().any(|b| matches!(&b.terminator, Terminator::Evaluate {operation:Operation::CapturePlace(_),destination,..} if destination.local==local))
    }
}

impl Emitter {
    pub(super) fn return_ty(&self, f: &Function) -> String {
        if f.result_view.is_some() {
            if self.resource(&f.result) {
                format!(
                    "V{}",
                    self.types.iter().position(|t| t == &f.result).unwrap()
                )
            } else {
                format!("*mut {}", self.ty(&f.result))
            }
        } else {
            self.ty(&f.result)
        }
    }
}

impl Emitter {
    pub(super) fn local_ty(&self, local: &hir::Local, capture: bool) -> String {
        if capture && matches!(local.ty, Type::Class { .. }) {
            return format!(
                "W{}",
                self.types.iter().position(|t| t == &local.ty).unwrap()
            );
        }
        if local.view.is_some() && self.resource(&local.ty) {
            format!(
                "V{}",
                self.types.iter().position(|t| t == &local.ty).unwrap()
            )
        } else if local.view.is_some() {
            format!("*mut {}", self.ty(&local.ty))
        } else {
            self.ty(&local.ty)
        }
    }
}
