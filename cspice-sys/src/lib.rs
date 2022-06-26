#![allow(
    clippy::missing_safety_doc,
    clippy::unreadable_literal,
    clippy::upper_case_acronyms,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    overflowing_literals,
    unused_imports
)]

include!(concat!(env!("OUT_DIR"), "/bindgen.rs"));

#[cfg(test)]
mod tests {
    use crate::*;
    use std::ffi::CString;

    /// Very basic test that we have successfully linked CSPICE and are able to call functions
    #[test]
    fn test() {
        unsafe {
            let set = CString::new("SET").unwrap().into_raw();
            let report = CString::new("REPORT").unwrap().into_raw();
            erract_c(set, 0, report);
            let str_ptr = CString::new("2027-MAR-23 16:00:00").unwrap().into_raw();
            let mut double = 0f64;
            str2et_c(str_ptr, &mut double);
            drop(CString::from_raw(set));
            drop(CString::from_raw(report));
            drop(CString::from_raw(str_ptr));
        }
    }
}
