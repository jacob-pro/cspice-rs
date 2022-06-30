mod date_time;
mod julian_date;

pub use date_time::DateTime;
pub use julian_date::JulianDate;

use crate::string::{SpiceStr, SpiceString, StringParam};
use crate::{Error, SPICE};
use cspice_sys::{str2et_c, timout_c, SpiceDouble, SpiceInt};
use std::fmt::{Debug, Display, Formatter};

/// Ephemeris Time (time in seconds past the ephemeris epoch J2000) (TDB)
///
/// See <https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/FORTRAN/req/time.html#In%20the%20Toolkit%20ET%20Means%20TDB>
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ET(pub SpiceDouble);

impl Display for ET {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ET {}", self.0)
    }
}

impl ET {
    /// Convert an Ephemeris Time (TDB) to a Julian Date
    #[inline]
    pub fn to_julian_date<S: Scale>(&self, spice: SPICE) -> JulianDate<S> {
        let pictur = SpiceString::from(format!("JULIAND.############# ::{}", S::name()));
        let mut buffer = [0; 40];
        unsafe {
            timout_c(
                self.0,
                pictur.as_mut_ptr(),
                buffer.len() as i32,
                buffer.as_mut_ptr(),
            );
        }
        spice.get_last_error().unwrap();
        JulianDate::new(SpiceStr::from_buffer(&buffer).as_str().parse().unwrap())
    }

    /// Equivalent to [JulianDate::to_et()]
    #[inline]
    pub fn from_julian_date<S: Scale>(jd: JulianDate<S>, spice: SPICE) -> Self {
        jd.to_et(spice)
    }

    /// Convert an Ephemeris Time (TDB) to a DateTime in a specified time zone
    ///
    /// # Arguments
    ///
    /// * `zone` - Zone offset in seconds
    #[inline]
    pub fn to_date_time_with_zone<C: Calendar, S: Scale>(
        &self,
        spice: SPICE,
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

    /// Convert an Ephemeris Time (TDB) to a DateTime
    #[inline]
    pub fn to_date_time<C: Calendar, S: Scale>(&self, spice: SPICE) -> DateTime<C, S> {
        self.to_date_time_with_zone(spice, 0)
    }

    /// Equivalent to [DateTime::to_et()]
    #[inline]
    pub fn from_date_time<C: Calendar, S: Scale>(date_time: DateTime<C, S>, spice: SPICE) -> Self {
        date_time.to_et(spice)
    }
}

impl SPICE {
    /// Convert Ephemeris Time to a different time format.
    ///
    /// `out_length` must be large enough to store the output string or otherwise this function
    /// will return Err.
    ///
    /// See <https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/timout_c.html>
    pub fn time_out<'p, P: Into<StringParam<'p>>>(
        &self,
        et: ET,
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
    /// See <https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/str2et_c.html>
    #[inline]
    pub fn string_to_et<'p, P: Into<StringParam<'p>>>(&self, string: P) -> Result<ET, Error> {
        let mut output = 0f64;
        unsafe {
            str2et_c(string.into().as_mut_ptr(), &mut output);
        }
        self.get_last_error()?;
        Ok(ET(output))
    }
}

/// See <https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/time.html>
pub trait Scale {
    fn name() -> &'static str;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TDT;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TDB;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UTC;

impl Scale for TDT {
    fn name() -> &'static str {
        "TDT"
    }
}

impl Scale for TDB {
    fn name() -> &'static str {
        "TDB"
    }
}

impl Scale for UTC {
    fn name() -> &'static str {
        "UTC"
    }
}

/// See <https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/timout_c.html>
pub trait Calendar {
    fn short_name() -> &'static str;
}

/// Uses the Julian calendar for dates prior to Oct 5, 1582, and the Gregorian calendar for dates
/// after Oct 15, 1582
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mixed;
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Gregorian;
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Julian;

impl Calendar for Mixed {
    fn short_name() -> &'static str {
        "MCAL"
    }
}

impl Calendar for Gregorian {
    fn short_name() -> &'static str {
        "GCAL"
    }
}

impl Calendar for Julian {
    fn short_name() -> &'static str {
        "JCAL"
    }
}

#[cfg(feature = "chrono")]
impl From<DateTime<Gregorian, UTC>> for chrono::DateTime<chrono::FixedOffset> {
    fn from(t: DateTime<Gregorian, UTC>) -> Self {
        use chrono::TimeZone;
        let ns = t.second.fract() * 1_000_000_f32;
        chrono::FixedOffset::east(0)
            .ymd(t.year as i32, t.month as u32, t.day as u32)
            .and_hms_nano(
                t.hour as u32,
                t.minute as u32,
                t.second.floor() as u32,
                ns as u32,
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::get_test_spice;

    #[test]
    fn test_et_to_jd() {
        let spice = get_test_spice();
        assert_eq!(
            ET(0f64).to_julian_date(spice),
            JulianDate::<TDB>::new(2451545.0)
        );
    }

    #[test]
    fn test_jd_to_utc() {
        let spice = get_test_spice();
        let et = JulianDate::<TDB>::new(1502273.5).to_et(spice);
        let ut = et.to_date_time::<Mixed, TDB>(spice);
        assert_eq!(ut, DateTime::new(-599, 1, 1, 0, 0, 0.0));
    }

    #[test]
    fn test_utc_to_jd() {
        let spice = get_test_spice();
        let et = JulianDate::<TDB>::new(1502273.5).to_et(spice);
        assert_eq!(
            et.to_date_time::<Mixed, TDB>(spice),
            DateTime::new(-599, 1, 1, 0, 0, 0.0)
        );
        assert_eq!(
            et.to_date_time::<Gregorian, TDB>(spice),
            DateTime::new(-600, 12, 26, 0, 0, 0.0)
        );
    }
}
