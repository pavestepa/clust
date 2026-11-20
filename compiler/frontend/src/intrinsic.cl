// Trusted owned storage, including nominal payloads with generated cleanup. No initialized-length policy lives here.
pub class RawStorage<T> {
    new(data: *var T, cap: usize) { self.data = data; self.cap = cap; }
    data:*var T
    cap:usize
    pub allocate(count:usize) -> RawStorage<T> {
        return RawStorage<T>.new(heap_allocate<T>(count), count);
    }
    pub from_raw_parts(pointer:*var T,count:usize) -> RawStorage<T> {
        heap_adopt<T>(pointer,count);return RawStorage<T>.new(pointer, count);
    }
    pub capacity(self) -> usize {return cap;}
    pub check_range(self,start:usize,count:usize) -> () {
        heap_check_range<T>(data,cap,start,count);
    }
    pub move_within(var self,destination:usize,source:usize,count:usize) -> () {
        heap_move_slots<T>(data,cap,destination,data,cap,source,count);
    }
    pub move_from(var self,destination:usize,var source:RawStorage<T>,start:usize,count:usize) -> () {
        heap_move_slots<T>(data,cap,destination,source.data,source.cap,start,count);
    }
    pub initialize(var self,index:usize,value:T) -> () {
        heap_initialize<T>(data,index,value);
    }
    pub view(self,index:usize) -> T {
        return heap_view<T>(self,index);
    }
    pub view_mut(var self,index:usize) -> T {
        return heap_view_mut<T>(self,index);
    }
    pub slot_address(self,index:usize) -> *T {
        return heap_address<T>(data,cap,index);
    }
    pub slot_address_mut(var self,index:usize) -> *var T {
        return heap_address_mut<T>(data,cap,index);
    }
    pub take(var self,index:usize) -> T {
        return heap_take<T>(data,index);
    }
    pub replace(var self,index:usize,value:T) -> T {
        return heap_replace<T>(data,index,value);
    }
    pub drop_range(var self,start:usize,count:usize) -> () {
        heap_drop_range<T>(data,cap,start,count);
    }
    pub drop_slot(var self,index:usize) -> () {
        heap_drop<T>(data,index);
    }
    drop() {heap_release<T>(data,cap);}
}
class Released<T> {
    pub storage: RawStorage<T>
    new(storage: RawStorage<T>) { self.storage = storage; }
}
pub fn allocate<T>(count:usize) -> RawStorage<T> {
    return RawStorage<T>.allocate(count);
}
pub fn capacity<T>(storage:RawStorage<T>) -> usize {return storage.capacity();}
pub fn size_of<T>() -> usize {return layout_size<T>();}
pub fn align_of<T>() -> usize {return layout_align<T>();}
pub fn slot_size<T>(storage:RawStorage<T>) -> usize {return layout_size<T>();}
pub fn slot_align<T>(storage:RawStorage<T>) -> usize {return layout_align<T>();}
pub fn initialize<T>(var storage:RawStorage<T>,index:usize,value:T) -> () {
    storage.initialize(index,value);
}
pub fn take<T>(var storage:RawStorage<T>,index:usize) -> T {
    return storage.take(index);
}
pub fn drop_slot<T>(var storage:RawStorage<T>,index:usize) -> () {
    storage.drop_slot(index);
}
pub fn release<T>(storage:RawStorage<T>) -> () {
    let guard=Released<T>.new(storage);
}

pub fn replace<T>(var storage:RawStorage<T>,index:usize,value:T) -> T {
    return storage.replace(index,value);
}

pub fn move_within<T>(var storage:RawStorage<T>,destination_start:usize,source_start:usize,count:usize) -> () {
    storage.move_within(destination_start,source_start,count);
}
pub fn move_slots<T>(var destination:RawStorage<T>,destination_start:usize,var source:RawStorage<T>,source_start:usize,count:usize) -> () {
    destination.move_from(destination_start,source,source_start,count);
}

// Safety: every slot in the range is initialized and owned by this storage.
// Cleanup continues after the first panic; a second panic aborts.
pub fn drop_range<T>(var storage:RawStorage<T>,start:usize,count:usize) -> () {
    storage.drop_range(start,count);
}

pub fn view<T>(storage:RawStorage<T>,index:usize) -> T {return storage.view(index);}
pub fn view_mut<T>(var storage:RawStorage<T>,index:usize) -> T {return storage.view_mut(index);}

pub fn checked_index(index:usize,length:usize) -> usize {return bounds_index<()>(index,length);}

// Safety: raw addresses do not retain storage or establish a safe loan.
// index may equal capacity (one-past); that address cannot be dereferenced.
pub fn slot_address<T>(storage:RawStorage<T>,index:usize) -> *T {
    return storage.slot_address(index);
}
pub fn slot_address_mut<T>(var storage:RawStorage<T>,index:usize) -> *var T {
    return storage.slot_address_mut(index);
}

// Safety: pointer addresses one initialized owned value, with exclusive access.
// The value becomes uninitialized before its destructor runs, including on panic.
pub fn drop_in_place<T>(pointer:*var T) -> () {
    pointer_drop<T>(pointer);
}

// Safety: caller assumes responsibility for dropping initialized slots and release.
pub fn into_raw_parts<T>(storage:RawStorage<T>) -> (*var T,usize) {
    return heap_detach<T>(storage);
}
// Safety: pointer/count are the exact allocation returned by into_raw_parts<T>,
// and no other owner exists. This transfers allocation ownership, not slot state.
pub fn from_raw_parts<T>(pointer:*var T,count:usize) -> RawStorage<T> {
    return RawStorage<T>.from_raw_parts(pointer,count);
}

// Safety: pointer identifies one fully initialized owned T, with exclusive
// access. Ownership transfers to the result; the source is now uninitialized.
// The caller must prevent subsequent source access and automatic double cleanup.
pub fn read_owned<T>(pointer:*var T) -> T {
    return pointer_take<T>(pointer);
}

// Safety: pointer identifies one uninitialized whole slot for T, with exclusive
// access. Consumes value without dropping any previous contents.
pub fn write_owned<T>(pointer:*var T,value:T) -> () {
    pointer_initialize<T>(pointer,0,value);
}

use core.option.{Option};
pub fn weak_create<T>(target:T) -> (usize,usize) {return weak_identity<T>(target);}
pub fn weak_exists<T>(token:(usize,usize)) -> bool {return weak_alive<T>(token);}
pub fn weak_read<T,R:Copy>(token:(usize,usize),callback:(T)=>R) -> Option<R> {return weak_access<T,R>(token,callback);}
