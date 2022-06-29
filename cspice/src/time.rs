use crate::string::SpiceString;
use crate::SPICE;
use cspice_sys::{str2et_c, timout_c, SpiceDouble, SpiceInt};
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

/// Ephemeris Time (time in seconds past the ephemeris epoch J2000) (TDB)
///
/// See https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/FORTRAN/req/time.html#In%20the%20Toolkit%20ET%20Means%20TDB
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ET(pub SpiceDouble);

impl Display for ET {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ET {}", self.0)
    }
}

impl ET {
    /// Convert an Ephemeris Time (TDB) to a Julian Date
    ///
    /// See https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/timout_c.html
    #[inline]
    pub fn to_julian_date<S: Scale>(&self, spice: SPICE) -> JulianDate<S> {
        let mut pictur = SpiceString::from(format!("JULIAND.############# ::{}", S::name()));
        let mut buffer = vec![0; 40];
        unsafe {
            timout_c(
                self.0,
                pictur.as_mut_ptr(),
                buffer.len() as i32,
                buffer.as_mut_ptr(),
            );
        }
        spice.get_last_error().unwrap();
        JulianDate::new(SpiceString::from_buffer(buffer).as_str().parse().unwrap())
    }

    /// Equivalent to [JulianDate::to_et()]
    #[inline]
    pub fn from_julian_date<S: Scale>(jd: JulianDate<S>, spice: SPICE) -> Self {
        jd.to_et(spice)
    }

    /// Convert an Ephemeris Time (TDB) to a DateTime
    ///
    /// See https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/timout_c.html
    pub fn to_date_time<C: Calendar, S: Scale>(&self, spice: SPICE) -> DateTime<C, S> {
        let mut pictur = SpiceString::from(format!(
            "ERA:YYYY:MM:DD:HR:MN:SC.##### ::{} ::{}",
            S::name(),
            C::short_name()
        ));
        let mut buffer = vec![0; 100];
        unsafe {
            timout_c(
                self.0,
                pictur.as_mut_ptr(),
                buffer.len() as SpiceInt,
                buffer.as_mut_ptr(),
            );
        };
        spice.get_last_error().unwrap();
        let output = SpiceString::from_buffer(buffer).to_string();
        let split: Vec<&str> = output.split(':').collect();
        let year: i16 = if split[0] == "B.C." {
            1 - split[1].trim().parse::<i16>().unwrap()
        } else {
            split[1].trim().parse().unwrap()
        };
        DateTime {
            year,
            month: split[2].parse().unwrap(),
            day: split[3].parse().unwrap(),
            hour: split[4].parse().unwrap(),
            minute: split[5].parse().unwrap(),
            second: split[6].parse().unwrap(),
            calendar: Default::default(),
            scale: Default::default(),
        }
    }

    /// Equivalent to [DateTime::to_et()]
    #[inline]
    pub fn from_date_time<C: Calendar, S: Scale>(date_time: DateTime<C, S>, spice: SPICE) -> Self {
        date_time.to_et(spice)
    }
}

/// See https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/time.html
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JulianDate<S: Scale> {
    pub value: SpiceDouble,
    scale: PhantomData<S>,
}

impl<S: Scale> JulianDate<S> {
    pub fn new(jd: SpiceDouble) -> Self {
        Self {
            value: jd,
            scale: Default::default(),
        }
    }

    /// Converts a Julian Date to Ephemeris Time (TDB)
    ///
    /// See https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/str2et_c.html
    #[inline]
    pub fn to_et(&self, spice: SPICE) -> ET {
        let mut value = SpiceString::from(format!("JD {} {}", S::name(), self.value));
        let mut output = 0f64;
        unsafe {
            str2et_c(value.as_mut_ptr(), &mut output);
        }
        spice.get_last_error().unwrap();
        ET(output)
    }

    /// Equivalent to [ET::to_jd()]
    #[inline]
    pub fn from_et(et: ET, spice: SPICE) -> Self {
        et.to_julian_date(spice)
    }

    #[inline]
    pub fn to_date_time<C: Calendar>(&self, spice: SPICE) -> DateTime<C, S> {
        self.to_et(spice).to_date_time(spice)
    }

    #[inline]
    pub fn from_date_time<C: Calendar>(date_time: DateTime<C, S>, spice: SPICE) -> Self {
        date_time.to_et(spice).to_julian_date(spice)
    }
}

impl<S: Scale> Display for JulianDate<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "JD {} {}", S::name(), self.value)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DateTime<T: Calendar, S: Scale> {
    pub year: i16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: f32,
    calendar: PhantomData<T>,
    scale: PhantomData<S>,
}

impl<C: Calendar, S: Scale> DateTime<C, S> {
    pub fn new(year: i16, month: u8, day: u8, hour: u8, minute: u8, second: f32) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            calendar: Default::default(),
            scale: Default::default(),
        }
    }

    /// Convert a DateTime to Ephemeris Time (TDB)
    ///
    /// See https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/str2et_c.html
    pub fn to_et(&self, spice: SPICE) -> ET {
        let year = if self.year > 0 {
            self.year.to_string()
        } else {
            format!("{} BC", self.year.abs() + 1)
        };
        let mut date = SpiceString::from(format!(
            "{year}-{}-{} {}:{}:{} {} {}",
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
            S::name(),
            C::short_name()
        ));
        let mut et: SpiceDouble = 0.0;
        unsafe {
            str2et_c(date.as_mut_ptr(), &mut et);
        }
        spice.get_last_error().unwrap();
        ET(et)
    }

    #[inline]
    pub fn to_julian_date(&self, spice: SPICE) -> JulianDate<S> {
        self.to_et(spice).to_julian_date(spice)
    }

    #[inline]
    pub fn from_julian_date(jd: JulianDate<S>, spice: SPICE) -> Self {
        jd.to_et(spice).to_date_time(spice)
    }
}

impl<C: Calendar, S: Scale> Display for DateTime<C, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}-{} {}:{}:{} {} {}",
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
            S::name(),
            C::short_name()
        )
    }
}

/// See https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/timout_c.html
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
