use super::*;
impl Type {
    /// Owned storage payloads with a concrete representation and no borrowed fields.
    pub fn raw_storage_value(&self) -> bool {
        match self {
            Self::Unit | Self::Pointer { .. } => true,
            Self::Class { .. } => !self.contains_views(),
            Self::Record { fields, .. } => fields.iter().all(|f| f.ty.raw_storage_value()),
            Self::Enum { variants, .. } => variants
                .iter()
                .flat_map(|v| &v.fields)
                .all(Self::raw_storage_value),
            Self::Tuple(fields) => fields.iter().all(Self::raw_storage_value),
            Self::Array(element, _) => element.raw_storage_value(),
            _ => self.raw_scalar(),
        }
    }
    /// Bitwise raw access is limited to values without ownership or borrowed descriptors.
    pub fn raw_copy_value(&self) -> bool {
        match self {
            Self::Unit | Self::Pointer { .. } => true,
            Self::Tuple(fields) => fields.iter().all(Self::raw_copy_value),
            Self::Array(element, _) => element.raw_copy_value(),
            Self::Record {
                copy: true, fields, ..
            } => fields.iter().all(|field| field.ty.raw_copy_value()),
            Self::Enum {
                copy: true,
                variants,
                ..
            } => variants
                .iter()
                .all(|variant| variant.fields.iter().all(Self::raw_copy_value)),
            _ => self.raw_scalar(),
        }
    }
    /// Bit reinterpretation changes the access type, never allocation provenance.
    /// Alignment, extent and initialization are unsafe access preconditions.
    pub fn raw_reinterpret_to(&self, target: &Self) -> bool {
        self.raw_bits_value() && target.raw_bits_value()
    }
    /// Every byte belongs to initialized payload, with no padding/provenance.
    /// bool/char/enum bit validity is an unsafe access precondition, diagnosed when
    /// the reference interpreters decode bytes into a typed value.
    pub fn raw_bits_value(&self) -> bool {
        if self.raw_scalar() {
            return true;
        }
        let packed = |fields: Vec<&Type>| {
            if fields.is_empty() || !fields.iter().all(|t| t.raw_bits_value()) {
                return false;
            }
            let size = fields.iter().try_fold(0u64, |size, t| {
                size.checked_add(crate::layout::layout(t).ok()?.size())
            });
            crate::layout::layout(self)
                .is_ok_and(|layout| Some(layout.size()) == size && layout.size() != 0)
        };
        match self {
            Self::Array(element, count) => {
                *count != 0 && element.raw_bits_value() && crate::layout::layout(self).is_ok()
            }
            Self::Enum {
                copy: true,
                variants,
                ..
            } => {
                // Every variant must initialize the entire union payload and
                // the explicit u32 tag must be adjacent to it (no padding).
                !variants.is_empty()
                    && crate::layout::layout(self).is_ok_and(|layout| {
                        variants.iter().all(|v| {
                            let payload = Self::Tuple(v.fields.clone());
                            (v.fields.is_empty() || payload.raw_bits_value())
                                && crate::layout::layout(&payload)
                                    .is_ok_and(|p| p.size().checked_add(4) == Some(layout.size()))
                        })
                    })
            }
            Self::Tuple(fields) => packed(fields.iter().collect()),
            Self::Record {
                copy: true, fields, ..
            } => packed(fields.iter().map(|f| &f.ty).collect()),
            _ => false,
        }
    }
    pub fn raw_scalar(&self) -> bool {
        matches!(
            self,
            Self::Bool | Self::Char | Self::Integer(_) | Self::F32 | Self::F64
        )
    }
    pub fn numeric(&self) -> bool {
        matches!(self, Self::Integer(_) | Self::F32 | Self::F64)
    }
}
