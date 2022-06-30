use crate::string::SpiceString;
use crate::{Error, SPICE};
use cspice_sys::furnsh_c;

impl SPICE {
    /// Load one or more SPICE kernels into a program.
    ///
    /// https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/furnsh_c.html
    pub fn furnish(&self, file: &SpiceString) -> Result<(), Error> {
        unsafe {
            furnsh_c(file.as_mut_ptr());
        }
        self.get_last_error()
    }
}

#[cfg(test)]
mod tests {
    use crate::string::SpiceString;
    use crate::SPICE;

    #[test]
    fn test_furnish() {
        let spice = SPICE::get_instance();
        let error = spice
            .furnish(&SpiceString::from("NON_EXISTENT_FILE"))
            .err()
            .unwrap();
        assert_eq!(error.short_message, "SPICE(NOSUCHFILE)");
    }
}
