use crate::string::StringParam;
use crate::{Error, Spice};
use cspice_sys::{furnsh_c, unload_c};

/// Functions for loading and unloading SPICE Kernels.
impl Spice {
    /// Load one or more SPICE kernels into a program.
    ///
    /// See [furnsh_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/furnsh_c.html).
    pub fn furnish<'f, F: Into<StringParam<'f>>>(&self, file: F) -> Result<(), Error> {
        unsafe {
            furnsh_c(file.into().as_mut_ptr());
        }
        self.get_last_error()
    }

    /// Unload a SPICE kernel.
    ///
    /// See [unload_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/unload_c.html).
    pub fn unload<'f, F: Into<StringParam<'f>>>(&self, file: F) -> Result<(), Error> {
        unsafe {
            unload_c(file.into().as_mut_ptr());
        }
        self.get_last_error()
    }
}

#[cfg(test)]
mod tests {
    use crate::Spice;

    #[test]
    fn test_furnish() {
        let spice = Spice::get_instance();
        let error = spice.furnish("NON_EXISTENT_FILE").err().unwrap();
        assert_eq!(error.short_message, "SPICE(NOSUCHFILE)");
    }
}
