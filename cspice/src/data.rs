//! Functions for loading and unloading SPICE Kernels.
use crate::error::get_last_error;
use crate::string::StringParam;
use crate::{spice_unsafe, Error};
use cspice_sys::{furnsh_c, unload_c};

/// Load one or more SPICE kernels into a program.
///
/// See [furnsh_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/furnsh_c.html).
pub fn furnish<'f, F: Into<StringParam<'f>>>(file: F) -> Result<(), Error> {
    spice_unsafe!({
        furnsh_c(file.into().as_mut_ptr());
    });
    get_last_error()
}

/// Unload a SPICE kernel.
///
/// See [unload_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/unload_c.html).
pub fn unload<'f, F: Into<StringParam<'f>>>(file: F) -> Result<(), Error> {
    spice_unsafe!({
        unload_c(file.into().as_mut_ptr());
    });
    get_last_error()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_furnish() {
        let error = furnish("NON_EXISTENT_FILE").err().unwrap();
        assert_eq!(error.short_message, "SPICE(NOSUCHFILE)");
    }
}
