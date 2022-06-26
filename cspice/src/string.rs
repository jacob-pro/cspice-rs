use std::ffi::CStr;
use std::iter::once;
use std::os::raw::c_char;

pub trait ToSpiceString {
    fn to_spice_string(&self) -> Vec<c_char>;
}

impl<T: AsRef<str>> ToSpiceString for T {
    fn to_spice_string(&self) -> Vec<c_char> {
        self.as_ref()
            .bytes()
            .map(|b| b as c_char)
            .chain(once(0))
            .collect()
    }
}

pub trait FromSpiceString {
    fn from_spice_string(bytes: &[c_char]) -> Self;
}

impl FromSpiceString for String {
    fn from_spice_string(bytes: &[c_char]) -> Self {
        unsafe {
            CStr::from_ptr(bytes.as_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }
}
