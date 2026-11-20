// This module is included as Rust source in native applications that import
// Clust's standard-library types. All storage belongs to the Rust values.
#[allow(dead_code, unused_imports)]
mod __clust_standard {
    pub mod option {
        pub use std::option::Option;
        pub fn is_some<T>(value: &Option<T>) -> bool {
            value.is_some()
        }
        pub fn unwrap_or<T>(value: Option<T>, fallback: T) -> T {
            value.unwrap_or(fallback)
        }
    }
    pub mod result {
        pub use std::result::Result;
        pub fn is_ok<T, E>(value: &Result<T, E>) -> bool {
            value.is_ok()
        }
    }
    pub mod vec {
        use std::vec::Vec as Native;
        #[repr(transparent)]
        pub struct Vec<T>(Native<T>);
        impl<T> Vec<T> {
            pub fn new() -> Self {
                Self(Native::with_capacity(4))
            }
            pub fn reserve(&mut self, capacity: usize) {
                if capacity > self.0.capacity() {
                    self.0.reserve(capacity - self.0.len());
                }
            }
            pub fn at(&self, i: usize) -> T
            where
                T: Copy,
            {
                self.0[i]
            }
            pub fn at_mut(&mut self, i: usize) -> &mut T {
                &mut self.0[i]
            }
            pub fn set(&mut self, i: usize, v: T) {
                self.0[i] = v;
            }
            pub fn into_native(self) -> Native<T> {
                self.0
            }
            pub fn from_native(v: Native<T>) -> Self {
                Self(v)
            }
        }
        impl<T> std::ops::Deref for Vec<T> {
            type Target = Native<T>;
            fn deref(&self) -> &Native<T> {
                &self.0
            }
        }
        impl<T> std::ops::DerefMut for Vec<T> {
            fn deref_mut(&mut self) -> &mut Native<T> {
                &mut self.0
            }
        }
    }
    pub mod string {
        use std::string::String as Native;
        #[repr(transparent)]
        pub struct String(Native);
        impl String {
            pub fn new(text: &str) -> Self {
                Self(text.to_owned())
            }
            pub fn append(&mut self, text: &str) {
                self.0.push_str(text);
            }
            pub fn append_string(&mut self, other: &String) {
                self.append(&other.0);
            }
            pub fn char_count(&self) -> usize {
                self.0.chars().count()
            }
            pub fn equals(&self, other: &String) -> bool {
                self.0 == other.0
            }
            pub fn byte_at(&self, i: usize) -> u8 {
                self.0.as_bytes()[i]
            }
            pub fn at(&self, i: usize) -> u8 {
                self.byte_at(i)
            }
            pub fn write(&self) {
                print!("{}", self.0);
            }
            pub fn into_native(self) -> Native {
                self.0
            }
            pub fn from_native(s: Native) -> Self {
                Self(s)
            }
        }
        impl std::ops::Deref for String {
            type Target = Native;
            fn deref(&self) -> &Native {
                &self.0
            }
        }
        impl std::ops::DerefMut for String {
            fn deref_mut(&mut self) -> &mut Native {
                &mut self.0
            }
        }
        impl std::fmt::Display for String {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    }
}
