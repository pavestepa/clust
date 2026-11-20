use core.vec.{Vec};

// An owned UTF-8 string. Only validated str and Unicode scalar inputs can
// extend its private byte buffer; read access never grants mutable bytes.
pub class String {
    bytes: Vec<u8>

    new(text: str) {
        self.bytes = Vec<u8>.new();
        var index: usize = 0;
        while index < str_byte_len(text) {
            self.bytes.push(str_byte_at(text, index));
            index += 1;
        }
    }

    pub len(self) -> usize { return self.bytes.len(); }
    pub is_empty(self) -> bool { return self.bytes.len() == 0; }
    pub byte_at(self, index: usize) -> u8 { return self.bytes.at(index); }
    pub at(self, index: usize) -> u8 { return self.bytes.at(index); }
    pub clear(var self) { self.bytes.clear(); }

    pub append(var self, text: str) {
        var index: usize = 0;
        while index < str_byte_len(text) {
            self.bytes.push(str_byte_at(text, index));
            index += 1;
        }
    }

    pub append_string(var self, text: String) {
        var index: usize = 0;
        while index < text.len() {
            self.bytes.push(text.byte_at(index));
            index += 1;
        }
    }

    pub push(var self, value: char) {
        let code = value as u32;
        // Reserve the complete scalar before modifying the buffer.
        let capacity = self.bytes.len() + 4;
        if capacity > self.bytes.capacity() {
            let grown = self.bytes.capacity() * 2;
            self.bytes.reserve(if grown > capacity { grown } else { capacity });
        }
        if code < 128 {
            self.bytes.push(code as u8);
        } else if code < 2048 {
            self.bytes.push(((code >> 6) | 192) as u8);
            self.bytes.push(((code & 63) | 128) as u8);
        } else if code < 65536 {
            self.bytes.push(((code >> 12) | 224) as u8);
            self.bytes.push((((code >> 6) & 63) | 128) as u8);
            self.bytes.push(((code & 63) | 128) as u8);
        } else {
            self.bytes.push(((code >> 18) | 240) as u8);
            self.bytes.push((((code >> 12) & 63) | 128) as u8);
            self.bytes.push((((code >> 6) & 63) | 128) as u8);
            self.bytes.push(((code & 63) | 128) as u8);
        }
    }

    pub char_count(self) -> usize {
        var count: usize = 0;
        var index: usize = 0;
        while index < self.bytes.len() {
            if (self.bytes.at(index) & 192) != 128 { count += 1; }
            index += 1;
        }
        return count;
    }

    pub equals(self, other: String) -> bool {
        if self.len() != other.len() { return false; }
        var index: usize = 0;
        while index < self.len() {
            if self.byte_at(index) != other.byte_at(index) { return false; }
            index += 1;
        }
        return true;
    }

    pub write(self) {
        var index: usize = 0;
        while index < self.bytes.len() {
            write_byte(self.bytes.at(index));
            index += 1;
        }
    }
}
