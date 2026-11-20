use core.intrinsic.{RawStorage,allocate,capacity,initialize,take,drop_range,checked_index,view,view_mut,replace,move_slots};

// Owns the initialized prefix. Push returns the supplied value when full.
// The compiler lowers the private slot operations for this bundled module.
pub enum BufferResult<T> {None,Some(T)}
pub class OwnedBuffer<T> {
    new(storage: RawStorage<T>, length: usize) { self.storage = storage; self.length = length; }
    storage:RawStorage<T>
    length:usize
    pub make(count:usize) -> OwnedBuffer<T> {
        make_empty();
    }
    pub reserve(var self,count:usize) -> () {
        if count>self.storage.capacity() {
            grow_initialized();
        }
    }
    pub len(self) -> usize {return self.length;}
    pub capacity(self) -> usize {return self.storage.capacity();}
    pub push(var self,value:T) -> BufferResult<T> {
        if self.length==self.storage.capacity() {return BufferResult<T>.Some(value);}
        push_slot();
        self.length+=1;
        return BufferResult<T>.None;
    }
    pub pop(var self) -> BufferResult<T> {
        if self.length==0 {return BufferResult<T>.None;}
        self.length-=1;
        pop_slot();
    }
    // Copy values materialize unless the caller uses an explicit ref binding.
    // Resource class results borrow their identity from this buffer.
    pub at(self,index:usize) -> T {
        let valid=checked_index(index,self.length);
        read_slot();
    }
    pub at_mut(var self,index:usize) -> T {
        let valid=checked_index(index,self.length);
        write_view();
    }
    pub set(var self,index:usize,value:T) -> T {
        let valid=checked_index(index,self.length);
        replace_slot();
    }
    pub clear(var self) -> () {
        let count=self.length;
        self.length=0;
        destroy_prefix();
    }
    drop() {
        let count=self.length;
        self.length=0;
        destroy_prefix();
    }
}
