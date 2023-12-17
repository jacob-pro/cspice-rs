//! Functions for working with 3D Vectors.
//!
//! See [Performing simple operations on 3D vectors](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/info/mostused.html#U)
use crate::coordinates::Rectangular;
use crate::with_spice_lock_or_panic;
use cspice_sys::{vsep_c, SpiceDouble};
use derive_more::{Deref, DerefMut, From, Into};

/// A 3D vector
#[derive(Copy, Clone, Debug, Default, PartialEq, From, Into, Deref, DerefMut)]
pub struct Vector3D(pub [SpiceDouble; 3]);

impl Vector3D {
    /// Find the separation angle in radians between two double precision, 3-dimensional vectors.
    /// This angle is defined as zero if either vector is zero.
    ///
    /// See [vsep_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/vsep_c.html)
    pub fn separation_angle(&self, other: &Vector3D) -> SpiceDouble {
        with_spice_lock_or_panic(|| unsafe {
            vsep_c(
                self.as_ptr() as *mut SpiceDouble,
                other.as_ptr() as *mut SpiceDouble,
            )
        })
    }
}

impl From<Rectangular> for Vector3D {
    fn from(rect: Rectangular) -> Self {
        Self([rect.x, rect.y, rect.z])
    }
}
