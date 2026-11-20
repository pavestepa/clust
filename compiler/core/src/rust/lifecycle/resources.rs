use super::*;

impl Emitter {
    pub(in super::super) fn evaluate_resource(
        &self,
        operation: &Operation,
        destination: &Place,
        f: &Function,
        unwind: usize,
    ) -> String {
        let (dest, ty) = self.place(destination, f);
        let expression = if matches!(operation, Operation::AllocateClass(_))
            && self.stack_slots.contains(&destination.local)
        {
            let id = self.type_id(&ty);
            let local = destination.local;
            format!("{{ frame{local}.write(empty{id}()); frame{local}.as_mut_ptr() }}")
        } else {
            self.operation(operation, &ty, f)
        };
        if matches!(
            operation,
            Operation::Borrow(..) | Operation::CapturePlace(..)
        ) {
            return format!("let result:Result<(),Fault>=(||{{let value={expression};({dest}).write(value);Ok(())}})();if let Err(e)=result {{ctx.raise();pending=Some(e);pc={unwind};continue;}}\n");
        }
        let forgotten = mir::dataflow::consumed(f, operation)
            .iter()
            .map(|p| format!("let _={};{}", self.place(p, f).0, self.forget(p, f)))
            .collect::<String>();
        let assignment = if mir::returns::call_view(operation).is_some()
            || matches!(operation, Operation::LoadView(_))
        {
            "target.write(value);".into()
        } else if self.resource(&ty) {
            let t = self.type_id(&ty);
            let (state, stamp) = self.state_place(destination, f);
            format!("let state={state};if let Err(e)=drop{t}(ctx,target,state,fault) {{let mut staged=S{t}::LIVE;let _=drop{t}(ctx,std::ptr::addr_of_mut!(value),std::ptr::addr_of_mut!(staged),fault);return Err(e);}}target.write(value);state.write(S{t}::LIVE);let stamp:*mut u64={stamp};if !stamp.is_null() {{stamp.write(ctx.stamp());}}")
        } else {
            "target.write(value);".into()
        };
        let raise = if matches!(
            operation,
            Operation::Call(..)
                | Operation::CallIndirect(..)
                | Operation::CallView { .. }
                | Operation::CallStorage { .. }
        ) {
            ""
        } else {
            "if entered_panicking || !ctx.panicking {ctx.raise();}"
        };
        format!("let entered_panicking=ctx.panicking;let result:Result<(),Fault>=(||{{let target={dest};{forgotten}let mut value={expression};{assignment}Ok(())}})();if let Err(e)=result {{{raise}pending=Some(e);pc={unwind};continue;}}\n")
    }
}
