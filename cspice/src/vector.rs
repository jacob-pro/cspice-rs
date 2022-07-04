use crate::spice_unsafe;
use cspice_sys::{vsep_c, SpiceDouble};
use derive_more::{Deref, DerefMut, From, Into};

/// A 3D vector
#[derive(Copy, Clone, Debug, Default, PartialEq, From, Into, Deref, DerefMut)]
pub struct Vector3D([SpiceDouble; 3]);

impl Vector3D {
    /// Find the separation angle in radians between two double precision, 3-dimensional vectors.
    /// This angle is defined as zero if either vector is zero.
    ///
    /// See [vsep_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/vsep_c.html)
    pub fn separation_angle(&self, other: &Vector3D) -> SpiceDouble {
        spice_unsafe!({
            vsep_c(
                self.as_ptr() as *mut SpiceDouble,
                other.as_ptr() as *mut SpiceDouble,
            )
        })
    }
}
