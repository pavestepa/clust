use core.storage.{OwnedBuffer, BufferResult};
use core.option.{Option};

// One owner for all elements. Growth transfers values without cloning them.
pub class Vec<T> {
    buffer: OwnedBuffer<T>

    new() {
        self.buffer = OwnedBuffer<T>.make(4);
    }

    pub len(self) -> usize {
        return self.buffer.len();
    }

    pub capacity(self) -> usize {
        return self.buffer.capacity();
    }

    pub reserve(var self, capacity: usize) {
        self.buffer.reserve(capacity);
    }

    pub push(var self, value: T) {
        if self.buffer.len() == self.buffer.capacity() {
            let capacity = self.buffer.capacity() * 2;
            self.reserve(capacity);
        }
        match self.buffer.push(value) {
            BufferResult.None => {},
            BufferResult.Some(rejected) => { assert(false); },
        };
    }

    pub pop(var self) -> Option<T> {
        return match self.buffer.pop() {
            BufferResult.Some(value) => Option.Some(value),
            BufferResult.None => Option.None,
        };
    }

    pub at(self, index: usize) -> T {
        return self.buffer.at(index);
    }

    pub at_mut(var self, index: usize) -> T {
        return self.buffer.at_mut(index);
    }

    pub set(var self, index: usize, value: T) {
        self.buffer.set(index, value);
    }

    pub clear(var self) {
        self.buffer.clear();
    }
}
