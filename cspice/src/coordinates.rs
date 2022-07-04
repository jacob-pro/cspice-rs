use crate::spice_unsafe;
use crate::vector::Vector3D;
use cspice_sys::{reclat_c, recrad_c, SpiceDouble};
use derive_more::{Deref, DerefMut, From, Into};

/// Rectangular coordinates
#[derive(Copy, Clone, Debug, Default, PartialEq, From, Into, Deref, DerefMut)]
pub struct Rectangular(Vector3D);

impl From<[SpiceDouble; 3]> for Rectangular {
    fn from(d: [SpiceDouble; 3]) -> Self {
        Vector3D::from(d).into()
    }
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

/// Range, right ascension, and declination.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct RaDec {
    pub range: SpiceDouble,
    pub ra: SpiceDouble,
    pub dec: SpiceDouble,
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
