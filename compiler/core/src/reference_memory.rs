//! Host-side cells used only by the bounded reference interpreters. Clearing a
//! slot ends its language lifetime even when host handles to that cell remain.
//! Native views use raw addresses and never allocate or retain these cells.
use crate::{
    hir::{Type, Value},
    interpret::TrapKind,
    mir::Projection,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
pub(crate) type Clock = Rc<Cell<u64>>;
pub(crate) struct Storage {
    value: RefCell<Option<Value>>,
    raw_origin: RefCell<Option<RawPointer>>,
    pub state: RefCell<crate::mir::drop_state::State>,
    pub initialized_at: Cell<u64>,
    finalizing: Cell<bool>,
    clock: Clock,
    ty: Type,
}
#[derive(Clone)]
pub(crate) struct Slot(Rc<Storage>);
impl std::ops::Deref for Slot {
    type Target = RefCell<Option<Value>>;
    fn deref(&self) -> &Self::Target {
        &self.0.value
    }
}
impl Slot {
    pub fn ty(&self) -> &Type {
        &self.0.ty
    }
    pub fn state(&self) -> &RefCell<crate::mir::drop_state::State> {
        &self.0.state
    }
    pub fn initialized_at(&self) -> u64 {
        self.0.initialized_at.get()
    }
    pub fn initialize(&self) -> Result<(), TrapKind> {
        self.0
            .state
            .borrow_mut()
            .initialize(&[])
            .map_err(|_| TrapKind::InvalidIr)?;
        let stamp = self
            .0
            .clock
            .get()
            .checked_add(1)
            .ok_or(TrapKind::StepLimit)?;
        self.0.clock.set(stamp);
        self.0.initialized_at.set(stamp);
        Ok(())
    }
}
pub(crate) fn typed_slot(ty: Type, clock: Clock) -> Slot {
    Slot(Rc::new(Storage {
        value: RefCell::new(None),
        raw_origin: RefCell::new(None),
        state: RefCell::new(Default::default()),
        initialized_at: Cell::new(0),
        finalizing: Cell::new(false),
        clock,
        ty,
    }))
}
#[derive(Clone)]
pub(crate) struct Reference {
    pub slot: Slot,
    pub projections: Vec<Projection>,
}
/// Returned descriptor used only by the reference interpreters.
#[derive(Clone)]
pub struct ViewValue(pub(crate) Reference);
impl std::fmt::Debug for ViewValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("View").field(&self.0.read()).finish()
    }
}
impl PartialEq for ViewValue {
    fn eq(&self, other: &Self) -> bool {
        self.0.read() == other.0.read()
    }
}
/// A handle used only by reference interpreters; native code owns a Box.
#[derive(Clone)]
pub struct ClassValue {
    pub(crate) payload: Slot,
}
impl ClassValue {
    pub(crate) fn same_identity(&self, other: &Self) -> bool {
        Rc::ptr_eq(
            &self.actual_identity().payload.0,
            &other.actual_identity().payload.0,
        )
    }
    pub(crate) fn actual_identity(&self) -> ClassValue {
        fn object(value: &ClassValue) -> ClassValue {
            if matches!(value.payload.ty(),Type::Record {name,..} if name.starts_with("$poly$")) {
                let payload = value.payload.borrow().clone();
                if let Some(Value::Aggregate(fields)) = payload {
                    if let Some(Value::Enum { fields, .. }) = fields.first() {
                        let field = fields.first().cloned();
                        let field = match field {
                            Some(Value::View(view)) => view.0.read().ok(),
                            value => value,
                        };
                        if let Some(Value::Class(child)) = field {
                            return object(&child);
                        }
                    }
                }
            }
            value.clone()
        }
        object(self)
    }
}
impl std::fmt::Debug for ClassValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Class")
            .field(&*self.payload.borrow())
            .finish()
    }
}
impl PartialEq for ClassValue {
    fn eq(&self, other: &Self) -> bool {
        self.payload.ty() == other.payload.ty() && *self.payload.borrow() == *other.payload.borrow()
    }
}
#[derive(Clone)]
pub(crate) enum Argument {
    Owned(Value),
    Borrowed(Reference),
}

/// Non-owning interpreter address. Weak storage cannot keep a returned local alive.
#[derive(Clone)]
pub struct RawPointer {
    // Keep Value compact: recursive diagnostic execution places values on the
    // host stack. Pointer copies share immutable type descriptions.
    address_only: Option<u64>,
    storage_type: Rc<Type>,
    view_type: Rc<Type>,
    heap: Option<std::rc::Weak<RawHeap>>,
    storage: std::rc::Weak<Storage>,
    prefix: Rc<Vec<Projection>>,
    region_offset: u64,
    allocation_extent: u64,
    index: i128,
    byte_in_element: i128,
    length: i128,
    array: bool,
}
impl std::fmt::Debug for RawPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawPointer")
            .field("index", &self.index)
            .field("byte_in_element", &self.byte_in_element)
            .field("length", &self.length)
            .finish_non_exhaustive()
    }
}
impl PartialEq for RawPointer {
    fn eq(&self, other: &Self) -> bool {
        if self.address_only.is_some() || other.address_only.is_some() {
            return self.address_only == other.address_only;
        }
        match (&self.heap, &other.heap) {
            (Some(a), Some(b)) => {
                return a.ptr_eq(b)
                    && self.index == other.index
                    && self.byte_in_element == other.byte_in_element
            }
            (None, None) => (),
            _ => return false,
        }
        self.storage.ptr_eq(&other.storage)
            && self.prefix == other.prefix
            && self.index == other.index
            && self.byte_in_element == other.byte_in_element
    }
}

struct RawHeap {
    owned: Cell<bool>,
    capacity: u64,
    slots: RefCell<std::collections::BTreeMap<u64, Slot>>,
    clock: Clock,
}
struct ExposedAddress {
    base: u64,
    extent: u64,
    pointer: RawPointer,
    writable: bool,
}
pub(crate) struct RawAllocations {
    heaps: Vec<Rc<RawHeap>>,
    exposed: Vec<ExposedAddress>,
    next_address: u64,
}
impl Default for RawAllocations {
    fn default() -> Self {
        Self {
            heaps: vec![],
            exposed: vec![],
            next_address: 4096,
        }
    }
}

/// Weak registry retains identity cells, never a target payload owner.
#[derive(Default)]
pub(crate) struct WeakRegistry {
    next: usize,
    entries: std::collections::BTreeMap<usize, (std::rc::Weak<Storage>, String)>,
}

mod addresses;
mod allocations;
mod bit_patterns;
mod class_storage;
mod offsets;
mod pointer_access;
mod pointer_math;
mod references;
mod storage_operations;
mod weak_registry;
use self::bit_patterns::bits_bytes;
use self::bit_patterns::bits_value;
pub(crate) use self::class_storage::allocate_class;
pub(crate) use self::class_storage::class_reference;
pub(crate) use self::class_storage::complete_class;
pub(crate) use self::class_storage::copy_value;
use self::offsets::projection_byte_offset;
use self::references::write;
