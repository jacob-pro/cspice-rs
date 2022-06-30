use cspice_sys::SpiceChar;
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct SpiceString(pub CString);

impl Debug for SpiceString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpiceString({})", self.as_str())
    }
}

impl Display for SpiceString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&*self.as_str())
    }
}

impl<T: Into<String>> From<T> for SpiceString {
    #[inline]
    fn from(s: T) -> Self {
        Self(CString::new(s.into()).unwrap())
    }
}

impl From<SpiceStr<'_>> for SpiceString {
    fn from(s: SpiceStr<'_>) -> Self {
        SpiceString(CString::from(s.0))
    }
}

impl SpiceString {
    /// Get the pointer to the SpiceString's data
    ///
    /// # Safety
    ///
    /// This is a mut pointer for compatibility with the SPICE APIs, however it must not actually
    /// be mutated.
    #[inline]
    pub unsafe fn as_mut_ptr(&self) -> *mut SpiceChar {
        self.0.as_ptr() as *mut SpiceChar
    }

    /// Convert a buffer of SpiceChar into a SpiceString
    ///
    /// This will panic if the buffer is not nul terminated
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

impl Deref for SpiceString {
    type Target = CString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct SpiceStr<'a>(&'a CStr);

impl SpiceStr<'_> {
    /// Get a SpiceStr (CStr) from a buffer
    ///
    /// # Panics
    ///
    /// Panics if the buffer is not nul terminated
    #[inline]
    pub fn from_buffer(buffer: &[SpiceChar]) -> Self {
        // https://doc.rust-lang.org/src/std/ffi/c_str.rs.html#1295-1306
        let nul_pos = buffer
            .iter()
            .position(|&x| x == 0)
            .expect("missing nul terminator");
        let subslice = &buffer[..nul_pos + 1];
        unsafe {
            let u8slice = &*(subslice as *const [i8] as *const [u8]);
            Self(CStr::from_bytes_with_nul_unchecked(u8slice))
        }
    }

    #[inline]
    pub fn as_str(&self) -> Cow<'_, str> {
        self.0.to_string_lossy()
    }
}

impl Deref for SpiceStr<'_> {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Debug for SpiceStr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpiceStr({})", self.as_str())
    }
}

impl Display for SpiceStr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&*self.as_str())
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
