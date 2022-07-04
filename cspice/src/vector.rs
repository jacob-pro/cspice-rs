use crate::Spice;
use cspice_sys::{vsep_c, SpiceDouble};

pub type Vector3D = [SpiceDouble; 3];

/// Simple operations on 3D vectors
impl Spice {
    /// Find the separation angle in radians between two double precision, 3-dimensional vectors.
    /// This angle is defined as zero if either vector is zero.
    ///
    /// See [vsep_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/vsep_c.html)
    pub fn vector_separation(vec1: &Vector3D, vec2: &Vector3D) -> SpiceDouble {
        unsafe {
            vsep_c(
                vec1.as_ptr() as *mut SpiceDouble,
                vec2.as_ptr() as *mut SpiceDouble,
            )
        }
    }
}
