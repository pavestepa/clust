use super::*;

impl RawPointer {
    pub(crate) fn new(reference: Reference) -> Result<Self, TrapKind> {
        let reference = reference.normalized()?;
        reference.read()?;
        if reference.projections.is_empty() {
            if let Some(pointer) = reference.slot.0.raw_origin.borrow().as_ref() {
                return Ok(pointer.clone());
            }
        }
        let mut ty = reference.slot.ty().clone();
        let mut array = None;
        for (depth, projection) in reference.projections.iter().enumerate() {
            ty = match (projection, ty) {
                (Projection::Field(i), Type::Array(element, count)) => {
                    if depth + 1 == reference.projections.len() {
                        array = Some((*i as i128, count as i128));
                    }
                    *element
                }
                (Projection::Field(i), Type::Record { fields, .. }) => fields[*i].ty.clone(),
                (Projection::Field(i), Type::Tuple(fields)) => fields[*i].clone(),
                (Projection::Variant(i), Type::Enum { variants, .. }) => {
                    Type::Tuple(variants[*i].fields.clone())
                }
                _ => return Err(TrapKind::InvalidIr),
            };
        }
        let mut prefix = reference.projections;
        if array.is_some() {
            prefix.pop();
        }
        let (index, length) = array.unwrap_or((0, 1));
        let region_offset = projection_byte_offset(reference.slot.ty(), &prefix).unwrap_or(0);
        let allocation_extent = crate::layout::layout(reference.slot.ty()).map_or(0, |l| l.size());
        Ok(Self {
            address_only: None,
            storage_type: Rc::new(ty.clone()),
            view_type: Rc::new(ty),
            heap: None,
            storage: std::rc::Rc::downgrade(&reference.slot.0),
            prefix: Rc::new(prefix),
            region_offset,
            allocation_extent,
            index,
            byte_in_element: 0,
            length,
            array: array.is_some(),
        })
    }
}

impl RawPointer {
    pub(crate) fn reinterpret(mut self, target: &Type) -> Option<Self> {
        if *self.view_type != *target && !self.view_type.raw_reinterpret_to(target) {
            return None;
        }
        self.view_type = Rc::new(target.clone());
        Some(self)
    }
}

impl RawPointer {
    pub(super) fn storage_stride(&self) -> Result<i128, TrapKind> {
        crate::layout::layout(&self.storage_type)
            .map(|v| i128::from(v.size()))
            .map_err(|_| TrapKind::InvalidIr)
    }
}

impl RawPointer {
    pub(super) fn byte_position(&self) -> Result<i128, TrapKind> {
        self.index
            .checked_mul(self.storage_stride()?)
            .and_then(|n| n.checked_add(self.byte_in_element))
            .ok_or(TrapKind::Bounds)
    }
}

impl RawPointer {
    pub(crate) fn raw_offset(self, amount: i128) -> Result<Self, TrapKind> {
        if amount != 0 && self.storage_stride()? == 0 {
            return Err(TrapKind::Bounds);
        }
        self.offset(amount)
    }
}

impl RawPointer {
    pub(crate) fn offset(mut self, amount: i128) -> Result<Self, TrapKind> {
        if self.address_only.is_some() {
            return if amount == 0 {
                Ok(self)
            } else {
                Err(TrapKind::Bounds)
            };
        }
        let stride = self.storage_stride()?;
        if stride == 0 {
            self.index = self.index.checked_add(amount).ok_or(TrapKind::Bounds)?;
        } else {
            let view = crate::layout::layout(&self.view_type).map_err(|_| TrapKind::InvalidIr)?;
            let position = amount
                .checked_mul(i128::from(view.size()))
                .and_then(|n| self.byte_position().ok()?.checked_add(n))
                .ok_or(TrapKind::Bounds)?;
            if position < 0 || position > self.length * stride {
                return Err(TrapKind::Bounds);
            }
            self.index = position / stride;
            self.byte_in_element = position % stride;
        }
        if self.index < 0 || self.index > self.length {
            return Err(TrapKind::Bounds);
        }
        Ok(self)
    }
}

impl RawPointer {
    pub(super) fn same_allocation(&self, other: &Self) -> bool {
        if self.address_only.is_some() || other.address_only.is_some() {
            return false;
        }
        match (&self.heap, &other.heap) {
            (Some(a), Some(b)) => a.ptr_eq(b),
            (None, None) => self.storage.ptr_eq(&other.storage),
            _ => false,
        }
    }
}

impl RawPointer {
    pub(super) fn same_region(&self, other: &Self) -> bool {
        if self.address_only.is_some() || other.address_only.is_some() {
            return false;
        }
        match (&self.heap, &other.heap) {
            (Some(a), Some(b)) => a.ptr_eq(b),
            (None, None) => {
                self.storage.ptr_eq(&other.storage)
                    && self.prefix == other.prefix
                    && self.array == other.array
            }
            _ => false,
        }
    }
}

impl RawPointer {
    pub(crate) fn difference(&self, other: &Self) -> Result<i128, TrapKind> {
        let same = match (&self.heap, &other.heap) {
            (Some(a), Some(b)) => a.ptr_eq(b) && a.upgrade().is_some(),
            (None, None) => {
                self.storage.ptr_eq(&other.storage)
                    && self.storage.upgrade().is_some()
                    && self.prefix == other.prefix
                    && self.array == other.array
            }
            _ => false,
        };
        if !same
            || self.length != other.length
            || self.index < 0
            || other.index < 0
            || self.index > self.length
            || other.index > other.length
        {
            return Err(TrapKind::Bounds);
        }
        let view = crate::layout::layout(&self.view_type).map_err(|_| TrapKind::InvalidIr)?;
        let size = i128::from(view.size());
        let bytes = self.byte_position()? - other.byte_position()?;
        if self.view_type != other.view_type || size == 0 || bytes % size != 0 {
            return Err(TrapKind::Bounds);
        }
        let distance = bytes / size;
        i64::try_from(distance)
            .map(|n| n as i128)
            .map_err(|_| TrapKind::Bounds)
    }
}

impl RawPointer {
    pub(super) fn reference(&self) -> Result<Reference, TrapKind> {
        if self.index < 0 || self.index >= self.length {
            return Err(TrapKind::Bounds);
        }
        let slot = Slot(self.storage.upgrade().ok_or(TrapKind::InvalidIr)?);
        let mut projections = (*self.prefix).clone();
        if self.array {
            projections.push(Projection::Field(self.index as usize));
        }
        Ok(Reference { slot, projections })
    }
}
