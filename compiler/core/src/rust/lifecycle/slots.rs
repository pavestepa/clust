use super::*;

impl Emitter {
    pub(in super::super) fn storage(&self, f: &Function) -> String {
        let mut out = String::new();
        for (id, local) in f
            .locals
            .iter()
            .enumerate()
            .filter(|(_, l)| l.view.is_none() && self.resource(&l.ty))
        {
            let t = self.type_id(&local.ty);
            out.push_str(&format!("let mut state{id}=S{t}::VACANT;let s{id}=std::ptr::addr_of_mut!(state{id});let mut stamp{id}=0u64;\n"));
            if f.parameters.contains(&id) {
                out.push_str(&format!("s{id}.write(S{t}::LIVE);stamp{id}=ctx.stamp();\n"));
            }
        }
        out
    }
}

impl Emitter {
    pub(in super::super) fn end_storage(&self, locals: &[usize], f: &Function) -> String {
        let mut out = String::from("let was_panicking=pending.is_some();\n");
        for (_, mut selected) in f.cleanup.groups(locals) {
            selected.retain(|id| f.locals[*id].view.is_none() && self.resource(&f.locals[*id].ty));
            if selected.is_empty() {
                continue;
            }
            {
                out.push_str(&format!("let mut order=[{}];order.sort_unstable_by_key(|v|std::cmp::Reverse(v.0));for (_,id) in order {{match id {{\n",selected.iter().map(|id|format!("(stamp{id},{id}usize)")).collect::<Vec<_>>().join(",")));
                for id in selected {
                    out.push_str(&format!("{id}=>{{{}}},", self.drop_local(id, f)));
                }
                out.push_str("_=>unreachable!(),}}\n");
            }
        }
        let unwind=f.blocks.iter().position(|b|matches!(&b.terminator,Terminator::EndStorage {locals,next} if locals.len()==f.locals.len() && matches!(f.blocks[*next].terminator,Terminator::ResumePanic))).unwrap();
        out.push_str(&format!(
            "if !was_panicking && pending.is_some() {{pc={unwind};continue;}}\n"
        ));
        out
    }
}

impl Emitter {
    pub(super) fn drop_local(&self, id: usize, f: &Function) -> String {
        let t = self.type_id(&f.locals[id].ty);
        format!("if let Err(e)=drop{t}(ctx,l{id},s{id},fault) {{if pending.is_none() {{pending=Some(e);}}}}\n")
    }
}
