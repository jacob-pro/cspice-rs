use crate::convert::SpiceFrom;
use crate::Spice;
use cspice_sys::{reclat_c, recrad_c, SpiceDouble};
use derive_more::{Deref, DerefMut, From, Into};

/// Rectangular coordinates
#[derive(Copy, Clone, Debug, Default, PartialEq, From, Into, Deref, DerefMut)]
pub struct Rectangular([SpiceDouble; 3]);

impl SpiceFrom<Rectangular> for RaDec {
    /// See [recrad_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/recrad_c.html).
    fn from(rect: Rectangular, _: Spice) -> Self {
        let mut ra_dec = RaDec::default();
        unsafe {
            recrad_c(
                rect.as_ptr() as *mut SpiceDouble,
                &mut ra_dec.range,
                &mut ra_dec.ra,
                &mut ra_dec.dec,
            )
        };
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

impl SpiceFrom<Rectangular> for Latitudinal {
    /// See [reclat_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/reclat_c.html).
    fn from(rect: Rectangular, _: Spice) -> Self {
        let mut lat = Latitudinal::default();
        unsafe {
            reclat_c(
                rect.0.as_ptr() as *mut SpiceDouble,
                &mut lat.radius,
                &mut lat.longitude,
                &mut lat.latitude,
            )
        };
        lat
    }
}
