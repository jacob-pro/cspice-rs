use crate::string::StringParam;
use crate::{spice_unsafe, Error, Spice};
use cspice_sys::{furnsh_c, unload_c};

/// Functions for loading and unloading SPICE Kernels.
impl Spice {
    /// Load one or more SPICE kernels into a program.
    ///
    /// See [furnsh_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/furnsh_c.html).
    pub fn furnish<'f, F: Into<StringParam<'f>>>(file: F) -> Result<(), Error> {
        spice_unsafe!({
            furnsh_c(file.into().as_mut_ptr());
        });
        Spice::get_last_error()
    }

    /// Unload a SPICE kernel.
    ///
    /// See [unload_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/unload_c.html).
    pub fn unload<'f, F: Into<StringParam<'f>>>(file: F) -> Result<(), Error> {
        spice_unsafe!({
            unload_c(file.into().as_mut_ptr());
        });
        Spice::get_last_error()
    }
}

#[cfg(test)]
mod tests {
    use crate::Spice;

    #[test]
    fn test_furnish() {
        let error = Spice::furnish("NON_EXISTENT_FILE").err().unwrap();
        assert_eq!(error.short_message, "SPICE(NOSUCHFILE)");
    }
}
