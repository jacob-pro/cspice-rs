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

impl<S: System> JulianDate<S> {
    #[inline]
    pub fn new(jd: SpiceDouble) -> Self {
        Self {
            value: jd,
            scale: Default::default(),
        }
    }

    /// Convert the Julian Date to Ephemeris Time (TDB).
    #[inline]
    pub fn to_et(&self, spice: Spice) -> Et {
        Et::from_string(format!("JD {} {}", S::system_name(), self.value), spice).unwrap()
    }

    /// Convert Ephemeris Time (TDB) to a Julian Date.
    #[inline]
    pub fn from_et(et: Et, spice: Spice) -> Self {
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

    #[inline]
    pub fn to_date_time<C: Calendar>(&self, spice: Spice) -> DateTime<C, S> {
        DateTime::from_et(self.to_et(spice), S::default(), spice)
    }

    #[inline]
    pub fn from_date_time<C: Calendar>(date_time: DateTime<C, S>, spice: Spice) -> Self {
        JulianDate::from_et(date_time.to_et(spice), spice)
    }
}

impl<S: System> Display for JulianDate<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "JD {} {}", S::system_name(), self.value)
    }
}
