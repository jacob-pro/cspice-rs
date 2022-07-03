mod date_time;
mod julian_date;

pub mod calendar;
pub mod scale;

pub use date_time::DateTime;
pub use julian_date::JulianDate;

use crate::constants::{CALENDAR, SET};
use crate::string::{SpiceStr, SpiceString, StringParam};
use crate::{Error, Spice};
use calendar::Calendar;
use cspice_sys::{str2et_c, timdef_c, timout_c, SpiceDouble, SpiceInt};
use scale::Scale;
use std::fmt::{Debug, Display, Formatter};

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

impl Et {
    /// Equivalent to [JulianDate::from_et()].
    #[inline]
    pub fn to_julian_date<S: Scale>(self, spice: Spice) -> JulianDate<S> {
        JulianDate::from_et(self, spice)
    }

    /// Equivalent to [JulianDate::to_et()].
    #[inline]
    pub fn from_julian_date<S: Scale>(jd: JulianDate<S>, spice: Spice) -> Self {
        jd.to_et(spice)
    }

    /// Convert an Ephemeris Time (TDB) to a DateTime in a specified time zone.
    ///
    /// # Arguments
    ///
    /// * `zone` - Zone offset in seconds
    #[inline]
    pub fn to_date_time_with_zone<C: Calendar, S: Scale>(
        &self,
        spice: Spice,
        zone: i32,
    ) -> DateTime<C, S> {
        let pictur = SpiceString::from(format!(
            "ERA:YYYY:MM:DD:HR:MN:SC.##### ::{} ::{}",
            S::name(),
            C::short_name()
        ));
        let mut buffer = [0; 100];
        let zone_adj = self.0 + zone as f64;
        unsafe {
            timout_c(
                zone_adj,
                pictur.as_mut_ptr(),
                buffer.len() as SpiceInt,
                buffer.as_mut_ptr(),
            );
        };
        spice.get_last_error().unwrap();
        let output = SpiceStr::from_buffer(&buffer);
        let cow = output.as_str();
        let split: Vec<&str> = cow.split(':').collect();
        let year: i16 = if split[0] == "B.C." {
            1 - split[1].trim().parse::<i16>().unwrap()
        } else {
            split[1].trim().parse().unwrap()
        };
        DateTime::with_zone(
            year,
            split[2].parse().unwrap(),
            split[3].parse().unwrap(),
            split[4].parse().unwrap(),
            split[5].parse().unwrap(),
            split[6].parse().unwrap(),
            zone,
        )
    }

    /// Convert an Ephemeris Time (TDB) to a DateTime.
    #[inline]
    pub fn to_date_time<C: Calendar, S: Scale>(&self, spice: Spice) -> DateTime<C, S> {
        self.to_date_time_with_zone(spice, 0)
    }

    /// Equivalent to [DateTime::to_et()].
    #[inline]
    pub fn from_date_time<C: Calendar, S: Scale>(date_time: DateTime<C, S>, spice: Spice) -> Self {
        date_time.to_et(spice)
    }
}

/// Functions for converting time formats and scales.
impl Spice {
    /// Convert Ephemeris Time to a different time format.
    ///
    /// `out_length` must be large enough to store the output string or otherwise this function
    /// will return Err.
    ///
    /// See [timout_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/timout_c.html).
    pub fn time_out<'p, P: Into<StringParam<'p>>>(
        &self,
        et: Et,
        pictur: P,
        out_length: usize,
    ) -> Result<String, Error> {
        let mut buffer = vec![0; out_length];
        unsafe {
            timout_c(
                et.0,
                pictur.into().as_mut_ptr(),
                buffer.len() as SpiceInt,
                buffer.as_mut_ptr(),
            );
        };
        self.get_last_error()?;
        Ok(SpiceString::from_buffer(buffer).to_string())
    }

    /// Convert a time string to Ephemeris Time (TDB)
    ///
    /// See [str2et_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/str2et_c.html)
    #[inline]
    pub fn string_to_et<'p, P: Into<StringParam<'p>>>(&self, string: P) -> Result<Et, Error> {
        let mut output = 0f64;
        unsafe {
            str2et_c(string.into().as_mut_ptr(), &mut output);
        }
        self.get_last_error()?;
        Ok(Et(output))
    }

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
    use crate::time::scale::{Tdb, Tdt, Utc};

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
        let ut = et.to_date_time::<Mixed, Tdb>(spice);
        assert_eq!(ut, DateTime::new(-599, 1, 1, 0, 0, 0.0));
    }

    #[test]
    fn test_date_time_to_jd() {
        let spice = get_test_spice();
        let jd = JulianDate::<Tdb>::new(1502273.5);
        assert_eq!(
            DateTime::<Mixed, Tdb>::new(-599, 1, 1, 0, 0, 0.0).to_julian_date(spice),
            jd
        );
        assert_eq!(
            DateTime::<Gregorian, Tdb>::new(-600, 12, 26, 0, 0, 0.0).to_julian_date(spice),
            jd
        );
    }
}
