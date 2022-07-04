mod date_time;
mod julian_date;

pub mod calendar;
pub mod system;

pub use date_time::DateTime;
pub use julian_date::JulianDate;

use crate::common::{CALENDAR, SET};
use crate::string::{SpiceString, StringParam};
use crate::{spice_unsafe, Error, Spice};
use calendar::Calendar;
use cspice_sys::{str2et_c, timdef_c, timout_c, SpiceDouble, SpiceInt};
use derive_more::{From, Into};
use std::fmt::{Debug, Display, Formatter};

/// Ephemeris Time (time in seconds past the ephemeris epoch J2000) (TDB).
///
/// See [ET Means TDB](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/FORTRAN/req/time.html#In%20the%20Toolkit%20ET%20Means%20TDB).
#[derive(Copy, Clone, Debug, PartialEq, From, Into)]
pub struct Et(pub SpiceDouble);

impl Display for Et {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ET {}", self.0)
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
    ) -> Result<String, Error> {
        let mut buffer = vec![0; out_length];
        spice_unsafe!({
            timout_c(
                self.0,
                pictur.into().as_mut_ptr(),
                buffer.len() as SpiceInt,
                buffer.as_mut_ptr(),
            );
        });
        Spice::get_last_error()?;
        Ok(SpiceString::from_buffer(buffer).to_string())
    }

    /// Convert a time string to Ephemeris Time (TDB)
    ///
    /// See [str2et_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/str2et_c.html)
    #[inline]
    pub fn from_string<'p, P: Into<StringParam<'p>>>(string: P) -> Result<Self, Error> {
        let mut output = 0f64;
        spice_unsafe!({
            str2et_c(string.into().as_mut_ptr(), &mut output);
        });
        Spice::get_last_error()?;
        Ok(Self(output))
    }
}

/// Functions relating to time conversion
impl Spice {
    /// Sets the default calendar to use with input strings.
    ///
    /// See [timdef_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/timdef_c.html).
    #[inline]
    pub fn set_default_calendar<C: Calendar>() {
        let name = SpiceString::from(C::name());
        spice_unsafe!({
            timdef_c(
                SET.as_mut_ptr(),
                CALENDAR.as_mut_ptr(),
                0,
                name.as_mut_ptr(),
            );
        });
        Self::get_last_error().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::load_test_data;
    use crate::time::calendar::{Gregorian, Mixed};
    use crate::time::system::{Tdb, Utc};

    #[test]
    fn test_et_to_jd() {
        load_test_data();
        assert_eq!(
            JulianDate::from(Et(0f64)),
            JulianDate::<Tdb>::new(2451545.0)
        );
    }

    #[test]
    fn test_jd_to_date_time() {
        load_test_data();
        let et = Et::from(JulianDate::<Tdb>::new(1502273.5));
        let dt = DateTime::<Mixed, _>::from_et(et, Tdb);
        assert_eq!(dt, DateTime::new(-599, 1, 1, 0, 0, 0.0, Tdb));
    }

    #[test]
    fn test_date_time_to_jd() {
        load_test_data();
        let jd = JulianDate::<Utc>::new(1502273.5);
        assert_eq!(
            JulianDate::from(DateTime::<Mixed, _>::new(
                -599,
                1,
                1,
                0,
                0,
                0.0,
                Utc::default()
            ),),
            jd
        );
        assert_eq!(
            JulianDate::from(DateTime::<Mixed, _>::new(
                -599,
                1,
                1,
                3,
                0,
                0.0,
                Utc::new(3, 0)
            ),),
            jd
        );
        assert_eq!(
            JulianDate::from(DateTime::<Gregorian, _>::new(
                -600,
                12,
                26,
                0,
                0,
                0.0,
                Utc::default()
            ),),
            jd
        );
    }
}
