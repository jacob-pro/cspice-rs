use crate::convert::SpiceFrom;
use crate::string::{SpiceStr, SpiceString};
use crate::time::calendar::Calendar;
use crate::time::date_time::DateTime;
use crate::time::system::System;
use crate::time::Et;
use crate::Spice;
use cspice_sys::{timout_c, SpiceDouble};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

/// See [Julian Date](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/time.html#Julian%20Date)
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JulianDate<S: System> {
    pub value: SpiceDouble,
    scale: PhantomData<S>,
}

impl<S: System> From<SpiceDouble> for JulianDate<S> {
    fn from(s: SpiceDouble) -> Self {
        JulianDate::new(s)
    }
}

impl<S: System> JulianDate<S> {
    #[inline]
    pub fn new(jd: SpiceDouble) -> Self {
        Self {
            value: jd,
            scale: Default::default(),
        }
    }
}

impl<S: System> SpiceFrom<JulianDate<S>> for Et {
    /// Convert a Julian Date to Ephemeris Time (TDB).
    #[inline]
    fn spice_from(jd: JulianDate<S>, spice: Spice) -> Self {
        Et::from_string(format!("JD {} {}", S::system_name(), jd.value), spice).unwrap()
    }
}

impl<S: System> SpiceFrom<Et> for JulianDate<S> {
    /// Convert Ephemeris Time (TDB) to a Julian Date.
    #[inline]
    fn spice_from(et: Et, spice: Spice) -> Self {
        let pictur = SpiceString::from(format!("JULIAND.############# ::{}", S::system_name()));
        let mut buffer = [0; 40];
        unsafe {
            timout_c(
                et.0,
                pictur.as_mut_ptr(),
                buffer.len() as i32,
                buffer.as_mut_ptr(),
            );
        }
        spice.get_last_error().unwrap();
        Self::new(SpiceStr::from_buffer(&buffer).as_str().parse().unwrap())
    }
}

impl<C: Calendar, S: System> SpiceFrom<DateTime<C, S>> for JulianDate<S> {
    #[inline]
    fn spice_from(dt: DateTime<C, S>, spice: Spice) -> Self {
        JulianDate::spice_from(Et::spice_from(dt, spice), spice)
    }
}

impl<S: System> Display for JulianDate<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "JD {} {}", S::system_name(), self.value)
    }
}
