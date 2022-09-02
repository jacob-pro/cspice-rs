//! Functions for converting between different types of coordinates.
use crate::spice_unsafe;
use crate::vector::Vector3D;
use cspice_sys::{azlrec_c, recazl_c, reclat_c, recrad_c, SpiceBoolean, SpiceDouble};
use derive_more::{Deref, DerefMut, From, Into};

/// Rectangular coordinates
#[derive(Copy, Clone, Debug, Default, PartialEq, From, Into, Deref, DerefMut)]
pub struct Rectangular(Vector3D);

impl From<[SpiceDouble; 3]> for Rectangular {
    fn from(d: [SpiceDouble; 3]) -> Self {
        Vector3D::from(d).into()
    }
}

/// Range, azimuth, and elevation
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct AzEl {
    pub range: SpiceDouble,
    pub az: SpiceDouble,
    pub el: SpiceDouble,
}

impl AzEl {
    /// See [azlrec_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/azlrec_c.html)
    pub fn to_rect(&self, azccw: bool, elplsz: bool) -> Rectangular {
        let rect = Rectangular::default();
        spice_unsafe!({
            azlrec_c(
                self.range as SpiceDouble,
                self.az as SpiceDouble,
                self.el as SpiceDouble,
                azccw as SpiceBoolean,
                elplsz as SpiceBoolean,
                rect.as_ptr() as *mut SpiceDouble,
            )
        });
        rect
    }
}

impl Rectangular {
    /// See [recazl_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/recazl_c.html)
    pub fn to_azel(&self, azccw: bool, elplsz: bool) -> AzEl {
        let mut az_el = AzEl::default();
        spice_unsafe!({
            recazl_c(
                self.as_ptr() as *mut SpiceDouble,
                azccw as SpiceBoolean,
                elplsz as SpiceBoolean,
                &mut az_el.range,
                &mut az_el.az,
                &mut az_el.el,
            )
        });
        az_el
    }

    pub fn from_azel(azel: &AzEl, azccw: bool, elplsz: bool) -> Self {
        let rect = Rectangular::default();
        spice_unsafe!({
            azlrec_c(
                azel.range as SpiceDouble,
                azel.az as SpiceDouble,
                azel.el as SpiceDouble,
                azccw as SpiceBoolean,
                elplsz as SpiceBoolean,
                rect.as_ptr() as *mut SpiceDouble,
            )
        });
        rect
    }
}

/// Range, right ascension, and declination.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct RaDec {
    pub range: SpiceDouble,
    pub ra: SpiceDouble,
    pub dec: SpiceDouble,
}

impl From<Rectangular> for RaDec {
    /// See [recrad_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/recrad_c.html).
    fn from(rect: Rectangular) -> Self {
        let mut ra_dec = RaDec::default();
        spice_unsafe!({
            recrad_c(
                rect.as_ptr() as *mut SpiceDouble,
                &mut ra_dec.range,
                &mut ra_dec.ra,
                &mut ra_dec.dec,
            )
        });
        ra_dec
    }
}

/// Latitudinal coordinates.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Latitudinal {
    pub radius: SpiceDouble,
    pub longitude: SpiceDouble,
    pub latitude: SpiceDouble,
}

impl From<Rectangular> for Latitudinal {
    /// See [reclat_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/reclat_c.html).
    fn from(rect: Rectangular) -> Self {
        let mut lat = Latitudinal::default();
        spice_unsafe!({
            reclat_c(
                rect.0.as_ptr() as *mut SpiceDouble,
                &mut lat.radius,
                &mut lat.longitude,
                &mut lat.latitude,
            )
        });
        lat
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn azel_to_rect() {
        let test1: [[f64; 6]; 11] = [
            [0.000, 0.000, 0.000, 0.000, 0.000, 0.000],
            [1.000, 0.000, 0.000, 1.000, 0.000, 0.000],
            [0.000, 1.000, 0.000, 1.000, 270.000, 0.000],
            [0.000, 0.000, 1.000, 1.000, 0.000, -90.000],
            [-1.000, 0.000, 0.000, 1.000, 180.000, 0.000],
            [0.000, -1.000, 0.000, 1.000, 90.000, 0.000],
            [0.000, 0.000, -1.000, 1.000, 0.000, 90.000],
            [1.000, 1.000, 0.000, 1.414, 315.000, 0.000], // These have much higher deltas ~0.00015
            [1.000, 0.000, 1.000, 1.414, 0.000, -45.000], // These have much higher deltas ~0.00015
            [0.000, 1.000, 1.000, 1.414, 270.000, -45.000], // These have much higher deltas ~0.00015
            [1.000, 1.000, 1.000, 1.732, 315.000, -35.264],
        ];
        for test in test1.iter() {
            let azel = AzEl {
                range: test[3] as SpiceDouble,
                az: test[4].to_radians() as SpiceDouble,
                el: test[5].to_radians() as SpiceDouble,
            };
            let rect = azel.to_rect(false, false);
            println!(
                "0: {:?}\t 1: {:?}\t 2: {:?}",
                f64::abs(rect.0[0] - test[0]),
                f64::abs(rect.0[1] - test[1]),
                f64::abs(rect.0[2] - test[2])
            );
            // Passes at this epsilon but we have issues when trying to constrain further
            let epsilon: f64 = 0.001;
            assert!(f64::abs(rect.0[0] - test[0]) < epsilon);
            assert!(f64::abs(rect.0[1] - test[1]) < epsilon);
            assert!(f64::abs(rect.0[2] - test[2]) < epsilon);
        }
    }
}
