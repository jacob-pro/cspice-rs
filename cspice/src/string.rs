//! Functions for converting between Rust strings and SPICE (C) strings.
use cspice_sys::SpiceChar;
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

/// An owned nul terminated C string that can be used as input to SPICE functions.
///
/// A SpiceString can be created from a Rust &str type using [SpiceString::from].
///
/// An existing dynamically sized buffer can be converted in-place into a SpiceString using
/// [SpiceString::from_buffer].
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct SpiceString(pub CString);

impl Debug for SpiceString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpiceString({})", self.as_str())
    }
}

impl Display for SpiceString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.as_str())
    }
}

/// A SpiceString can be created from a Rust string.
impl<T: AsRef<str>> From<T> for SpiceString {
    #[inline]
    fn from(s: T) -> Self {
        Self(CString::new(s.as_ref()).unwrap())
    }
}

impl SpiceString {
    /// Get the pointer to the SpiceString's data. Intended for use passing string input to SPICE.
    ///
    /// # Safety
    ///
    /// This is a mut pointer for compatibility with the SPICE APIs, however it must not actually
    /// be mutated.
    #[inline]
    pub unsafe fn as_mut_ptr(&self) -> *mut SpiceChar {
        self.0.as_ptr() as *mut SpiceChar
    }

    /// Convert a buffer of SpiceChar into a SpiceString.
    ///
    /// This will panic if the buffer is not nul terminated.
    #[inline]
    pub fn from_buffer(mut s: Vec<SpiceChar>) -> Self {
        // Truncate from nul terminator
        let nul_pos = s
            .iter()
            .position(|&x| x == 0)
            .expect("missing nul terminator");
        s.resize(nul_pos, 0);

        // Convert from Vec<i8> to Vec<u8>
        // https://doc.rust-lang.org/src/alloc/vec/mod.rs.html#725
        // https://stackoverflow.com/a/59707887/7547647
        let mut s = std::mem::ManuallyDrop::new(s);
        let (ptr, len, cap) = (s.as_mut_ptr(), s.len(), s.capacity());

        unsafe {
            let s = Vec::from_raw_parts(ptr as *mut u8, len, cap);
            Self(CString::from_vec_unchecked(s))
        }
    }

    #[inline]
    pub fn as_str(&self) -> Cow<'_, str> {
        self.0.to_string_lossy()
    }
}

/// A reference to a nul-terminated C string.
///
/// A SpiceStr can be created from a reference to a byte buffer using [SpiceStr::from_buffer].
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct SpiceStr<'a>(pub &'a CStr);

impl SpiceStr<'_> {
    /// Get a SpiceStr (CStr) from a buffer. Intended for reading a buffer containing a string
    /// output from SPICE.
    ///
    /// # Panics
    ///
    /// Panics if the buffer is not nul terminated.
    #[inline]
    pub fn from_buffer(buffer: &[SpiceChar]) -> Self {
        // https://doc.rust-lang.org/src/std/ffi/c_str.rs.html#1295-1306
        let nul_pos = buffer
            .iter()
            .position(|&x| x == 0)
            .expect("missing nul terminator");
        let subslice = &buffer[..nul_pos + 1];
        unsafe {
            let u8slice = &*(subslice as *const [std::os::raw::c_char] as *const [u8]);
            Self(CStr::from_bytes_with_nul_unchecked(u8slice))
        }
    }

    #[inline]
    pub fn as_str(&self) -> Cow<'_, str> {
        self.0.to_string_lossy()
    }
}

impl Debug for SpiceStr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpiceStr({})", self.as_str())
    }
}

impl Display for SpiceStr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.as_str())
    }
}

/// Internal static C strings used when calling SPICE APIs.
///
/// Should be created using the [static_spice_str!] macro to ensure nul termination.
#[derive(Copy, Clone)]
pub(crate) struct StaticSpiceStr(pub &'static str);

impl StaticSpiceStr {
    pub(crate) unsafe fn as_mut_ptr(&self) -> *mut SpiceChar {
        self.0.as_ptr() as *mut SpiceChar
    }
}

macro_rules! static_spice_str {
    ($input:literal) => {
        StaticSpiceStr(concat!($input, "\0"))
    };
}
pub(crate) use static_spice_str;

/// Allows you to pass a Rust string that will automatically be converted into a nul terminated C
/// string. Alternatively you can pass an existing &SpiceString as an argument so that the string
/// does not need to be converted on each call.
pub enum StringParam<'a> {
    Ref(&'a SpiceString),
    Owned(SpiceString),
}

impl<S: AsRef<str>> From<S> for StringParam<'_> {
    fn from(s: S) -> Self {
        StringParam::Owned(SpiceString::from(s))
    }
}

impl<'a> From<&'a SpiceString> for StringParam<'a> {
    fn from(s: &'a SpiceString) -> Self {
        StringParam::Ref(s)
    }
}

impl From<SpiceString> for StringParam<'_> {
    fn from(s: SpiceString) -> Self {
        StringParam::Owned(s)
    }
}

impl Deref for StringParam<'_> {
    type Target = SpiceString;

    fn deref(&self) -> &Self::Target {
        match &self {
            StringParam::Ref(r) => r,
            StringParam::Owned(o) => o,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_buffer() {
        let buffer = vec!['a' as SpiceChar, 'b' as SpiceChar, 0, 0, 0];
        let spice_str = SpiceString::from_buffer(buffer);
        assert_eq!(spice_str.as_str(), "ab");
    }

    #[test]
    fn test_from_bad_buffer() {
        std::panic::catch_unwind(|| {
            let buffer = vec!['a' as SpiceChar, 'b' as SpiceChar];
            SpiceString::from_buffer(buffer);
        })
        .err()
        .expect("Expected to panic");
    }
}
