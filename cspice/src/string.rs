use cspice_sys::SpiceChar;
use std::borrow::Cow;
use std::ffi::CStr;
use std::fmt::{Debug, Display, Formatter};
use std::iter::once;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct SpiceString(Vec<SpiceChar>);

impl Debug for SpiceString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.as_str(), f)
    }
}

impl Display for SpiceString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.as_str(), f)
    }
}

impl<T: AsRef<str>> From<T> for SpiceString {
    fn from(s: T) -> Self {
        Self(
            s.as_ref()
                .bytes()
                .map(|b| b as SpiceChar)
                .chain(once(0))
                .collect(),
        )
    }
}

impl SpiceString {
    /// Get the pointer to the SpiceString's data
    pub fn as_mut_ptr(&mut self) -> *mut SpiceChar {
        self.0.as_mut_ptr()
    }

    /// Convert a buffer of SpiceChar into a SpiceString
    ///
    /// This will panic if the buffer is not nul terminated
    pub fn from_buffer(mut s: Vec<SpiceChar>) -> Self {
        let end = s
            .iter()
            .position(|&x| x == 0)
            .expect("missing nul terminator");
        s.resize(end + 1, 0);
        Self(s)
    }

    pub fn as_str(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(self.0.as_ptr()).to_string_lossy() }
    }
}

impl From<SpiceString> for String {
    fn from(s: SpiceString) -> Self {
        s.as_str().into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_buffer() {
        let buffer = vec!['a' as SpiceChar, 'b' as SpiceChar, 0, 0, 0];
        let spice_str = SpiceString::from_buffer(buffer);
        assert_eq!(spice_str.0.len(), 3);
        assert_eq!(*spice_str.0.last().unwrap(), 0);
        assert_eq!(spice_str.as_str(), "ab");
    }

    #[test]
    #[should_panic]
    fn test_from_bad_buffer() {
        let buffer = vec!['a' as SpiceChar, 'b' as SpiceChar];
        SpiceString::from_buffer(buffer);
    }
}
