use super::*;

impl RawPointer {
    pub(super) fn bits_window(&self) -> Result<(i128, usize, usize), TrapKind> {
        let source = crate::layout::layout(&self.storage_type).map_err(|_| TrapKind::InvalidIr)?;
        let target = crate::layout::layout(&self.view_type).map_err(|_| TrapKind::InvalidIr)?;
        let position = self.byte_position()?;
        if position < 0
            || position + i128::from(target.size()) > self.length * i128::from(source.size())
            || position % i128::from(target.align()) != 0
            || source.align() < target.align()
        {
            return Err(TrapKind::Bounds);
        }
        Ok((position, source.size() as usize, target.size() as usize))
    }
}

impl RawPointer {
    pub(crate) fn read(&self) -> Result<Value, TrapKind> {
        if self.storage_type == self.view_type && self.byte_in_element == 0 {
            return self.read_storage();
        }
        let (position, stride, size) = self.bits_window()?;
        let mut bytes = Vec::with_capacity(size);
        let mut offset = 0;
        while offset < size {
            let absolute = position + offset as i128;
            let mut source = self.clone();
            source.index = absolute / stride as i128;
            source.byte_in_element = 0;
            let stored = bits_bytes(source.read_storage()?, &self.storage_type)?;
            let start = (absolute % stride as i128) as usize;
            let count = (size - offset).min(stride - start);
            bytes.extend_from_slice(&stored[start..start + count]);
            offset += count;
        }
        bits_value(&bytes, &self.view_type)
    }
}

impl RawPointer {
    pub(super) fn read_storage(&self) -> Result<Value, TrapKind> {
        if let Some(heap) = &self.heap {
            let heap = heap.upgrade().ok_or(TrapKind::InvalidIr)?;
            return heap
                .slots
                .borrow()
                .get(&(self.index as u64))
                .ok_or(TrapKind::InvalidIr)?
                .borrow()
                .clone()
                .ok_or(TrapKind::InvalidIr);
        }
        self.reference()?.read()
    }
}

impl RawPointer {
    pub(crate) fn initialize(&self, value: Value) -> Result<Value, TrapKind> {
        if self.index < 0
            || self.index >= self.length
            || self.byte_in_element != 0
            || crate::layout::layout(&self.view_type)
                .map_err(|_| TrapKind::InvalidIr)?
                .size()
                != self.storage_stride()? as u64
        {
            return Err(TrapKind::Bounds);
        }
        let value = if self.storage_type == self.view_type {
            value
        } else {
            bits_value(&bits_bytes(value, &self.view_type)?, &self.storage_type)?
        };
        if let Some(heap) = &self.heap {
            let heap = heap.upgrade().ok_or(TrapKind::InvalidIr)?;
            let mut slots = heap.slots.borrow_mut();
            let slot = slots
                .entry(self.index as u64)
                .or_insert_with(|| typed_slot((*self.storage_type).clone(), heap.clock.clone()));
            if slot.borrow().is_some() {
                return Err(TrapKind::InvalidIr);
            }
            let mut origin = self.clone();
            origin.view_type = self.storage_type.clone();
            slot.0.raw_origin.replace(Some(origin));
            slot.initialize()?;
            *slot.borrow_mut() = Some(value);
        } else {
            let reference = self.reference()?;
            if reference.slot.state().borrow().get(&reference.parts())
                != crate::mir::drop_plan::Initialization::Vacant
            {
                return Err(TrapKind::InvalidIr);
            }
            reference.write(value)?;
        }
        Ok(Value::Unit)
    }
}

impl RawPointer {
    pub(crate) fn take(&self) -> Result<Value, TrapKind> {
        // An owning operation must target an entire original slot, never a
        // byte window into another value. Byte access creates no new owner.
        if self.byte_in_element != 0
            || crate::layout::layout(&self.view_type)
                .map_err(|_| TrapKind::InvalidIr)?
                .size()
                != self.storage_stride()? as u64
        {
            return Err(TrapKind::InvalidIr);
        }
        // Decode before invalidating: a same-size reinterpret transfers the
        // requested typed value, not the original diagnostic representation.
        let value = self.read()?;
        if let Some(heap) = &self.heap {
            let heap = heap.upgrade().ok_or(TrapKind::InvalidIr)?;
            heap.slots
                .borrow()
                .get(&(self.index as u64))
                .ok_or(TrapKind::InvalidIr)?
                .borrow_mut()
                .take()
                .ok_or(TrapKind::InvalidIr)?;
            return Ok(value);
        }
        let reference = self.reference()?;
        reference.forget()?;
        if reference.projections.is_empty() {
            *reference.slot.borrow_mut() = None;
        } else {
            write(
                reference
                    .slot
                    .borrow_mut()
                    .as_mut()
                    .ok_or(TrapKind::InvalidIr)?,
                &reference.projections,
                Value::Unit,
            )?;
        }
        Ok(value)
    }
}

impl RawPointer {
    pub(crate) fn write(&self, value: Value) -> Result<Value, TrapKind> {
        if self.storage_type == self.view_type && self.byte_in_element == 0 {
            return self.write_storage(value);
        }
        let (position, stride, size) = self.bits_window()?;
        let bytes = bits_bytes(value, &self.view_type)?;
        if bytes.len() != size {
            return Err(TrapKind::InvalidIr);
        }
        let first = position / stride as i128;
        let last = (position + size as i128 - 1) / stride as i128;
        let mut updates = Vec::new();
        for index in first..=last {
            let mut destination = self.clone();
            destination.index = index;
            destination.byte_in_element = 0;
            let mut stored = bits_bytes(destination.read_storage()?, &self.storage_type)?;
            for (offset, byte) in stored.iter_mut().enumerate() {
                let relative = index * stride as i128 + offset as i128 - position;
                if relative >= 0 && relative < size as i128 {
                    *byte = bytes[relative as usize];
                }
            }
            updates.push((destination, bits_value(&stored, &self.storage_type)?));
        }
        for (destination, value) in updates {
            destination.write_storage(value)?;
        }
        Ok(Value::Unit)
    }
}

impl RawPointer {
    pub(super) fn write_storage(&self, value: Value) -> Result<Value, TrapKind> {
        if let Some(heap) = &self.heap {
            let heap = heap.upgrade().ok_or(TrapKind::InvalidIr)?;
            let mut slots = heap.slots.borrow_mut();
            let slot = slots
                .get_mut(&(self.index as u64))
                .ok_or(TrapKind::InvalidIr)?;
            let mut stored = slot.borrow_mut();
            if stored.is_none() {
                return Err(TrapKind::InvalidIr);
            }
            *stored = Some(value);
            return Ok(Value::Unit);
        }
        let reference = self.reference()?;
        reference.read()?; // This operation replaces initialized scalar storage only.
        write(
            reference
                .slot
                .borrow_mut()
                .as_mut()
                .ok_or(TrapKind::InvalidIr)?,
            &reference.projections,
            value,
        )?;
        Ok(Value::Unit)
    }
}
