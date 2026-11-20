use super::*;

impl RawAllocations {
    pub(crate) fn operation(&mut self, op: &str, argument: Value) -> Result<Value, TrapKind> {
        let Value::Aggregate(mut args) = argument else {
            return Err(TrapKind::InvalidIr);
        };
        if op == "storage_move_slots" {
            let [Value::Pointer(destination), Value::Integer(dc), Value::Integer(ds), Value::Pointer(source), Value::Integer(sc), Value::Integer(ss), Value::Integer(count)] =
                args.as_slice()
            else {
                return Err(TrapKind::InvalidIr);
            };
            let target = destination
                .heap
                .as_ref()
                .and_then(|h| h.upgrade())
                .ok_or(TrapKind::InvalidIr)?;
            let origin = source
                .heap
                .as_ref()
                .and_then(|h| h.upgrade())
                .ok_or(TrapKind::InvalidIr)?;
            let (dc, ds, sc, ss, count) =
                (dc.bits(), ds.bits(), sc.bits(), ss.bits(), count.bits());
            if destination.byte_in_element != 0
                || source.byte_in_element != 0
                || destination.index != 0
                || source.index != 0
                || dc != target.capacity as u128
                || sc != origin.capacity as u128
            {
                return Err(TrapKind::InvalidIr);
            }
            if ds > dc || count > dc - ds || ss > sc || count > sc - ss {
                return Err(TrapKind::Bounds);
            }
            let same = Rc::ptr_eq(&target, &origin);
            // Validate every slot before transferring any ownership.
            for index in 0..count {
                if !origin
                    .slots
                    .borrow()
                    .get(&((ss + index) as u64))
                    .is_some_and(|s| s.borrow().is_some())
                {
                    return Err(TrapKind::InvalidIr);
                }
                let dest = ds + index;
                if !(same && dest >= ss && dest < ss + count)
                    && target
                        .slots
                        .borrow()
                        .get(&(dest as u64))
                        .is_some_and(|s| s.borrow().is_some())
                {
                    return Err(TrapKind::InvalidIr);
                }
            }
            if same && ds == ss {
                return Ok(Value::Unit);
            }
            let values: Vec<_> = (0..count)
                .map(|i| {
                    origin
                        .slots
                        .borrow()
                        .get(&((ss + i) as u64))
                        .unwrap()
                        .borrow_mut()
                        .take()
                        .unwrap()
                })
                .collect();
            for (i, value) in values.into_iter().enumerate() {
                let mut slots = target.slots.borrow_mut();
                let slot = slots.entry((ds + i as u128) as u64).or_insert_with(|| {
                    typed_slot((*destination.storage_type).clone(), target.clock.clone())
                });
                slot.0
                    .raw_origin
                    .replace(Some(destination.clone().offset((ds + i as u128) as i128)?));
                slot.initialize()?;
                *slot.borrow_mut() = Some(value);
            }
            return Ok(Value::Unit);
        }
        let (Some(Value::Pointer(pointer)), Some(Value::Integer(index))) =
            (args.first(), args.get(1))
        else {
            return Err(TrapKind::InvalidIr);
        };
        if op == "raw_take" {
            return pointer.clone().offset(index.bits() as i128)?.take();
        }
        if op == "raw_initialize" {
            if args.len() != 3 {
                return Err(TrapKind::InvalidIr);
            }
            let pointer = pointer.clone().offset(index.bits() as i128)?;
            return pointer.initialize(args.pop().unwrap());
        }
        let heap = pointer
            .heap
            .as_ref()
            .and_then(|h| h.upgrade())
            .ok_or(TrapKind::InvalidIr)?;
        if pointer.byte_in_element != 0 || pointer.index != 0 {
            return Err(TrapKind::InvalidIr);
        }
        let index = u64::try_from(index.bits()).map_err(|_| TrapKind::Bounds)?;
        if matches!(op, "storage_address" | "storage_address_mut") {
            let [_, _, Value::Integer(offset)] = args.as_slice() else {
                return Err(TrapKind::InvalidIr);
            };
            if index != heap.capacity || offset.bits() > index as u128 {
                return Err(TrapKind::Bounds);
            }
            return pointer
                .clone()
                .offset(offset.bits() as i128)
                .map(Value::Pointer);
        }
        if op == "storage_check_range" {
            let [_, _, Value::Integer(start), Value::Integer(count)] = args.as_slice() else {
                return Err(TrapKind::InvalidIr);
            };
            if index != heap.capacity
                || start.bits() > index as u128
                || count.bits() > index as u128 - start.bits()
            {
                return Err(TrapKind::Bounds);
            }
            return Ok(Value::Unit);
        }
        if op == "storage_adopt" && pointer.storage_type != pointer.view_type {
            return Err(TrapKind::InvalidIr);
        }
        if matches!(op, "storage_detach" | "storage_adopt") {
            let detaching = op == "storage_detach";
            if index != heap.capacity || heap.owned.get() != detaching {
                return Err(TrapKind::InvalidIr);
            }
            heap.owned.set(!detaching);
            return if detaching {
                Ok(Value::Aggregate(args))
            } else {
                Ok(Value::Unit)
            };
        }
        if op == "storage_release" {
            if index != heap.capacity
                || !heap.owned.get()
                || heap.slots.borrow().values().any(|s| s.borrow().is_some())
            {
                return Err(TrapKind::InvalidIr);
            }
            self.heaps.retain(|h| !Rc::ptr_eq(h, &heap));
            return Ok(Value::Unit);
        }
        if index >= heap.capacity {
            return Err(TrapKind::Bounds);
        }
        let mut slots = heap.slots.borrow_mut();
        match op {
            "storage_initialize" => {
                if slots.get(&index).is_some_and(|s| s.borrow().is_some()) || args.len() != 3 {
                    return Err(TrapKind::InvalidIr);
                }
                let slot = slots.entry(index).or_insert_with(|| {
                    typed_slot((*pointer.storage_type).clone(), heap.clock.clone())
                });
                slot.0
                    .raw_origin
                    .replace(Some(pointer.clone().offset(i128::from(index))?));
                slot.initialize()?;
                *slot.borrow_mut() = Some(args.pop().unwrap());
                Ok(Value::Unit)
            }
            "storage_replace" => {
                let target = slots.get_mut(&index).ok_or(TrapKind::InvalidIr)?;
                if args.len() != 3 {
                    return Err(TrapKind::InvalidIr);
                }
                target
                    .borrow_mut()
                    .replace(args.pop().unwrap())
                    .ok_or(TrapKind::InvalidIr)
            }
            "storage_take" => slots
                .get(&index)
                .ok_or(TrapKind::InvalidIr)?
                .borrow_mut()
                .take()
                .ok_or(TrapKind::InvalidIr),
            "storage_drop" => {
                slots
                    .get(&index)
                    .ok_or(TrapKind::InvalidIr)?
                    .borrow_mut()
                    .take()
                    .ok_or(TrapKind::InvalidIr)?;
                Ok(Value::Unit)
            }
            _ => Err(TrapKind::InvalidIr),
        }
    }
}
