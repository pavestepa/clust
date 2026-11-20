use super::*;

impl RawAllocations {
    pub(crate) fn expose(
        &mut self,
        value: Value,
        writable: bool,
        target: &Type,
    ) -> Result<Value, TrapKind> {
        let Type::Integer(kind) = target else {
            return Err(TrapKind::InvalidIr);
        };
        let Value::Pointer(pointer) = value else {
            return Err(TrapKind::InvalidIr);
        };
        let address = if let Some(address) = pointer.address_only {
            address
        } else {
            let position = u64::try_from(pointer.byte_position()?).map_err(|_| TrapKind::Bounds)?;
            let existing = self
                .exposed
                .iter_mut()
                .find(|entry| entry.pointer.same_region(&pointer));
            if let Some(entry) = existing {
                entry.writable |= writable;
                entry.base.checked_add(position).ok_or(TrapKind::Bounds)?
            } else {
                let extent = u64::try_from(pointer.length * pointer.storage_stride()?)
                    .map_err(|_| TrapKind::Bounds)?;
                // Different projections of the same object expose the same
                // allocation address plus their physical field/element offsets.
                let known = self.exposed.iter().find(|entry| {
                    pointer.allocation_extent != 0 && entry.pointer.same_allocation(&pointer)
                });
                let base = if let Some(entry) = known {
                    entry
                        .base
                        .checked_sub(entry.pointer.region_offset)
                        .and_then(|base| base.checked_add(pointer.region_offset))
                        .ok_or(TrapKind::Bounds)?
                } else {
                    let base = self.next_address;
                    self.next_address = base
                        .checked_add(pointer.allocation_extent.max(extent))
                        .and_then(|n| n.checked_add(4096))
                        .and_then(|n| n.checked_add(15))
                        .map(|n| n & !15)
                        .ok_or(TrapKind::Allocation)?;
                    base.checked_add(pointer.region_offset)
                        .ok_or(TrapKind::Bounds)?
                };
                let mut root = pointer;
                root.index = 0;
                root.byte_in_element = 0;
                self.exposed.push(ExposedAddress {
                    base,
                    extent,
                    pointer: root,
                    writable,
                });
                base.checked_add(position).ok_or(TrapKind::Bounds)?
            }
        };
        Ok(Value::Integer(clust_semantics::Integer::from_bits(
            *kind,
            u128::from(address),
        )))
    }
}

impl RawAllocations {
    pub(crate) fn recover(&self, value: Value, target: &Type) -> Result<Value, TrapKind> {
        let Value::Integer(address) = value else {
            return Err(TrapKind::InvalidIr);
        };
        let Type::Pointer { mutable, pointee } = target else {
            return Err(TrapKind::InvalidIr);
        };
        let address = u64::try_from(address.bits()).map_err(|_| TrapKind::Bounds)?;
        let mut exposed = self.exposed.iter().collect::<Vec<_>>();
        exposed.sort_by_key(|entry| std::cmp::Reverse(entry.extent));
        for entry in exposed {
            if address < entry.base || address - entry.base > entry.extent {
                continue;
            }
            if *mutable && !entry.writable {
                continue;
            }
            let mut pointer = entry.pointer.clone();
            let stride = pointer.storage_stride()?;
            if stride == 0 {
                return Err(TrapKind::InvalidIr);
            }
            let position = i128::from(address - entry.base);
            pointer.index = position / stride;
            pointer.byte_in_element = position % stride;
            if let Some(pointer) = pointer.reinterpret(pointee) {
                return Ok(Value::Pointer(pointer));
            }
        }
        if self
            .exposed
            .iter()
            .any(|entry| address >= entry.base && address - entry.base <= entry.extent)
        {
            return Err(TrapKind::InvalidIr);
        }
        Ok(Value::Pointer(RawPointer {
            address_only: Some(address),
            storage_type: Rc::new((**pointee).clone()),
            view_type: Rc::new((**pointee).clone()),
            heap: None,
            storage: Default::default(),
            prefix: Rc::new(vec![]),
            region_offset: 0,
            allocation_extent: 0,
            index: 0,
            byte_in_element: 0,
            length: 0,
            array: false,
        }))
    }
}

impl RawAllocations {
    pub(crate) fn storage_view(&self, storage: Value, index: Value) -> Result<Value, TrapKind> {
        let Value::Aggregate(fields) = storage else {
            return Err(TrapKind::InvalidIr);
        };
        let [Value::Pointer(pointer), Value::Integer(capacity)] = fields.as_slice() else {
            return Err(TrapKind::InvalidIr);
        };
        let Value::Integer(index) = index else {
            return Err(TrapKind::InvalidIr);
        };
        if index.bits() >= capacity.bits() {
            return Err(TrapKind::Bounds);
        }
        let heap = pointer
            .heap
            .as_ref()
            .and_then(|h| h.upgrade())
            .ok_or(TrapKind::InvalidIr)?;
        let slot = heap
            .slots
            .borrow()
            .get(&(index.bits() as u64))
            .cloned()
            .ok_or(TrapKind::InvalidIr)?;
        let place = Reference {
            slot,
            projections: vec![],
        };
        place.read()?;
        Ok(Value::View(ViewValue(place)))
    }
}
