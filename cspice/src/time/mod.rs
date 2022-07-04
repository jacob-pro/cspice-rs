mod date_time;
mod julian_date;

pub mod calendar;
pub mod system;

pub use date_time::DateTime;
pub use julian_date::JulianDate;

use crate::common::{CALENDAR, SET};
use crate::string::{SpiceString, StringParam};
use crate::{Error, Spice};
use calendar::Calendar;
use cspice_sys::{str2et_c, timdef_c, timout_c, SpiceDouble, SpiceInt};
use std::fmt::{Debug, Display, Formatter};
use system::System;

/// Ephemeris Time (time in seconds past the ephemeris epoch J2000) (TDB).
///
/// See [ET Means TDB](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/FORTRAN/req/time.html#In%20the%20Toolkit%20ET%20Means%20TDB).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Et(pub SpiceDouble);

impl Display for Et {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ET {}", self.0)
    }
}

impl From<SpiceDouble> for Et {
    fn from(et: SpiceDouble) -> Self {
        Self(et)
    }
}

impl Et {
    /// Convert Ephemeris Time to a different time format.
    ///
    /// `out_length` must be large enough to store the output string or otherwise this function
    /// will return Err.
    ///
    /// See [timout_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/timout_c.html).
    #[inline]
    pub fn time_out<'p, P: Into<StringParam<'p>>>(
        &self,
        pictur: P,
        out_length: usize,
        spice: Spice,
    ) -> Result<String, Error> {
        let mut buffer = vec![0; out_length];
        unsafe {
            timout_c(
                self.0,
                pictur.into().as_mut_ptr(),
                buffer.len() as SpiceInt,
                buffer.as_mut_ptr(),
            );
        };
        spice.get_last_error()?;
        Ok(SpiceString::from_buffer(buffer).to_string())
    }

    /// Convert a time string to Ephemeris Time (TDB)
    ///
    /// See [str2et_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/str2et_c.html)
    #[inline]
    pub fn from_string<'p, P: Into<StringParam<'p>>>(
        string: P,
        spice: Spice,
    ) -> Result<Self, Error> {
        let mut output = 0f64;
        unsafe {
            str2et_c(string.into().as_mut_ptr(), &mut output);
        }
        spice.get_last_error()?;
        Ok(Self(output))
    }

    /// Equivalent to [JulianDate::from_et()].
    #[inline]
    pub fn to_julian_date<S: System>(self, spice: Spice) -> JulianDate<S> {
        JulianDate::from_et(self, spice)
    }

    /// Equivalent to [JulianDate::to_et()].
    #[inline]
    pub fn from_julian_date<S: System>(jd: JulianDate<S>, spice: Spice) -> Self {
        jd.to_et(spice)
    }

    /// Equivalent to [DateTime::from_et()].
    #[inline]
    pub fn to_date_time<C: Calendar, S: System>(self, system: S, spice: Spice) -> DateTime<C, S> {
        DateTime::from_et(self, system, spice)
    }

    /// Equivalent to [DateTime::to_et()].
    #[inline]
    pub fn from_date_time<C: Calendar, S: System>(date_time: DateTime<C, S>, spice: Spice) -> Self {
        date_time.to_et(spice)
    }
}

/// Functions relating to time conversion
impl Spice {
    /// Sets the default calendar to use with input strings.
    ///
    /// See [timdef_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/timdef_c.html).
    #[inline]
    pub fn set_default_calendar<C: Calendar>(&self) {
        let name = SpiceString::from(C::name());
        unsafe {
            timdef_c(
                SET.as_mut_ptr(),
                CALENDAR.as_mut_ptr(),
                0,
                name.as_mut_ptr(),
            );
        }
        self.get_last_error().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::get_test_spice;
    use crate::time::calendar::{Gregorian, Mixed};
    use crate::time::system::{Tdb, Utc};

    #[test]
    fn test_et_to_jd() {
        let spice = get_test_spice();
        assert_eq!(
            Et(0f64).to_julian_date(spice),
            JulianDate::<Tdb>::new(2451545.0)
        );
    }

    #[test]
    fn test_jd_to_date_time() {
        let spice = get_test_spice();
        let et = JulianDate::<Tdb>::new(1502273.5).to_et(spice);
        let ut = et.to_date_time::<Mixed, Tdb>(Tdb, spice);
        assert_eq!(ut, DateTime::new(-599, 1, 1, 0, 0, 0.0, Tdb));
    }

    #[test]
    fn test_date_time_to_jd() {
        let spice = get_test_spice();
        let jd = JulianDate::<Utc>::new(1502273.5);
        assert_eq!(
            DateTime::<Mixed, _>::new(-599, 1, 1, 0, 0, 0.0, Utc::default()).to_julian_date(spice),
            jd
        );
        assert_eq!(
            DateTime::<Mixed, _>::new(-599, 1, 1, 3, 0, 0.0, Utc::new(3, 0)).to_julian_date(spice),
            jd
        );
        assert_eq!(
            DateTime::<Gregorian, _>::new(-600, 12, 26, 0, 0, 0.0, Utc::default())
                .to_julian_date(spice),
            jd
        );
    }
}
