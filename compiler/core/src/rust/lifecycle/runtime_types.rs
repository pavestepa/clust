use super::*;

impl Emitter {
    pub(in super::super) fn lifecycle_types(&self) -> String {
        let mut out=String::from("struct Context { clock:u64, panicking:bool, weak:std::collections::BTreeMap<usize,(u64,usize)> }\nimpl Context {fn raise(&mut self) {if self.panicking {std::process::abort();} self.panicking=true;} fn stamp(&mut self)->u64 {self.clock=self.clock.checked_add(1).unwrap_or_else(||std::process::abort());self.clock}}\n");
        for (id, ty) in self
            .types
            .iter()
            .enumerate()
            .filter(|(_, t)| self.resource(t))
        {
            if let Type::Class { .. } = ty {
                if let Type::Class { name, .. } = ty {
                    for (helper, leaf) in [
                        ("identity", "heap as usize".to_owned()),
                        ("identity_type", id.to_string()),
                        (
                            "identity_live",
                            "usize::from(!(*heap).finalizing)".to_owned(),
                        ),
                    ] {
                        if name.starts_with("$poly$") {
                            let fields = &self.classes[&ty.class_key()].fields;
                            let Type::Enum { variants, .. } = &fields[0].ty else {
                                unreachable!()
                            };
                            out.push_str(&format!("unsafe fn {helper}{id}(heap:*mut H{id})->usize {{let cases=std::ptr::addr_of!((*(*heap).data.as_ptr()).f0);match (*cases).tag {{"));
                            for (index, variant) in variants.iter().enumerate() {
                                let (target, stored) = match &variant.fields[0] {
                                    Type::StoredView(target) => (&**target, true),
                                    target => (target, false),
                                };
                                let pointer = format!(
                                    "(*std::ptr::addr_of!((*cases).data.v{index}).cast::<{}>()).f0{}",
                                    self.ty(&Type::Tuple(variant.fields.clone())),
                                    if stored { ".data" } else { "" }
                                );
                                out.push_str(&format!(
                                    "{index}=>{helper}{}({pointer}),",
                                    self.type_id(target)
                                ));
                            }
                            out.push_str("_=>std::process::abort(),}}\n");
                        } else {
                            out.push_str(&format!(
                                "unsafe fn {helper}{id}(heap:*mut H{id})->usize {{{leaf}}}\n"
                            ));
                        }
                    }
                }
                let payload = self.class_payload(&ty.class_key());
                let payload_id = self.type_id(&payload);
                let field_count = self.classes[&ty.class_key()].fields.len();
                let data = self.ty(&payload);
                let (state_ty, state_init) = if self.resource(&payload) {
                    (
                        format!("S{payload_id}"),
                        format!("{{let mut s=S{payload_id}::VACANT;s.live=true;s}}"),
                    )
                } else {
                    ("()".into(), "()".into())
                };
                out.push_str(&format!("struct H{id} {{data:MaybeUninit<{data}>,state:{state_ty},complete:bool,finalizing:bool,heap_owned:bool,order:[u64;{field_count}]}}\nfn empty{id}()->H{id} {{H{id} {{data:MaybeUninit::uninit(),state:{state_init},complete:false,finalizing:false,heap_owned:false,order:[0;{field_count}]}}}}\nunsafe fn new{id}()->*mut H{id} {{let heap=Box::into_raw(Box::new(H{id} {{data:MaybeUninit::uninit(),state:{state_init},complete:false,finalizing:false,heap_owned:true,order:[0;{field_count}]}}));\n// class-allocation\nheap}}\n#[derive(Clone,Copy)] struct S{id} {{live:bool}}\nimpl S{id} {{const VACANT:Self=Self {{live:false}};const LIVE:Self=Self {{live:true}};}}\n#[derive(Clone,Copy)] struct V{id} {{data:*mut H{id}}}\n#[derive(Clone,Copy)] struct W{id} {{data:*mut *mut H{id},state:*mut S{id},stamp:*mut u64}}\nunsafe fn drop{id}(ctx:&mut Context,p:*mut *mut H{id},s:*mut S{id},fault:Fault)->Result<(),Fault> {{if !(*s).live {{return Ok(());}}(*s).live=false;let heap=p.read();(*heap).finalizing=true;ctx.weak.remove(&(heap as usize));let mut pending=None;\n"));
                if let Some(entry) = self.destructors.entry(ty) {
                    out.push_str(&format!("if (*heap).complete {{\n// class-destructor-entry {entry}\nif let Err(e)=f{entry}(ctx,V{id} {{data:heap}}) {{pending=Some(e);}}}}\n"));
                }
                if self.resource(&payload) {
                    out.push_str(&format!("if (*heap).complete {{if let Err(e)=drop{payload_id}(ctx,std::ptr::addr_of_mut!((*heap).data).cast(),std::ptr::addr_of_mut!((*heap).state),fault) {{if pending.is_none() {{pending=Some(e);}}}}}} else {{\n"));
                    let fields = &self.classes[&ty.class_key()].fields;
                    let resources: Vec<_> = fields
                        .iter()
                        .enumerate()
                        .filter(|(_, field)| self.resource(&field.ty))
                        .collect();
                    out.push_str(&format!("let mut order=[{}];order.sort_unstable_by_key(|value|std::cmp::Reverse(value.0));for (_,index) in order {{match index {{\n",resources.iter().map(|(index,_)|format!("((*heap).order[{index}],{index}usize)")).collect::<Vec<_>>().join(",")));
                    for (index, field) in resources {
                        let child = self.type_id(&field.ty);
                        out.push_str(&format!("{index}=>{{if let Err(e)=drop{child}(ctx,std::ptr::addr_of_mut!((*(*heap).data.as_mut_ptr()).f{index}),std::ptr::addr_of_mut!((*heap).state.f{index}),fault) {{if pending.is_none() {{pending=Some(e);}}}}}},\n"));
                    }
                    out.push_str("_=>unreachable!(),}}}\n");
                }
                out.push_str("if (*heap).heap_owned {\n// class-deallocation\ndrop(Box::from_raw(heap));}match pending {Some(e)=>Err(e),None=>Ok(())}}\n");
                continue;
            }
            let mut members = Vec::new();
            match ty {
                Type::Record { fields, .. } => {
                    for (i, f) in fields.iter().enumerate() {
                        if self.resource(&f.ty) {
                            members.push((format!("f{i}"), self.type_id(&f.ty), None));
                        }
                    }
                }
                Type::Tuple(fields) => {
                    for (i, f) in fields.iter().enumerate() {
                        if self.resource(f) {
                            members.push((format!("f{i}"), self.type_id(f), None));
                        }
                    }
                }
                Type::Array(t, n) => members.push(("elements".into(), self.type_id(t), Some(*n))),
                Type::Enum { variants, .. } => {
                    for (i, v) in variants.iter().enumerate() {
                        let t = Type::Tuple(v.fields.clone());
                        if self.resource(&t) {
                            members.push((format!("v{i}"), self.type_id(&t), None));
                        }
                    }
                }
                _ => unreachable!(),
            }
            out.push_str(&format!("#[derive(Clone,Copy)] struct S{id} {{live:bool,"));
            for (name, child, count) in &members {
                out.push_str(&format!(
                    "{name}:{},",
                    count.map_or(format!("S{child}"), |n| format!("[S{child};{n}]"))
                ));
            }
            out.push_str(&format!("}}\nimpl S{id} {{"));
            for (name, live) in [("VACANT", false), ("LIVE", true)] {
                out.push_str(&format!("const {name}:Self=Self {{live:{live},"));
                for (field, child, count) in &members {
                    out.push_str(&format!(
                        "{field}:{},",
                        count.map_or(format!("S{child}::{name}"), |n| format!(
                            "[S{child}::{name};{n}]"
                        ))
                    ));
                }
                out.push_str("};");
            }
            out.push_str(&format!("}}\n#[derive(Clone,Copy)] struct V{id} {{data:*mut {},state:*mut S{id},stamp:*mut u64}}\n",self.ty(ty)));
            out.push_str(&format!("unsafe fn drop{id}(ctx:&mut Context,p:*mut {},s:*mut S{id},fault:Fault)->Result<(),Fault> {{if !(*s).live {{return Ok(());}} (*s).live=false;let mut pending=None;\n",self.ty(ty)));
            if let Some(entry) = self.destructors.entry(ty) {
                out.push_str(&format!("// destructor-entry {entry}\nif let Err(e)=f{entry}(ctx,V{id} {{data:p,state:s,stamp:std::ptr::null_mut()}}) {{pending=Some(e);}}\n"));
            }
            for (field, child, count) in &members {
                let (data, state) = if let Some(count) = count {
                    out.push_str(&format!("for index in 0..{count}usize {{\n"));
                    (
                        format!("p.cast::<{}>().add(index)", self.ty(&self.types[*child])),
                        format!(
                            "std::ptr::addr_of_mut!((*s).elements).cast::<S{child}>().add(index)"
                        ),
                    )
                } else if matches!(ty, Type::Enum { .. }) {
                    let variant = field.trim_start_matches('v');
                    out.push_str(&format!("if (*p).tag=={variant} {{\n"));
                    (
                        format!("std::ptr::addr_of_mut!((*p).data.{field}).cast()"),
                        format!("std::ptr::addr_of_mut!((*s).{field})"),
                    )
                } else {
                    (
                        format!("std::ptr::addr_of_mut!((*p).{field})"),
                        format!("std::ptr::addr_of_mut!((*s).{field})"),
                    )
                };
                out.push_str(&format!("if let Err(e)=drop{child}(ctx,{data},{state},fault) {{if pending.is_none() {{pending=Some(e);}}}}\n"));
                if count.is_some() || matches!(ty, Type::Enum { .. }) {
                    out.push_str("}\n");
                }
            }
            out.push_str("match pending {Some(e)=>Err(e),None=>Ok(())}}\n");
        }
        out
    }
}
