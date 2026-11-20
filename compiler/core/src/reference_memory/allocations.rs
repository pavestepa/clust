use super::*;

impl RawAllocations {
    pub(crate) fn allocate(
        &mut self,
        ty: &Type,
        capacity: u64,
        clock: Clock,
    ) -> Result<Value, TrapKind> {
        crate::layout::layout(ty)
            .map_err(|_| TrapKind::InvalidIr)?
            .array(capacity)
            .map_err(|_| TrapKind::Allocation)?;
        let heap = Rc::new(RawHeap {
            clock,
            owned: Cell::new(true),
            capacity,
            slots: RefCell::new(Default::default()),
        });
        let pointer = RawPointer {
            address_only: None,
            storage_type: Rc::new(ty.clone()),
            view_type: Rc::new(ty.clone()),
            heap: Some(Rc::downgrade(&heap)),
            storage: Default::default(),
            prefix: Rc::new(vec![]),
            region_offset: 0,
            allocation_extent: crate::layout::layout(ty)
                .map_err(|_| TrapKind::InvalidIr)?
                .array(capacity)
                .map_err(|_| TrapKind::Allocation)?
                .size(),
            index: 0,
            byte_in_element: 0,
            length: capacity as i128,
            array: true,
        };
        self.heaps.push(heap);
        Ok(Value::Pointer(pointer))
    }
}

impl RawAllocations {
    pub(crate) fn drop_range_arguments(
        &self,
        argument: Value,
    ) -> Result<(Value, u64, u64), TrapKind> {
        let Value::Aggregate(args) = argument else {
            return Err(TrapKind::InvalidIr);
        };
        let [Value::Pointer(pointer), Value::Integer(capacity), Value::Integer(start), Value::Integer(count)] =
            args.as_slice()
        else {
            return Err(TrapKind::InvalidIr);
        };
        let heap = pointer
            .heap
            .as_ref()
            .and_then(|h| h.upgrade())
            .ok_or(TrapKind::InvalidIr)?;
        if pointer.byte_in_element != 0
            || pointer.index != 0
            || capacity.bits() != heap.capacity as u128
        {
            return Err(TrapKind::InvalidIr);
        }
        if start.bits() > capacity.bits() || count.bits() > capacity.bits() - start.bits() {
            return Err(TrapKind::Bounds);
        }
        let (start, count) = (start.bits() as u64, count.bits() as u64);
        if heap
            .slots
            .borrow()
            .range(start..start + count)
            .filter(|(_, s)| s.borrow().is_some())
            .count() as u64
            != count
        {
            return Err(TrapKind::InvalidIr);
        }
        Ok((Value::Pointer(pointer.clone()), start, count))
    }
}
